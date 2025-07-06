//! Network layer for Raft communication.
//!
//! This module provides the networking infrastructure for Raft consensus,
//! including TCP connections, message serialization, and connection management.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::{Buf, BufMut, BytesMut};
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock, oneshot};
use tokio::time::{sleep, timeout};
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::{debug, error, info, warn};

use raft_core::{Message, MessageType};
use crate::config::{NetworkConfig, RetryConfig};
use crate::error::{RaftStorageError, RaftStorageResult};

/// Network message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// Source node ID
    pub from: u64,
    /// Destination node ID
    pub to: u64,
    /// Message ID for correlation
    pub message_id: u64,
    /// Message type
    pub message_type: MessageType,
    /// Serialized message payload
    pub payload: Vec<u8>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Network connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Connection is healthy
    Connected,
    /// Connection is being established
    Connecting,
    /// Connection is temporarily unavailable
    Disconnected,
    /// Connection has failed permanently
    Failed,
}

/// Network connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Connection state
    pub state: ConnectionState,
    /// Remote address
    pub remote_addr: SocketAddr,
    /// Last successful communication
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Connection latency
    pub latency: Option<Duration>,
}

/// Message codec for encoding/decoding network messages
pub struct MessageCodec;

impl Encoder<NetworkMessage> for MessageCodec {
    type Error = RaftStorageError;

    fn encode(&mut self, msg: NetworkMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = bincode::serialize(&msg)?;
        let len = encoded.len() as u32;
        
        dst.reserve(4 + encoded.len());
        dst.put_u32(len);
        dst.put_slice(&encoded);
        
        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = NetworkMessage;
    type Error = RaftStorageError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let len = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
        
        if src.len() < 4 + len {
            return Ok(None);
        }

        src.advance(4);
        let data = src.split_to(len);
        let msg: NetworkMessage = bincode::deserialize(&data)?;
        
        Ok(Some(msg))
    }
}

/// Network layer for Raft communication
pub struct RaftNetwork {
    /// This node's ID
    node_id: u64,
    
    /// Network configuration
    config: NetworkConfig,
    
    /// Peer addresses
    peer_addresses: Arc<RwLock<HashMap<u64, String>>>,
    
    /// Active connections
    connections: Arc<RwLock<HashMap<u64, ConnectionInfo>>>,
    
    /// Message sender for outgoing messages
    message_sender: mpsc::UnboundedSender<NetworkMessage>,
    
    /// Message receiver for incoming messages
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<NetworkMessage>>>>,
    
    /// Shutdown signal
    shutdown_sender: Option<oneshot::Sender<()>>,
    
    /// Connection retry state
    retry_state: Arc<RwLock<HashMap<u64, RetryState>>>,
}

/// Retry state for connection attempts
#[derive(Debug, Clone)]
struct RetryState {
    /// Number of attempts made
    attempts: usize,
    /// Next retry time
    next_retry: chrono::DateTime<chrono::Utc>,
    /// Current delay
    current_delay: Duration,
}

impl RaftNetwork {
    /// Create a new Raft network
    pub fn new(
        node_id: u64,
        config: NetworkConfig,
        peer_addresses: HashMap<u64, String>,
    ) -> Self {
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        
        Self {
            node_id,
            config,
            peer_addresses: Arc::new(RwLock::new(peer_addresses)),
            connections: Arc::new(RwLock::new(HashMap::new())),
            message_sender,
            message_receiver: Arc::new(RwLock::new(Some(message_receiver))),
            shutdown_sender: None,
            retry_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start the network layer
    pub async fn start(&mut self, bind_addr: SocketAddr) -> RaftStorageResult<()> {
        info!("Starting Raft network on node {} at {}", self.node_id, bind_addr);
        
        // Start server
        let server_handle = self.start_server(bind_addr).await?;
        
        // Start connection manager
        let connection_handle = self.start_connection_manager().await?;
        
        // Start message processor
        let message_handle = self.start_message_processor().await?;
        
        info!("Raft network started successfully on node {}", self.node_id);
        
        Ok(())
    }
    
    /// Send a message to a peer
    pub async fn send_message(&self, to: u64, message: Message) -> RaftStorageResult<()> {
        let network_msg = NetworkMessage {
            from: self.node_id,
            to,
            message_id: rand::random(),
            message_type: message.message_type(),
            payload: bincode::serialize(&message)?,
            timestamp: chrono::Utc::now(),
        };
        
        self.message_sender.send(network_msg)
            .map_err(|_| RaftStorageError::ChannelSend)?;
        
        Ok(())
    }
    
    /// Get message receiver
    pub async fn take_message_receiver(&self) -> Option<mpsc::UnboundedReceiver<NetworkMessage>> {
        self.message_receiver.write().await.take()
    }
    
    /// Get connection information for all peers
    pub async fn connection_info(&self) -> HashMap<u64, ConnectionInfo> {
        self.connections.read().await.clone()
    }
    
    /// Update peer addresses
    pub async fn update_peer_addresses(&self, addresses: HashMap<u64, String>) {
        let mut peer_addresses = self.peer_addresses.write().await;
        *peer_addresses = addresses;
    }
    
    /// Shutdown the network
    pub async fn shutdown(&mut self) -> RaftStorageResult<()> {
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
        }
        
        info!("Raft network shut down on node {}", self.node_id);
        Ok(())
    }
    
    /// Start TCP server for incoming connections
    async fn start_server(&self, bind_addr: SocketAddr) -> RaftStorageResult<tokio::task::JoinHandle<()>> {
        let listener = TcpListener::bind(bind_addr).await?;
        let node_id = self.node_id;
        let message_sender = self.message_sender.clone();
        
        let handle = tokio::spawn(async move {
            info!("Server listening on {} for node {}", bind_addr, node_id);
            
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        debug!("Accepted connection from {} on node {}", peer_addr, node_id);
                        
                        let message_sender = message_sender.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, message_sender).await {
                                error!("Error handling connection from {}: {}", peer_addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection on node {}: {}", node_id, e);
                    }
                }
            }
        });
        
        Ok(handle)
    }
    
    /// Handle incoming TCP connection
    async fn handle_connection(
        stream: TcpStream,
        message_sender: mpsc::UnboundedSender<NetworkMessage>,
    ) -> RaftStorageResult<()> {
        let mut framed = Framed::new(stream, MessageCodec);
        
        while let Some(msg) = framed.next().await {
            match msg {
                Ok(network_msg) => {
                    debug!("Received message from node {}: {:?}", network_msg.from, network_msg.message_type);
                    
                    if let Err(e) = message_sender.send(network_msg) {
                        error!("Failed to forward received message: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Error decoding message: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Start connection manager for outgoing connections
    async fn start_connection_manager(&self) -> RaftStorageResult<tokio::task::JoinHandle<()>> {
        let node_id = self.node_id;
        let peer_addresses = self.peer_addresses.clone();
        let connections = self.connections.clone();
        let retry_state = self.retry_state.clone();
        let config = self.config.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                let addresses = peer_addresses.read().await.clone();
                let mut conn_info = connections.write().await;
                let mut retry_info = retry_state.write().await;
                
                for (peer_id, address) in addresses {
                    if peer_id == node_id {
                        continue;
                    }
                    
                    let should_retry = match retry_info.get(&peer_id) {
                        Some(retry) => chrono::Utc::now() >= retry.next_retry,
                        None => true,
                    };
                    
                    if should_retry {
                        match Self::establish_connection(&address, &config).await {
                            Ok(latency) => {
                                conn_info.insert(peer_id, ConnectionInfo {
                                    state: ConnectionState::Connected,
                                    remote_addr: address.parse().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap()),
                                    last_success: Some(chrono::Utc::now()),
                                    failure_count: 0,
                                    latency: Some(latency),
                                });
                                retry_info.remove(&peer_id);
                                debug!("Established connection to node {} at {}", peer_id, address);
                            }
                            Err(e) => {
                                let retry = retry_info.entry(peer_id).or_insert(RetryState {
                                    attempts: 0,
                                    next_retry: chrono::Utc::now(),
                                    current_delay: config.retry.initial_delay,
                                });
                                
                                retry.attempts += 1;
                                retry.current_delay = std::cmp::min(
                                    Duration::from_secs_f64(retry.current_delay.as_secs_f64() * config.retry.backoff_multiplier),
                                    config.retry.max_delay,
                                );
                                retry.next_retry = chrono::Utc::now() + chrono::Duration::from_std(retry.current_delay).unwrap();
                                
                                let conn_info_entry = conn_info.entry(peer_id).or_insert(ConnectionInfo {
                                    state: ConnectionState::Disconnected,
                                    remote_addr: address.parse().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap()),
                                    last_success: None,
                                    failure_count: 0,
                                    latency: None,
                                });
                                
                                conn_info_entry.state = ConnectionState::Failed;
                                conn_info_entry.failure_count += 1;
                                
                                warn!("Failed to connect to node {} at {}: {} (attempt {})", peer_id, address, e, retry.attempts);
                            }
                        }
                    }
                }
            }
        });
        
        Ok(handle)
    }
    
    /// Start message processor
    async fn start_message_processor(&self) -> RaftStorageResult<tokio::task::JoinHandle<()>> {
        let node_id = self.node_id;
        let peer_addresses = self.peer_addresses.clone();
        let connections = self.connections.clone();
        let config = self.config.clone();
        
        let handle = tokio::spawn(async move {
            // Message processing logic would go here
            // For now, we'll just log that the processor is running
            info!("Message processor started for node {}", node_id);
            
            // In a full implementation, this would:
            // 1. Receive messages from the outgoing queue
            // 2. Find appropriate connections for each message
            // 3. Send messages over TCP connections
            // 4. Handle connection failures and retries
            
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                // Placeholder for message processing logic
            }
        });
        
        Ok(handle)
    }
    
    /// Establish connection to a peer
    async fn establish_connection(address: &str, config: &NetworkConfig) -> RaftStorageResult<Duration> {
        let start_time = std::time::Instant::now();
        
        let stream = timeout(
            config.connect_timeout,
            TcpStream::connect(address),
        ).await
        .map_err(|_| RaftStorageError::network(format!("Connection timeout to {}", address)))?
        .map_err(|e| RaftStorageError::network(format!("Failed to connect to {}: {}", address, e)))?;
        
        // Set socket options
        if let Some(keepalive) = config.tcp_keepalive {
            let socket = socket2::Socket::from(stream.into_std()?);
            socket.set_keepalive(true)?;
            socket.set_keepalive_time(keepalive)?;
            let _ = TcpStream::from_std(socket.into())?;
        }
        
        let latency = start_time.elapsed();
        Ok(latency)
    }
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Connection failures
    pub connection_failures: u64,
    /// Average latency
    pub avg_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_network_creation() {
        let mut peer_addresses = HashMap::new();
        peer_addresses.insert(2, "127.0.0.1:8081".to_string());
        peer_addresses.insert(3, "127.0.0.1:8082".to_string());
        
        let network = RaftNetwork::new(
            1,
            NetworkConfig::default(),
            peer_addresses,
        );
        
        assert_eq!(network.node_id, 1);
        assert_eq!(network.peer_addresses.read().await.len(), 2);
    }
    
    #[tokio::test]
    async fn test_message_codec() {
        let mut codec = MessageCodec;
        let mut buf = BytesMut::new();
        
        let msg = NetworkMessage {
            from: 1,
            to: 2,
            message_id: 12345,
            message_type: MessageType::AppendEntries,
            payload: b"test payload".to_vec(),
            timestamp: chrono::Utc::now(),
        };
        
        // Encode
        codec.encode(msg.clone(), &mut buf).unwrap();
        assert!(!buf.is_empty());
        
        // Decode
        let decoded = codec.decode(&mut buf).unwrap();
        assert!(decoded.is_some());
        
        let decoded_msg = decoded.unwrap();
        assert_eq!(decoded_msg.from, msg.from);
        assert_eq!(decoded_msg.to, msg.to);
        assert_eq!(decoded_msg.message_id, msg.message_id);
        assert_eq!(decoded_msg.payload, msg.payload);
    }
    
    #[test]
    fn test_retry_state_creation() {
        let retry = RetryState {
            attempts: 0,
            next_retry: chrono::Utc::now(),
            current_delay: Duration::from_millis(100),
        };
        
        assert_eq!(retry.attempts, 0);
        assert_eq!(retry.current_delay, Duration::from_millis(100));
    }
} 