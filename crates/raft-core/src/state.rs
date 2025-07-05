//! State management for Raft nodes.
//!
//! This module implements the three states of a Raft node: Follower, Candidate, and Leader.
//! It manages state transitions, persistent state, and volatile state as defined in the Raft paper.

use crate::error::{RaftError, RaftResult};
use crate::log::{LogIndex, Term};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// The current state of a Raft node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    /// Follower state - accepts AppendEntries and RequestVote
    Follower,
    
    /// Candidate state - requests votes from other nodes
    Candidate,
    
    /// Leader state - sends AppendEntries to other nodes
    Leader,
}

impl std::fmt::Display for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeState::Follower => write!(f, "Follower"),
            NodeState::Candidate => write!(f, "Candidate"),
            NodeState::Leader => write!(f, "Leader"),
        }
    }
}

/// Persistent state that must be stored on stable storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    /// Latest term server has seen (initialized to 0 on first boot)
    pub current_term: Term,
    
    /// Candidate ID that received vote in current term (or None if none)
    pub voted_for: Option<u64>,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
        }
    }
}

/// Volatile state for all servers
#[derive(Debug, Clone)]
pub struct VolatileState {
    /// Index of highest log entry known to be committed
    pub commit_index: LogIndex,
    
    /// Index of highest log entry applied to state machine
    pub last_applied: LogIndex,
}

impl Default for VolatileState {
    fn default() -> Self {
        Self {
            commit_index: 0,
            last_applied: 0,
        }
    }
}

/// Volatile state for leaders (reinitialized after election)
#[derive(Debug, Clone)]
pub struct LeaderState {
    /// For each server, index of the next log entry to send to that server
    pub next_index: HashMap<u64, LogIndex>,
    
    /// For each server, index of highest log entry known to be replicated on server
    pub match_index: HashMap<u64, LogIndex>,
    
    /// Last time heartbeat was sent to each peer
    pub last_heartbeat: HashMap<u64, Instant>,
    
    /// Responses received for current AppendEntries requests
    pub pending_responses: HashMap<u64, PendingResponse>,
}

/// Information about a pending AppendEntries response
#[derive(Debug, Clone)]
pub struct PendingResponse {
    /// The index of the last entry sent in the request
    pub last_entry_index: LogIndex,
    
    /// When the request was sent
    pub sent_at: Instant,
    
    /// Number of retries for this request
    pub retries: u32,
}

impl LeaderState {
    /// Create new leader state for the given peers and last log index
    pub fn new(peers: &[u64], last_log_index: LogIndex) -> Self {
        let mut next_index = HashMap::new();
        let mut match_index = HashMap::new();
        let mut last_heartbeat = HashMap::new();
        
        for &peer in peers {
            next_index.insert(peer, last_log_index + 1);
            match_index.insert(peer, 0);
            last_heartbeat.insert(peer, Instant::now());
        }
        
        Self {
            next_index,
            match_index,
            last_heartbeat,
            pending_responses: HashMap::new(),
        }
    }

    /// Update the next index for a peer
    pub fn update_next_index(&mut self, peer: u64, index: LogIndex) {
        self.next_index.insert(peer, index);
    }

    /// Update the match index for a peer
    pub fn update_match_index(&mut self, peer: u64, index: LogIndex) {
        self.match_index.insert(peer, index);
        // When match index is updated, next index should be at least match + 1
        let next = self.next_index.get(&peer).copied().unwrap_or(0);
        if next <= index {
            self.next_index.insert(peer, index + 1);
        }
    }

    /// Record that a heartbeat was sent to a peer
    pub fn record_heartbeat(&mut self, peer: u64) {
        self.last_heartbeat.insert(peer, Instant::now());
    }

    /// Check if a heartbeat is needed for a peer
    pub fn needs_heartbeat(&self, peer: u64, heartbeat_interval: Duration) -> bool {
        match self.last_heartbeat.get(&peer) {
            Some(last) => last.elapsed() >= heartbeat_interval,
            None => true,
        }
    }

    /// Add a pending response
    pub fn add_pending_response(&mut self, peer: u64, last_entry_index: LogIndex) {
        self.pending_responses.insert(peer, PendingResponse {
            last_entry_index,
            sent_at: Instant::now(),
            retries: 0,
        });
    }

    /// Remove a pending response
    pub fn remove_pending_response(&mut self, peer: u64) -> Option<PendingResponse> {
        self.pending_responses.remove(&peer)
    }

    /// Get peers that need to be contacted
    pub fn peers_needing_entries(&self, heartbeat_interval: Duration) -> Vec<u64> {
        self.next_index.keys()
            .filter(|&&peer| {
                self.needs_heartbeat(peer, heartbeat_interval) || 
                !self.pending_responses.contains_key(&peer)
            })
            .copied()
            .collect()
    }

    /// Calculate the commit index based on match indices
    pub fn calculate_commit_index(&self, current_commit: LogIndex, majority: usize) -> LogIndex {
        if self.match_index.is_empty() {
            return current_commit;
        }

        let mut indices: Vec<LogIndex> = self.match_index.values().copied().collect();
        indices.sort_by(|a, b| b.cmp(a)); // Sort in descending order

        // Find the highest index that is replicated on a majority of servers
        // For a cluster of N nodes, we need majority = (N/2) + 1 nodes to agree
        // Since the leader always has the latest entries, we need (majority - 1) followers
        if indices.len() >= majority {
            let majority_index = indices[majority - 1]; // majority-1 because 0-indexed
            std::cmp::max(current_commit, majority_index)
        } else {
            current_commit
        }
    }
}

/// Volatile state for candidates
#[derive(Debug, Clone)]
pub struct CandidateState {
    /// Votes received from other nodes
    pub votes_received: HashSet<u64>,
    
    /// When the election started
    pub election_start: Instant,
    
    /// Election timeout for this election
    pub election_timeout: Duration,
    
    /// Whether this is a pre-vote election
    pub pre_vote: bool,
}

impl CandidateState {
    /// Create new candidate state
    pub fn new(election_timeout: Duration, pre_vote: bool) -> Self {
        Self {
            votes_received: HashSet::new(),
            election_start: Instant::now(),
            election_timeout,
            pre_vote,
        }
    }

    /// Add a vote from a node
    pub fn add_vote(&mut self, node_id: u64) {
        self.votes_received.insert(node_id);
    }

    /// Check if election has timed out
    pub fn has_timed_out(&self) -> bool {
        self.election_start.elapsed() >= self.election_timeout
    }

    /// Check if majority of votes received
    pub fn has_majority(&self, cluster_size: usize) -> bool {
        let votes_needed = (cluster_size / 2) + 1;
        self.votes_received.len() + 1 >= votes_needed // +1 for self-vote
    }

    /// Get the number of votes received
    pub fn vote_count(&self) -> usize {
        self.votes_received.len() + 1 // +1 for self-vote
    }
}

/// Volatile state for followers
#[derive(Debug, Clone)]
pub struct FollowerState {
    /// Current known leader (if any)
    pub leader_id: Option<u64>,
    
    /// Last time we received a message from the leader
    pub last_leader_contact: Option<Instant>,
    
    /// Election timeout for this follower
    pub election_timeout: Duration,
}

impl FollowerState {
    /// Create new follower state
    pub fn new(election_timeout: Duration) -> Self {
        Self {
            leader_id: None,
            last_leader_contact: None,
            election_timeout,
        }
    }

    /// Update leader information
    pub fn update_leader(&mut self, leader_id: u64) {
        self.leader_id = Some(leader_id);
        self.last_leader_contact = Some(Instant::now());
    }

    /// Record contact from leader
    pub fn record_leader_contact(&mut self) {
        self.last_leader_contact = Some(Instant::now());
    }

    /// Check if election timeout has occurred
    pub fn has_election_timeout(&self) -> bool {
        match self.last_leader_contact {
            Some(last) => last.elapsed() >= self.election_timeout,
            None => true,
        }
    }

    /// Get time since last leader contact
    pub fn time_since_leader_contact(&self) -> Option<Duration> {
        self.last_leader_contact.map(|last| last.elapsed())
    }
}

/// Complete state of a Raft node
#[derive(Debug, Clone)]
pub struct RaftState {
    /// Node ID
    pub node_id: u64,
    
    /// Current node state
    pub state: NodeState,
    
    /// Persistent state
    pub persistent: PersistentState,
    
    /// Volatile state
    pub volatile: VolatileState,
    
    /// Leader-specific state (only valid when state == Leader)
    pub leader_state: Option<LeaderState>,
    
    /// Candidate-specific state (only valid when state == Candidate)
    pub candidate_state: Option<CandidateState>,
    
    /// Follower-specific state (only valid when state == Follower)
    pub follower_state: Option<FollowerState>,
    
    /// When the node started
    pub start_time: DateTime<Utc>,
    
    /// Last state transition time
    pub last_state_change: Instant,
}

impl RaftState {
    /// Create new Raft state for a node
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            state: NodeState::Follower,
            persistent: PersistentState::default(),
            volatile: VolatileState::default(),
            leader_state: None,
            candidate_state: None,
            follower_state: None,
            start_time: Utc::now(),
            last_state_change: Instant::now(),
        }
    }

    /// Transition to follower state
    pub fn become_follower(&mut self, term: Term, election_timeout: Duration) {
        self.state = NodeState::Follower;
        self.persistent.current_term = term;
        self.persistent.voted_for = None;
        self.leader_state = None;
        self.candidate_state = None;
        self.follower_state = Some(FollowerState::new(election_timeout));
        self.last_state_change = Instant::now();
    }

    /// Transition to candidate state
    pub fn become_candidate(&mut self, election_timeout: Duration, pre_vote: bool) -> RaftResult<()> {
        if !pre_vote {
            self.persistent.current_term += 1;
            self.persistent.voted_for = Some(self.node_id);
        }
        
        self.state = NodeState::Candidate;
        self.leader_state = None;
        self.candidate_state = Some(CandidateState::new(election_timeout, pre_vote));
        self.follower_state = None;
        self.last_state_change = Instant::now();
        
        Ok(())
    }

    /// Transition to leader state
    pub fn become_leader(&mut self, peers: &[u64], last_log_index: LogIndex) -> RaftResult<()> {
        if self.state != NodeState::Candidate {
            return Err(RaftError::InvalidState {
                node_id: self.node_id,
                current_state: self.state.to_string(),
                expected_state: "Candidate".to_string(),
            });
        }

        self.state = NodeState::Leader;
        self.leader_state = Some(LeaderState::new(peers, last_log_index));
        self.candidate_state = None;
        self.follower_state = None;
        self.last_state_change = Instant::now();
        
        Ok(())
    }

    /// Update the current term
    pub fn update_term(&mut self, term: Term) -> RaftResult<()> {
        if term < self.persistent.current_term {
            return Err(RaftError::InvalidTerm {
                received: term,
                current: self.persistent.current_term,
            });
        }

        if term > self.persistent.current_term {
            self.persistent.current_term = term;
            self.persistent.voted_for = None;
        }

        Ok(())
    }

    /// Vote for a candidate
    pub fn vote_for(&mut self, candidate_id: u64) -> RaftResult<()> {
        if self.persistent.voted_for.is_some() {
            return Err(RaftError::internal("Already voted in this term"));
        }

        self.persistent.voted_for = Some(candidate_id);
        Ok(())
    }

    /// Check if can vote for candidate
    pub fn can_vote_for(&self, candidate_id: u64) -> bool {
        match self.persistent.voted_for {
            Some(voted_for) => voted_for == candidate_id,
            None => true,
        }
    }

    /// Get current term
    pub fn current_term(&self) -> Term {
        self.persistent.current_term
    }

    /// Get voted for
    pub fn voted_for(&self) -> Option<u64> {
        self.persistent.voted_for
    }

    /// Get commit index
    pub fn commit_index(&self) -> LogIndex {
        self.volatile.commit_index
    }

    /// Set commit index
    pub fn set_commit_index(&mut self, index: LogIndex) {
        self.volatile.commit_index = index;
    }

    /// Get last applied index
    pub fn last_applied(&self) -> LogIndex {
        self.volatile.last_applied
    }

    /// Set last applied index
    pub fn set_last_applied(&mut self, index: LogIndex) {
        self.volatile.last_applied = index;
    }

    /// Check if node is leader
    pub fn is_leader(&self) -> bool {
        self.state == NodeState::Leader
    }

    /// Check if node is candidate
    pub fn is_candidate(&self) -> bool {
        self.state == NodeState::Candidate
    }

    /// Check if node is follower
    pub fn is_follower(&self) -> bool {
        self.state == NodeState::Follower
    }

    /// Get time since last state change
    pub fn time_in_current_state(&self) -> Duration {
        self.last_state_change.elapsed()
    }

    /// Get leader state (if leader)
    pub fn leader_state(&self) -> Option<&LeaderState> {
        self.leader_state.as_ref()
    }

    /// Get mutable leader state (if leader)
    pub fn leader_state_mut(&mut self) -> Option<&mut LeaderState> {
        self.leader_state.as_mut()
    }

    /// Get candidate state (if candidate)
    pub fn candidate_state(&self) -> Option<&CandidateState> {
        self.candidate_state.as_ref()
    }

    /// Get mutable candidate state (if candidate)
    pub fn candidate_state_mut(&mut self) -> Option<&mut CandidateState> {
        self.candidate_state.as_mut()
    }

    /// Get follower state (if follower)
    pub fn follower_state(&self) -> Option<&FollowerState> {
        self.follower_state.as_ref()
    }

    /// Get mutable follower state (if follower)
    pub fn follower_state_mut(&mut self) -> Option<&mut FollowerState> {
        self.follower_state.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_node_state_transitions() {
        let mut state = RaftState::new(1);
        assert_eq!(state.state, NodeState::Follower);

        // Become candidate
        state.become_candidate(Duration::from_millis(150), false).unwrap();
        assert_eq!(state.state, NodeState::Candidate);
        assert_eq!(state.current_term(), 1);
        assert_eq!(state.voted_for(), Some(1));

        // Become leader
        state.become_leader(&[2, 3], 0).unwrap();
        assert_eq!(state.state, NodeState::Leader);
        assert!(state.leader_state().is_some());

        // Become follower
        state.become_follower(2, Duration::from_millis(150));
        assert_eq!(state.state, NodeState::Follower);
        assert_eq!(state.current_term(), 2);
        assert_eq!(state.voted_for(), None);
    }

    #[test]
    fn test_leader_state() {
        let mut leader_state = LeaderState::new(&[2, 3], 5);
        
        assert_eq!(leader_state.next_index.get(&2), Some(&6));
        assert_eq!(leader_state.match_index.get(&2), Some(&0));
        
        leader_state.update_match_index(2, 3);
        assert_eq!(leader_state.match_index.get(&2), Some(&3));
        assert_eq!(leader_state.next_index.get(&2), Some(&6)); // Should be at least 4
        
        leader_state.update_next_index(2, 4);
        assert_eq!(leader_state.next_index.get(&2), Some(&4));
    }

    #[test]
    fn test_candidate_state() {
        let mut candidate_state = CandidateState::new(Duration::from_millis(150), false);
        
        assert_eq!(candidate_state.vote_count(), 1); // Self-vote
        assert!(!candidate_state.has_majority(5)); // Need 3 votes for 5-node cluster
        
        candidate_state.add_vote(2);
        candidate_state.add_vote(3);
        assert_eq!(candidate_state.vote_count(), 3);
        assert!(candidate_state.has_majority(5));
    }

    #[test]
    fn test_follower_state() {
        let mut follower_state = FollowerState::new(Duration::from_millis(150));
        
        assert!(follower_state.has_election_timeout()); // No leader contact yet
        
        follower_state.update_leader(2);
        assert_eq!(follower_state.leader_id, Some(2));
        assert!(!follower_state.has_election_timeout()); // Just contacted
    }

    #[test]
    fn test_term_updates() {
        let mut state = RaftState::new(1);
        
        // Can update to higher term
        state.update_term(5).unwrap();
        assert_eq!(state.current_term(), 5);
        assert_eq!(state.voted_for(), None);
        
        // Cannot update to lower term
        assert!(state.update_term(3).is_err());
    }

    #[test]
    fn test_voting() {
        let mut state = RaftState::new(1);
        
        assert!(state.can_vote_for(2));
        state.vote_for(2).unwrap();
        assert_eq!(state.voted_for(), Some(2));
        
        // Can't vote for someone else
        assert!(!state.can_vote_for(3));
        assert!(state.vote_for(3).is_err());
        
        // Can vote for same candidate
        assert!(state.can_vote_for(2));
    }

    #[test]
    fn test_commit_index_calculation() {
        let mut leader_state = LeaderState::new(&[2, 3, 4, 5], 10);
        
        // Update match indices
        leader_state.update_match_index(2, 8);
        leader_state.update_match_index(3, 9);
        leader_state.update_match_index(4, 7);
        leader_state.update_match_index(5, 6);
        
        // With 5 nodes, need 3 for majority
        // Sorted match indices: [9, 8, 7, 6]
        // Majority (3rd highest) is 7
        let commit_index = leader_state.calculate_commit_index(0, 3);
        assert_eq!(commit_index, 7);
    }
}