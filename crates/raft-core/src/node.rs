//! Main Raft node implementation.
//!
//! This module contains the RaftNode struct which orchestrates the entire Raft consensus
//! algorithm, including leader election, log replication, and state machine management.

use crate::config::RaftConfig;
use crate::error::{RaftError, RaftResult};
use crate::log::{Log, LogEntry, LogIndex, Term};
use crate::message::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    Message, VoteRequest, VoteResponse, ClientRequest, ClientResponse,
};
use crate::state::RaftState;

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, sleep, Interval};
use tracing::{error, info, warn};

/// Trait for handling applied log entries
#[async_trait]
pub trait StateMachine: Send + Sync {
    /// Apply a log entry to the state machine
    async fn apply(&mut self, entry: &LogEntry) -> RaftResult<Vec<u8>>;
    
    /// Take a snapshot of the current state
    async fn take_snapshot(&mut self) -> RaftResult<Vec<u8>>;
    
    /// Restore state from a snapshot
    async fn restore_from_snapshot(&mut self, snapshot: &[u8]) -> RaftResult<()>;
}

/// Channel for sending messages to other nodes
pub type MessageSender = mpsc::UnboundedSender<(u64, Message)>;

/// Channel for receiving messages from other nodes
pub type MessageReceiver = mpsc::UnboundedReceiver<(u64, Message)>;

/// Main Raft node implementation
pub struct RaftNode {
    /// Node configuration
    config: RaftConfig,
    
    /// Current state of the node
    state: Arc<RwLock<RaftState>>,
    
    /// Replicated log
    log: Arc<RwLock<Log>>,
    
    /// State machine
    state_machine: Arc<RwLock<dyn StateMachine>>,
    
    /// Channel for sending messages to other nodes
    message_sender: MessageSender,
    
    /// Channel for receiving messages from other nodes
    message_receiver: MessageReceiver,
    
    /// Channel for receiving client requests
    client_receiver: mpsc::UnboundedReceiver<ClientRequest>,
    
    /// Channel for sending client responses
    client_sender: mpsc::UnboundedSender<ClientResponse>,
    
    /// Heartbeat interval timer
    heartbeat_timer: Option<Interval>,
    
    /// Election timeout timer
    election_timeout: Option<Duration>,
    
    /// Shutdown signal
    shutdown: tokio::sync::broadcast::Receiver<()>,
}

impl RaftNode {
    /// Create a new Raft node
    pub fn new(
        config: RaftConfig,
        state_machine: Arc<RwLock<dyn StateMachine>>,
        message_sender: MessageSender,
        message_receiver: MessageReceiver,
        client_receiver: mpsc::UnboundedReceiver<ClientRequest>,
        client_sender: mpsc::UnboundedSender<ClientResponse>,
        shutdown: tokio::sync::broadcast::Receiver<()>,
    ) -> RaftResult<Self> {
        config.validate()?;
        
        let state = Arc::new(RwLock::new(RaftState::new(config.node_id)));
        let log = Arc::new(RwLock::new(Log::new()));
        
        Ok(Self {
            config,
            state,
            log,
            state_machine,
            message_sender,
            message_receiver,
            client_receiver,
            client_sender,
            heartbeat_timer: None,
            election_timeout: None,
            shutdown,
        })
    }

    /// Start the Raft node
    pub async fn run(mut self) -> RaftResult<()> {
        info!("Starting Raft node {}", self.config.node_id);
        
        // Initialize as follower
        self.reset_election_timeout().await?;
        
        // Main event loop
        loop {
            tokio::select! {
                // Handle shutdown signal
                _ = self.shutdown.recv() => {
                    info!("Shutting down Raft node {}", self.config.node_id);
                    break;
                }
                
                // Handle incoming messages
                Some((sender, message)) = self.message_receiver.recv() => {
                    if let Err(e) = self.handle_message(sender, message).await {
                        error!("Error handling message: {}", e);
                    }
                }
                
                // Handle client requests
                Some(request) = self.client_receiver.recv() => {
                    if let Err(e) = self.handle_client_request(request).await {
                        error!("Error handling client request: {}", e);
                    }
                }
                
                // Handle heartbeat timer (if leader)
                _ = async {
                    if let Some(ref mut timer) = self.heartbeat_timer {
                        timer.tick().await;
                    } else {
                        std::future::pending().await
                    }
                } => {
                    if let Err(e) = self.send_heartbeats().await {
                        error!("Error sending heartbeats: {}", e);
                    }
                }
                
                // Handle election timeout
                _ = async {
                    if let Some(timeout_duration) = self.election_timeout {
                        sleep(timeout_duration).await;
                    } else {
                        std::future::pending().await
                    }
                } => {
                    if let Err(e) = self.handle_election_timeout().await {
                        error!("Error handling election timeout: {}", e);
                    }
                }
                
                // Apply committed entries
                _ = sleep(Duration::from_millis(10)) => {
                    if let Err(e) = self.apply_committed_entries().await {
                        error!("Error applying committed entries: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(&mut self, sender: u64, message: Message) -> RaftResult<()> {
        let _current_term = {
            let state = self.state.read().await;
            state.current_term()
        };
        
        // Check if message is from a valid cluster member
        if !self.config.is_member(sender) {
            warn!("Received message from non-member node {}", sender);
            return Ok(());
        }
        
        match message {
            Message::AppendEntriesRequest(request) => {
                self.handle_append_entries_request(sender, request).await
            }
            Message::AppendEntriesResponse(response) => {
                self.handle_append_entries_response(sender, response).await
            }
            Message::VoteRequest(request) => {
                self.handle_vote_request(sender, request).await
            }
            Message::VoteResponse(response) => {
                self.handle_vote_response(sender, response).await
            }
            Message::InstallSnapshotRequest(request) => {
                self.handle_install_snapshot_request(sender, request).await
            }
            Message::InstallSnapshotResponse(response) => {
                self.handle_install_snapshot_response(sender, response).await
            }
            Message::Heartbeat(request) => {
                self.handle_append_entries_request(sender, request).await
            }
            _ => {
                warn!("Received unexpected message type: {:?}", message.message_type());
                Ok(())
            }
        }
    }

    /// Handle AppendEntries request
    async fn handle_append_entries_request(
        &mut self,
        sender: u64,
        request: AppendEntriesRequest,
    ) -> RaftResult<()> {
        let should_reset_timeout = {
            let mut state = self.state.write().await;
            
            // Update term if necessary
            if request.term > state.current_term() {
                state.update_term(request.term)?;
                state.become_follower(request.term, self.config.random_election_timeout());
            }
            
            // Reject if term is stale
            if request.term < state.current_term() {
                let response = AppendEntriesResponse::failure(
                    request.message_id,
                    state.current_term(),
                    self.config.node_id,
                    None,
                    Some("Stale term".to_string()),
                );
                
                drop(state);
                self.send_message(sender, Message::AppendEntriesResponse(response)).await?;
                return Ok(());
            }
            
            // Convert to follower if necessary
            if !state.is_follower() {
                state.become_follower(request.term, self.config.random_election_timeout());
            }
            
            // Update leader information
            if let Some(follower_state) = state.follower_state_mut() {
                follower_state.update_leader(request.leader_id);
            }
            
            true
        };
        
        // Process the request
        let response = self.process_append_entries_request(request).await?;
        self.send_message(sender, Message::AppendEntriesResponse(response)).await?;
        
        // Reset election timeout
        if should_reset_timeout {
            self.reset_election_timeout().await?;
        }
        
        Ok(())
    }

    /// Process AppendEntries request logic
    async fn process_append_entries_request(
        &self,
        request: AppendEntriesRequest,
    ) -> RaftResult<AppendEntriesResponse> {
        let mut log = self.log.write().await;
        let state = self.state.read().await;
        
        // Check log consistency
        if request.prev_log_index > 0 {
            if request.prev_log_index > log.last_log_index() {
                // Log is too short
                return Ok(AppendEntriesResponse::failure(
                    request.message_id,
                    state.current_term(),
                    self.config.node_id,
                    Some(log.last_log_index() + 1),
                    Some("Log too short".to_string()),
                ));
            }
            
            if !log.matches(request.prev_log_index, request.prev_log_term) {
                // Log doesn't match
                let next_index = self.find_next_index(&log, request.prev_log_index, request.prev_log_term)?;
                return Ok(AppendEntriesResponse::failure(
                    request.message_id,
                    state.current_term(),
                    self.config.node_id,
                    Some(next_index),
                    Some("Log inconsistency".to_string()),
                ));
            }
        }
        
        // Handle entries
        if !request.entries.is_empty() {
            // Find conflicting entry
            let mut conflict_index = None;
            for (i, entry) in request.entries.iter().enumerate() {
                let entry_index = request.prev_log_index + 1 + i as LogIndex;
                if entry_index <= log.last_log_index() {
                    if !log.matches(entry_index, entry.term) {
                        conflict_index = Some(entry_index);
                        break;
                    }
                } else {
                    break;
                }
            }
            
            // Remove conflicting entries
            if let Some(index) = conflict_index {
                log.truncate_from(index)?;
            }
            
            // Append new entries
            let start_index = if let Some(index) = conflict_index {
                index
            } else {
                log.last_log_index() + 1
            };
            
            let entries_to_append: Vec<LogEntry> = request.entries
                .into_iter()
                .skip((start_index - request.prev_log_index - 1) as usize)
                .collect();
            
            for entry in entries_to_append {
                log.append_entry(entry)?;
            }
        }
        
        // Update commit index
        if request.leader_commit > log.commit_index() {
            let new_commit = std::cmp::min(request.leader_commit, log.last_log_index());
            log.set_commit_index(new_commit)?;
        }
        
        Ok(AppendEntriesResponse::success(
            request.message_id,
            state.current_term(),
            self.config.node_id,
        ))
    }

    /// Handle AppendEntries response
    async fn handle_append_entries_response(
        &mut self,
        sender: u64,
        response: AppendEntriesResponse,
    ) -> RaftResult<()> {
        let should_reset_timeout = {
            let mut state = self.state.write().await;
            
            // Only process if we're still leader
            if !state.is_leader() {
                return Ok(());
            }
            
            // Update term if necessary
            if response.term > state.current_term() {
                state.update_term(response.term)?;
                state.become_follower(response.term, self.config.random_election_timeout());
                return Ok(()); // We'll reset timeout after this block
            }
            
            // Ignore stale responses
            if response.term < state.current_term() {
                return Ok(());
            }
            
            let current_commit = state.commit_index();
            let leader_state = state.leader_state_mut().unwrap();
            leader_state.remove_pending_response(sender);
            
            if response.success {
                // Update match and next indices
                let match_index = leader_state.next_index.get(&sender).copied().unwrap_or(0);
                leader_state.update_match_index(sender, match_index);
                
                // Update commit index
                let new_commit = leader_state.calculate_commit_index(
                    current_commit,
                    self.config.quorum_size(),
                );
                
                if new_commit > current_commit {
                    state.set_commit_index(new_commit);
                    // We'll update the log outside this block
                    return Ok(());
                }
            } else {
                // Decrement next index and retry
                if let Some(next_index) = response.next_index {
                    leader_state.update_next_index(sender, next_index);
                } else {
                    let current_next = leader_state.next_index.get(&sender).copied().unwrap_or(1);
                    leader_state.update_next_index(sender, std::cmp::max(1, current_next - 1));
                }
            }
            
            false
        };
        
        // Update log commit index if needed
        if should_reset_timeout {
            let new_commit = {
                let state = self.state.read().await;
                state.commit_index()
            };
            let mut log = self.log.write().await;
            log.set_commit_index(new_commit)?;
        }
        
        Ok(())
    }

    /// Handle RequestVote request
    async fn handle_vote_request(&mut self, sender: u64, request: VoteRequest) -> RaftResult<()> {
        let (response, should_reset_timeout) = {
            let mut state = self.state.write().await;
            
            let mut should_reset = false;
            
            // Update term if necessary
            if request.term > state.current_term() {
                state.update_term(request.term)?;
                state.become_follower(request.term, self.config.random_election_timeout());
                should_reset = true;
            }
            
            let response = if self.should_grant_vote(&state, &request).await? {
                if !request.pre_vote {
                    state.vote_for(request.candidate_id)?;
                }
                VoteResponse::granted(request.message_id, state.current_term(), self.config.node_id)
            } else {
                let reason = self.vote_denial_reason(&state, &request).await?;
                VoteResponse::denied(request.message_id, state.current_term(), self.config.node_id, reason)
            };
            
            (response, should_reset)
        };
        
        self.send_message(sender, Message::VoteResponse(response)).await?;
        
        if should_reset_timeout {
            self.reset_election_timeout().await?;
        }
        
        Ok(())
    }

    /// Handle RequestVote response
    async fn handle_vote_response(&mut self, sender: u64, response: VoteResponse) -> RaftResult<()> {
        let mut state = self.state.write().await;
        
        // Only process if we're still candidate
        if !state.is_candidate() {
            return Ok(());
        }
        
        // Update term if necessary
        if response.term > state.current_term() {
            state.update_term(response.term)?;
            state.become_follower(response.term, self.config.random_election_timeout());
            drop(state);
            self.reset_election_timeout().await?;
            return Ok(());
        }
        
        // Ignore stale responses
        if response.term < state.current_term() {
            return Ok(());
        }
        
        // Add vote if granted
        if response.vote_granted {
            if let Some(candidate_state) = state.candidate_state_mut() {
                candidate_state.add_vote(sender);
                
                // Check if we have majority
                if candidate_state.has_majority(self.config.cluster_size()) {
                    // Become leader
                    let log = self.log.read().await;
                    let last_log_index = log.last_log_index();
                    drop(log);
                    
                    let current_term = state.current_term();
                    state.become_leader(&self.config.peers, last_log_index)?;
                    drop(state);
                    
                    self.start_heartbeat_timer().await?;
                    
                    info!("Node {} became leader for term {}", self.config.node_id, current_term);
                    
                    // Send initial heartbeats
                    self.send_heartbeats().await?;
                }
            }
        }
        
        Ok(())
    }

    /// Handle InstallSnapshot request
    async fn handle_install_snapshot_request(
        &mut self,
        sender: u64,
        request: InstallSnapshotRequest,
    ) -> RaftResult<()> {
        let mut state = self.state.write().await;
        
        // Update term if necessary
        if request.term > state.current_term() {
            state.update_term(request.term)?;
            state.become_follower(request.term, self.config.random_election_timeout());
            drop(state);
            self.reset_election_timeout().await?;
            return Ok(());
        }
        
        // Reject if term is stale
        if request.term < state.current_term() {
            let response = InstallSnapshotResponse::failure(
                request.message_id,
                state.current_term(),
                self.config.node_id,
                "Stale term".to_string(),
            );
            
            drop(state);
            self.send_message(sender, Message::InstallSnapshotResponse(response)).await?;
            return Ok(());
        }
        
        // Update leader information
        if let Some(follower_state) = state.follower_state_mut() {
            follower_state.update_leader(request.leader_id);
        }
        
        drop(state);
        
        // Process snapshot
        let response = self.process_install_snapshot_request(request).await?;
        self.send_message(sender, Message::InstallSnapshotResponse(response)).await?;
        
        // Reset election timeout
        self.reset_election_timeout().await?;
        
        Ok(())
    }

    /// Handle InstallSnapshot response
    async fn handle_install_snapshot_response(
        &mut self,
        sender: u64,
        response: InstallSnapshotResponse,
    ) -> RaftResult<()> {
        let mut state = self.state.write().await;
        
        // Only process if we're still leader
        if !state.is_leader() {
            return Ok(());
        }
        
        // Update term if necessary
        if response.term > state.current_term() {
            state.update_term(response.term)?;
            state.become_follower(response.term, self.config.random_election_timeout());
            drop(state);
            self.reset_election_timeout().await?;
            return Ok(());
        }
        
        // Process successful snapshot installation
        if response.success {
            if let Some(leader_state) = state.leader_state_mut() {
                // Update indices after successful snapshot
                leader_state.update_match_index(sender, 0); // Will be updated with actual index
                leader_state.update_next_index(sender, 1);
            }
        }
        
        Ok(())
    }

    /// Handle client request
    async fn handle_client_request(&mut self, request: ClientRequest) -> RaftResult<()> {
        let state = self.state.read().await;
        
        // Only leader can handle write requests
        if !state.is_leader() && !request.read_only {
            let leader_hint = if let Some(follower_state) = state.follower_state() {
                follower_state.leader_id
            } else {
                None
            };
            
            let response = ClientResponse::failure(
                request.message_id,
                "Not leader".to_string(),
                leader_hint,
            );
            
            self.client_sender.send(response).map_err(|_| {
                RaftError::internal("Failed to send client response")
            })?;
            
            return Ok(());
        }
        
        // Handle read-only requests
        if request.read_only {
            // For read-only requests, we need to ensure we're still leader
            // This is a simplified implementation
            let response = ClientResponse::success(request.message_id, Some(b"read-only".to_vec()));
            self.client_sender.send(response).map_err(|_| {
                RaftError::internal("Failed to send client response")
            })?;
            return Ok(());
        }
        
        // Create log entry for write request
        let term = state.current_term();
        drop(state);
        
        let mut log = self.log.write().await;
        let entry = LogEntry::new_command(term, log.last_log_index() + 1, request.command);
        log.append_entry(entry)?;
        
        // Response will be sent when entry is committed and applied
        // This is a simplified implementation - in practice, you'd track pending requests
        
        Ok(())
    }

    /// Handle election timeout
    async fn handle_election_timeout(&mut self) -> RaftResult<()> {
        let mut state = self.state.write().await;
        
        // Only followers and candidates can timeout
        if state.is_leader() {
            return Ok(());
        }
        
        // Check if we actually timed out
        if state.is_follower() {
            if let Some(follower_state) = state.follower_state() {
                if !follower_state.has_election_timeout() {
                    return Ok(());
                }
            }
        }
        
        info!("Election timeout for node {}", self.config.node_id);
        
        // Start election
        self.start_election(&mut state).await?;
        drop(state);
        
        // Reset election timeout
        self.reset_election_timeout().await?;
        
        Ok(())
    }

    /// Start a new election
    async fn start_election(&self, state: &mut RaftState) -> RaftResult<()> {
        let election_timeout = self.config.random_election_timeout();
        state.become_candidate(election_timeout, false)?;
        
        info!("Node {} starting election for term {}", self.config.node_id, state.current_term());
        
        // Send vote requests to all peers
        let log = self.log.read().await;
        let last_log_index = log.last_log_index();
        let last_log_term = log.last_log_term();
        drop(log);
        
        let vote_request = VoteRequest::new(
            state.current_term(),
            self.config.node_id,
            last_log_index,
            last_log_term,
            false,
        );
        
        for &peer in &self.config.peers {
            self.send_message(peer, Message::VoteRequest(vote_request.clone())).await?;
        }
        
        Ok(())
    }

    /// Send heartbeats to all followers
    async fn send_heartbeats(&mut self) -> RaftResult<()> {
        let peers_to_contact = {
            let mut state = self.state.write().await;
            
            if !state.is_leader() {
                return Ok(());
            }
            
            let leader_state = state.leader_state_mut().unwrap();
            leader_state.peers_needing_entries(self.config.heartbeat_interval)
        };
        
        for peer in peers_to_contact {
            let mut state = self.state.write().await;
            self.send_append_entries_to_peer(peer, &mut state).await?;
        }
        
        Ok(())
    }

    /// Send AppendEntries to a specific peer
    async fn send_append_entries_to_peer(
        &self,
        peer: u64,
        state: &mut RaftState,
    ) -> RaftResult<()> {
        let next_index = {
            let leader_state = state.leader_state_mut().unwrap();
            leader_state.next_index.get(&peer).copied().unwrap_or(1)
        };
        
        let log = self.log.read().await;
        let prev_log_index = if next_index > 1 { next_index - 1 } else { 0 };
        let prev_log_term = if prev_log_index > 0 {
            log.get_term(prev_log_index)?
        } else {
            0
        };
        
        let entries = if next_index <= log.last_log_index() {
            let max_entries = std::cmp::min(
                self.config.max_entries_per_request,
                (log.last_log_index() - next_index + 1) as usize,
            );
            log.get_entries_from(next_index, max_entries)?
        } else {
            vec![]
        };
        
        let current_term = state.current_term();
        let commit_index = log.commit_index();
        drop(log);
        
        let request = AppendEntriesRequest::new(
            current_term,
            self.config.node_id,
            prev_log_index,
            prev_log_term,
            entries,
            commit_index,
        );
        
        // Record pending response
        let leader_state = state.leader_state_mut().unwrap();
        leader_state.add_pending_response(peer, request.prev_log_index + request.entries.len() as LogIndex);
        leader_state.record_heartbeat(peer);
        
        self.send_message(peer, Message::AppendEntriesRequest(request)).await?;
        
        Ok(())
    }

    /// Apply committed entries to state machine
    async fn apply_committed_entries(&mut self) -> RaftResult<()> {
        let state = self.state.read().await;
        let last_applied = state.last_applied();
        let commit_index = state.commit_index();
        drop(state);
        
        if last_applied >= commit_index {
            return Ok(());
        }
        
        let log = self.log.read().await;
        let entries = log.get_entries(last_applied + 1, commit_index)?;
        drop(log);
        
        let mut state_machine = self.state_machine.write().await;
        let mut new_last_applied = last_applied;
        
        for entry in entries {
            state_machine.apply(&entry).await?;
            new_last_applied = entry.index;
        }
        
        drop(state_machine);
        
        // Update last applied
        let mut state = self.state.write().await;
        state.set_last_applied(new_last_applied);
        
        let mut log = self.log.write().await;
        log.set_last_applied(new_last_applied)?;
        
        Ok(())
    }

    /// Check if should grant vote
    async fn should_grant_vote(&self, state: &RaftState, request: &VoteRequest) -> RaftResult<bool> {
        // Don't vote for stale terms
        if request.term < state.current_term() {
            return Ok(false);
        }
        
        // Check if we can vote for this candidate
        if !state.can_vote_for(request.candidate_id) {
            return Ok(false);
        }
        
        // Check if candidate's log is at least as up-to-date as ours
        let log = self.log.read().await;
        let our_last_log_term = log.last_log_term();
        let our_last_log_index = log.last_log_index();
        
        if request.last_log_term > our_last_log_term {
            return Ok(true);
        }
        
        if request.last_log_term == our_last_log_term && request.last_log_index >= our_last_log_index {
            return Ok(true);
        }
        
        Ok(false)
    }

    /// Get reason for vote denial
    async fn vote_denial_reason(&self, state: &RaftState, request: &VoteRequest) -> RaftResult<String> {
        if request.term < state.current_term() {
            return Ok("Stale term".to_string());
        }
        
        if !state.can_vote_for(request.candidate_id) {
            return Ok("Already voted".to_string());
        }
        
        Ok("Candidate log not up-to-date".to_string())
    }

    /// Process install snapshot request
    async fn process_install_snapshot_request(
        &self,
        request: InstallSnapshotRequest,
    ) -> RaftResult<InstallSnapshotResponse> {
        let state = self.state.read().await;
        
        // For simplicity, we'll just accept the snapshot
        // In a real implementation, you'd handle chunked snapshots
        if request.done {
            let mut state_machine = self.state_machine.write().await;
            state_machine.restore_from_snapshot(&request.data).await?;
            drop(state_machine);
            
            // Update log state
            let mut log = self.log.write().await;
            log.compact(request.last_included_index)?;
            log.set_commit_index(request.last_included_index)?;
            log.set_last_applied(request.last_included_index)?;
        }
        
        Ok(InstallSnapshotResponse::success(
            request.message_id,
            state.current_term(),
            self.config.node_id,
        ))
    }

    /// Find the next index to try after a log inconsistency
    fn find_next_index(&self, _log: &Log, _prev_log_index: LogIndex, _prev_log_term: Term) -> RaftResult<LogIndex> {
        // Simple implementation: just go back one index
        // In practice, you'd implement more sophisticated optimizations
        Ok(std::cmp::max(1, 1))
    }

    /// Send a message to a peer
    async fn send_message(&self, peer: u64, message: Message) -> RaftResult<()> {
        self.message_sender.send((peer, message)).map_err(|_| {
            RaftError::network("Failed to send message")
        })?;
        
        Ok(())
    }

    /// Reset election timeout
    async fn reset_election_timeout(&mut self) -> RaftResult<()> {
        self.election_timeout = Some(self.config.random_election_timeout());
        Ok(())
    }

    /// Start heartbeat timer
    async fn start_heartbeat_timer(&mut self) -> RaftResult<()> {
        self.heartbeat_timer = Some(interval(self.config.heartbeat_interval));
        Ok(())
    }

    /// Stop heartbeat timer
    async fn stop_heartbeat_timer(&mut self) {
        self.heartbeat_timer = None;
    }
}

/// Simple in-memory state machine for testing
pub struct InMemoryStateMachine {
    data: HashMap<String, String>,
}

impl InMemoryStateMachine {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

#[async_trait]
impl StateMachine for InMemoryStateMachine {
    async fn apply(&mut self, entry: &LogEntry) -> RaftResult<Vec<u8>> {
        let command = String::from_utf8(entry.data.clone()).unwrap_or_default();
        
        // Simple key-value operations
        if command.starts_with("SET ") {
            let parts: Vec<&str> = command.splitn(3, ' ').collect();
            if parts.len() == 3 {
                self.data.insert(parts[1].to_string(), parts[2].to_string());
                return Ok(b"OK".to_vec());
            }
        } else if command.starts_with("GET ") {
            let key = &command[4..];
            if let Some(value) = self.data.get(key) {
                return Ok(value.as_bytes().to_vec());
            } else {
                return Ok(b"NOT_FOUND".to_vec());
            }
        }
        
        Ok(b"UNKNOWN_COMMAND".to_vec())
    }
    
    async fn take_snapshot(&mut self) -> RaftResult<Vec<u8>> {
        serde_json::to_vec(&self.data).map_err(|e| RaftError::internal(e.to_string()))
    }
    
    async fn restore_from_snapshot(&mut self, snapshot: &[u8]) -> RaftResult<()> {
        self.data = serde_json::from_slice(snapshot).map_err(|e| RaftError::internal(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_state_machine() {
        let mut sm = InMemoryStateMachine::new();
        
        let entry1 = LogEntry::new_command(1, 1, b"SET key1 value1".to_vec());
        let result1 = sm.apply(&entry1).await.unwrap();
        assert_eq!(result1, b"OK");
        
        let entry2 = LogEntry::new_command(1, 2, b"GET key1".to_vec());
        let result2 = sm.apply(&entry2).await.unwrap();
        assert_eq!(result2, b"value1");
        
        let entry3 = LogEntry::new_command(1, 3, b"GET key2".to_vec());
        let result3 = sm.apply(&entry3).await.unwrap();
        assert_eq!(result3, b"NOT_FOUND");
    }

    #[tokio::test]
    async fn test_snapshot() {
        let mut sm = InMemoryStateMachine::new();
        
        let entry = LogEntry::new_command(1, 1, b"SET key1 value1".to_vec());
        sm.apply(&entry).await.unwrap();
        
        let snapshot = sm.take_snapshot().await.unwrap();
        
        let mut sm2 = InMemoryStateMachine::new();
        sm2.restore_from_snapshot(&snapshot).await.unwrap();
        
        let entry2 = LogEntry::new_command(1, 2, b"GET key1".to_vec());
        let result = sm2.apply(&entry2).await.unwrap();
        assert_eq!(result, b"value1");
    }
}