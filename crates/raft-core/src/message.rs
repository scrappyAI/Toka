//! Message types for Raft RPC communication.
//!
//! This module defines all the message types used in the Raft consensus protocol,
//! including AppendEntries, RequestVote, and InstallSnapshot RPCs.

use crate::log::{LogEntry, LogIndex, Term};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for messages
pub type MessageId = Uuid;

/// Enumeration of all message types in the Raft protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// AppendEntries RPC request
    AppendEntriesRequest(AppendEntriesRequest),
    
    /// AppendEntries RPC response
    AppendEntriesResponse(AppendEntriesResponse),
    
    /// RequestVote RPC request
    VoteRequest(VoteRequest),
    
    /// RequestVote RPC response
    VoteResponse(VoteResponse),
    
    /// InstallSnapshot RPC request
    InstallSnapshotRequest(InstallSnapshotRequest),
    
    /// InstallSnapshot RPC response
    InstallSnapshotResponse(InstallSnapshotResponse),
    
    /// Client command request
    ClientRequest(ClientRequest),
    
    /// Client command response
    ClientResponse(ClientResponse),
    
    /// Heartbeat message (empty AppendEntries)
    Heartbeat(AppendEntriesRequest),
}

/// Type of message for routing and processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    AppendEntriesRequest,
    AppendEntriesResponse,
    VoteRequest,
    VoteResponse,
    InstallSnapshotRequest,
    InstallSnapshotResponse,
    ClientRequest,
    ClientResponse,
    Heartbeat,
}

impl Message {
    /// Get the message type
    pub fn message_type(&self) -> MessageType {
        match self {
            Message::AppendEntriesRequest(_) => MessageType::AppendEntriesRequest,
            Message::AppendEntriesResponse(_) => MessageType::AppendEntriesResponse,
            Message::VoteRequest(_) => MessageType::VoteRequest,
            Message::VoteResponse(_) => MessageType::VoteResponse,
            Message::InstallSnapshotRequest(_) => MessageType::InstallSnapshotRequest,
            Message::InstallSnapshotResponse(_) => MessageType::InstallSnapshotResponse,
            Message::ClientRequest(_) => MessageType::ClientRequest,
            Message::ClientResponse(_) => MessageType::ClientResponse,
            Message::Heartbeat(_) => MessageType::Heartbeat,
        }
    }

    /// Get the term from the message (if applicable)
    pub fn term(&self) -> Option<Term> {
        match self {
            Message::AppendEntriesRequest(req) => Some(req.term),
            Message::AppendEntriesResponse(resp) => Some(resp.term),
            Message::VoteRequest(req) => Some(req.term),
            Message::VoteResponse(resp) => Some(resp.term),
            Message::InstallSnapshotRequest(req) => Some(req.term),
            Message::InstallSnapshotResponse(resp) => Some(resp.term),
            Message::Heartbeat(req) => Some(req.term),
            _ => None,
        }
    }

    /// Get the sender node ID (if applicable)
    pub fn sender(&self) -> Option<u64> {
        match self {
            Message::AppendEntriesRequest(req) => Some(req.leader_id),
            Message::VoteRequest(req) => Some(req.candidate_id),
            Message::InstallSnapshotRequest(req) => Some(req.leader_id),
            Message::Heartbeat(req) => Some(req.leader_id),
            _ => None,
        }
    }
}

/// AppendEntries RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesRequest {
    /// Message ID for tracking
    pub message_id: MessageId,
    
    /// Leader's term
    pub term: Term,
    
    /// Leader's node ID
    pub leader_id: u64,
    
    /// Index of log entry immediately preceding new ones
    pub prev_log_index: LogIndex,
    
    /// Term of prev_log_index entry
    pub prev_log_term: Term,
    
    /// Log entries to store (empty for heartbeat)
    pub entries: Vec<LogEntry>,
    
    /// Leader's commit index
    pub leader_commit: LogIndex,
}

impl AppendEntriesRequest {
    /// Create a new AppendEntries request
    pub fn new(
        term: Term,
        leader_id: u64,
        prev_log_index: LogIndex,
        prev_log_term: Term,
        entries: Vec<LogEntry>,
        leader_commit: LogIndex,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            term,
            leader_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        }
    }

    /// Create a heartbeat message (empty AppendEntries)
    pub fn heartbeat(term: Term, leader_id: u64, leader_commit: LogIndex) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            term,
            leader_id,
            prev_log_index: 0,
            prev_log_term: 0,
            entries: vec![],
            leader_commit,
        }
    }

    /// Check if this is a heartbeat message
    pub fn is_heartbeat(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of entries
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// AppendEntries RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    /// Message ID for tracking (matches request)
    pub message_id: MessageId,
    
    /// Current term, for leader to update itself
    pub term: Term,
    
    /// Responding node's ID
    pub node_id: u64,
    
    /// True if follower contained entry matching prev_log_index and prev_log_term
    pub success: bool,
    
    /// Hint for leader to find the correct next index
    pub next_index: Option<LogIndex>,
    
    /// Additional error information
    pub error: Option<String>,
}

impl AppendEntriesResponse {
    /// Create a successful response
    pub fn success(message_id: MessageId, term: Term, node_id: u64) -> Self {
        Self {
            message_id,
            term,
            node_id,
            success: true,
            next_index: None,
            error: None,
        }
    }

    /// Create a failure response
    pub fn failure(
        message_id: MessageId,
        term: Term,
        node_id: u64,
        next_index: Option<LogIndex>,
        error: Option<String>,
    ) -> Self {
        Self {
            message_id,
            term,
            node_id,
            success: false,
            next_index,
            error,
        }
    }
}

/// RequestVote RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRequest {
    /// Message ID for tracking
    pub message_id: MessageId,
    
    /// Candidate's term
    pub term: Term,
    
    /// Candidate requesting vote
    pub candidate_id: u64,
    
    /// Index of candidate's last log entry
    pub last_log_index: LogIndex,
    
    /// Term of candidate's last log entry
    pub last_log_term: Term,
    
    /// True if this is a pre-vote request
    pub pre_vote: bool,
}

impl VoteRequest {
    /// Create a new vote request
    pub fn new(
        term: Term,
        candidate_id: u64,
        last_log_index: LogIndex,
        last_log_term: Term,
        pre_vote: bool,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            term,
            candidate_id,
            last_log_index,
            last_log_term,
            pre_vote,
        }
    }
}

/// RequestVote RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResponse {
    /// Message ID for tracking (matches request)
    pub message_id: MessageId,
    
    /// Current term, for candidate to update itself
    pub term: Term,
    
    /// Responding node's ID
    pub node_id: u64,
    
    /// True means candidate received vote
    pub vote_granted: bool,
    
    /// Reason for vote denial (if applicable)
    pub reason: Option<String>,
}

impl VoteResponse {
    /// Create a vote granted response
    pub fn granted(message_id: MessageId, term: Term, node_id: u64) -> Self {
        Self {
            message_id,
            term,
            node_id,
            vote_granted: true,
            reason: None,
        }
    }

    /// Create a vote denied response
    pub fn denied(message_id: MessageId, term: Term, node_id: u64, reason: String) -> Self {
        Self {
            message_id,
            term,
            node_id,
            vote_granted: false,
            reason: Some(reason),
        }
    }
}

/// InstallSnapshot RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSnapshotRequest {
    /// Message ID for tracking
    pub message_id: MessageId,
    
    /// Leader's term
    pub term: Term,
    
    /// Leader's node ID
    pub leader_id: u64,
    
    /// The snapshot replaces all entries up through and including this index
    pub last_included_index: LogIndex,
    
    /// Term of last_included_index
    pub last_included_term: Term,
    
    /// Byte offset where chunk is positioned in the snapshot file
    pub offset: u64,
    
    /// Raw bytes of the snapshot chunk, starting at offset
    pub data: Vec<u8>,
    
    /// True if this is the last chunk
    pub done: bool,
}

impl InstallSnapshotRequest {
    /// Create a new install snapshot request
    pub fn new(
        term: Term,
        leader_id: u64,
        last_included_index: LogIndex,
        last_included_term: Term,
        offset: u64,
        data: Vec<u8>,
        done: bool,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            term,
            leader_id,
            last_included_index,
            last_included_term,
            offset,
            data,
            done,
        }
    }
}

/// InstallSnapshot RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSnapshotResponse {
    /// Message ID for tracking (matches request)
    pub message_id: MessageId,
    
    /// Current term, for leader to update itself
    pub term: Term,
    
    /// Responding node's ID
    pub node_id: u64,
    
    /// True if snapshot chunk was successfully received
    pub success: bool,
    
    /// Error message if failed
    pub error: Option<String>,
}

impl InstallSnapshotResponse {
    /// Create a successful response
    pub fn success(message_id: MessageId, term: Term, node_id: u64) -> Self {
        Self {
            message_id,
            term,
            node_id,
            success: true,
            error: None,
        }
    }

    /// Create a failure response
    pub fn failure(message_id: MessageId, term: Term, node_id: u64, error: String) -> Self {
        Self {
            message_id,
            term,
            node_id,
            success: false,
            error: Some(error),
        }
    }
}

/// Client request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRequest {
    /// Message ID for tracking
    pub message_id: MessageId,
    
    /// Client identifier
    pub client_id: String,
    
    /// Request sequence number
    pub sequence: u64,
    
    /// Command data
    pub command: Vec<u8>,
    
    /// Whether this is a read-only request
    pub read_only: bool,
}

impl ClientRequest {
    /// Create a new client request
    pub fn new(client_id: String, sequence: u64, command: Vec<u8>, read_only: bool) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            client_id,
            sequence,
            command,
            read_only,
        }
    }
}

/// Client response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientResponse {
    /// Message ID for tracking (matches request)
    pub message_id: MessageId,
    
    /// Whether the request was successful
    pub success: bool,
    
    /// Response data (if successful)
    pub data: Option<Vec<u8>>,
    
    /// Error message (if failed)
    pub error: Option<String>,
    
    /// Current leader hint (if not leader)
    pub leader_hint: Option<u64>,
}

impl ClientResponse {
    /// Create a successful response
    pub fn success(message_id: MessageId, data: Option<Vec<u8>>) -> Self {
        Self {
            message_id,
            success: true,
            data,
            error: None,
            leader_hint: None,
        }
    }

    /// Create a failure response
    pub fn failure(message_id: MessageId, error: String, leader_hint: Option<u64>) -> Self {
        Self {
            message_id,
            success: false,
            data: None,
            error: Some(error),
            leader_hint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::LogEntry;

    #[test]
    fn test_append_entries_request() {
        let entries = vec![
            LogEntry::new_command(1, 1, b"cmd1".to_vec()),
            LogEntry::new_command(1, 2, b"cmd2".to_vec()),
        ];
        
        let req = AppendEntriesRequest::new(1, 1, 0, 0, entries, 0);
        assert_eq!(req.term, 1);
        assert_eq!(req.leader_id, 1);
        assert_eq!(req.entry_count(), 2);
        assert!(!req.is_heartbeat());
    }

    #[test]
    fn test_heartbeat_message() {
        let heartbeat = AppendEntriesRequest::heartbeat(1, 1, 0);
        assert!(heartbeat.is_heartbeat());
        assert_eq!(heartbeat.entry_count(), 0);
    }

    #[test]
    fn test_append_entries_response() {
        let msg_id = Uuid::new_v4();
        let success_resp = AppendEntriesResponse::success(msg_id, 1, 2);
        assert!(success_resp.success);
        assert_eq!(success_resp.term, 1);
        assert_eq!(success_resp.node_id, 2);
        
        let failure_resp = AppendEntriesResponse::failure(
            msg_id,
            1,
            2,
            Some(5),
            Some("Log inconsistency".to_string()),
        );
        assert!(!failure_resp.success);
        assert_eq!(failure_resp.next_index, Some(5));
        assert!(failure_resp.error.is_some());
    }

    #[test]
    fn test_vote_request() {
        let req = VoteRequest::new(2, 1, 5, 1, false);
        assert_eq!(req.term, 2);
        assert_eq!(req.candidate_id, 1);
        assert_eq!(req.last_log_index, 5);
        assert_eq!(req.last_log_term, 1);
        assert!(!req.pre_vote);
    }

    #[test]
    fn test_vote_response() {
        let msg_id = Uuid::new_v4();
        let granted = VoteResponse::granted(msg_id, 2, 2);
        assert!(granted.vote_granted);
        assert_eq!(granted.term, 2);
        assert_eq!(granted.node_id, 2);
        
        let denied = VoteResponse::denied(msg_id, 2, 2, "Already voted".to_string());
        assert!(!denied.vote_granted);
        assert_eq!(denied.reason, Some("Already voted".to_string()));
    }

    #[test]
    fn test_client_request() {
        let req = ClientRequest::new("client1".to_string(), 1, b"command".to_vec(), false);
        assert_eq!(req.client_id, "client1");
        assert_eq!(req.sequence, 1);
        assert_eq!(req.command, b"command");
        assert!(!req.read_only);
    }

    #[test]
    fn test_client_response() {
        let msg_id = Uuid::new_v4();
        let success = ClientResponse::success(msg_id, Some(b"result".to_vec()));
        assert!(success.success);
        assert_eq!(success.data, Some(b"result".to_vec()));
        
        let failure = ClientResponse::failure(msg_id, "Not leader".to_string(), Some(2));
        assert!(!failure.success);
        assert_eq!(failure.error, Some("Not leader".to_string()));
        assert_eq!(failure.leader_hint, Some(2));
    }

    #[test]
    fn test_message_type_detection() {
        let req = AppendEntriesRequest::new(1, 1, 0, 0, vec![], 0);
        let msg = Message::AppendEntriesRequest(req);
        assert_eq!(msg.message_type(), MessageType::AppendEntriesRequest);
        assert_eq!(msg.term(), Some(1));
        assert_eq!(msg.sender(), Some(1));
    }
}