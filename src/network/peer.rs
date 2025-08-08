use crate::message::{Message, Peer, ChatEvent};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct PeerManager {
    listener: TcpListener,
    connections: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
    event_sender: mpsc::UnboundedSender<ChatEvent>,
    username: String,
    our_peer_id: Uuid,
}

struct PeerConnection {
    peer: Peer,
    writer: Arc<RwLock<tokio::io::WriteHalf<TcpStream>>>,
}

impl PeerManager {
    pub async fn new(
        port: u16,
        event_sender: mpsc::UnboundedSender<ChatEvent>,
        username: String,
        our_peer_id: Uuid,
    ) -> Result<Self> {
        // Try the specified port first, then any available port
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = match TcpListener::bind(&addr).await {
            Ok(listener) => {
                info!("Peer manager listening on {}", addr);
                listener
            }
            Err(_) => {
                // Port is in use, try any available port
                let addr = SocketAddr::from(([0, 0, 0, 0], 0));
                let listener = TcpListener::bind(&addr)
                    .await
                    .context("Failed to bind TCP listener to any port")?;
                
                let actual_addr = listener.local_addr()?;
                info!("Peer manager listening on {} (dynamic port)", actual_addr);
                listener
            }
        };
        
        Ok(Self {
            listener,
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            username,
            our_peer_id,
        })
    }

    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("Starting peer manager...");
        
        let connections = self.connections.clone();
        let event_sender = self.event_sender.clone();
        let username = self.username.clone();
        let our_peer_id = self.our_peer_id;
        
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New peer connection from {}", addr);
                    
                    let connections_clone = connections.clone();
                    let event_sender_clone = event_sender.clone();
                    let username_clone = username.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_incoming_connection(
                            stream, 
                            addr, 
                            connections_clone,
                            event_sender_clone,
                            username_clone,
                            our_peer_id
                        ).await {
                            error!("Error handling peer connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept peer connection: {}", e);
                }
            }
        }
    }

    async fn handle_incoming_connection(
        stream: TcpStream,
        addr: SocketAddr,
        connections: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
        event_sender: mpsc::UnboundedSender<ChatEvent>,
        our_username: String,
        our_peer_id: Uuid,
    ) -> Result<()> {
        let (reader, writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);
        let writer = Arc::new(RwLock::new(writer));
        
        let mut line = String::new();
        let mut peer_info: Option<Peer> = None;
        
        loop {
            line.clear();
            
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("Peer {} disconnected", addr);
                    break;
                }
                Ok(_) => {
                    let line = line.trim();
                    
                    if !line.is_empty() {
                        match serde_json::from_str::<Message>(line) {
                            Ok(message) => {
                                debug!("Received message from {}: {:?}", addr, message);
                                
                                match &message {
                                    Message::UserJoin { username, peer_id, .. } => {
                                        // Create peer info for this connection
                                        let peer = Peer {
                                            id: *peer_id,
                                            username: username.clone(),
                                            ip: addr.ip(),
                                            port: addr.port(),
                                            last_seen: chrono::Utc::now(),
                                        };
                                        
                                        // Store connection
                                        let connection = PeerConnection {
                                            peer: peer.clone(),
                                            writer: writer.clone(),
                                        };
                                        
                                        connections.write().await.insert(*peer_id, connection);
                                        peer_info = Some(peer.clone());
                                        
                                        // Send join event
                                        let event = ChatEvent::new(peer, message);
                                        if let Err(e) = event_sender.send(event) {
                                            error!("Failed to send user join event: {}", e);
                                        }
                                        
                                        // Send our own join message back
                                        let our_join = Message::user_join(our_username.clone(), our_peer_id);
                                        Self::send_message_to_writer(&writer, &our_join).await?;
                                    }
                                    Message::ChatMessage { .. } => {
                                        if let Some(ref peer) = peer_info {
                                            let event = ChatEvent::new(peer.clone(), message);
                                            if let Err(e) = event_sender.send(event) {
                                                error!("Failed to send chat message event: {}", e);
                                            }
                                        }
                                    }
                                    _ => {
                                        // Handle other message types
                                        if let Some(ref peer) = peer_info {
                                            let event = ChatEvent::new(peer.clone(), message);
                                            if let Err(e) = event_sender.send(event) {
                                                error!("Failed to send event: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse message from {}: {}", addr, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from peer {}: {}", addr, e);
                    break;
                }
            }
        }
        
        // Clean up connection when peer disconnects
        if let Some(peer) = peer_info {
            connections.write().await.remove(&peer.id);
            info!("Removed peer {} from connections", peer.username);
        }
        
        Ok(())
    }
    
    async fn send_message_to_writer(
        writer: &Arc<RwLock<tokio::io::WriteHalf<TcpStream>>>,
        message: &Message,
    ) -> Result<()> {
        let data = serde_json::to_string(message)
            .context("Failed to serialize message")?;
        
        let mut writer_guard = writer.write().await;
        writer_guard.write_all(data.as_bytes()).await
            .context("Failed to write message to peer")?;
        writer_guard.write_all(b"\n").await
            .context("Failed to write newline to peer")?;
        writer_guard.flush().await
            .context("Failed to flush message to peer")?;
        
        debug!("Sent message: {:?}", message);
        Ok(())
    }

    pub async fn send_message_to_peer(&self, peer_id: &Uuid, message: &Message) -> Result<()> {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(peer_id) {
            Self::send_message_to_writer(&connection.writer, message).await?;
            debug!("Sent message to peer {}: {:?}", peer_id, message);
        } else {
            warn!("Peer {} not found in connections", peer_id);
        }
        
        Ok(())
    }
    
    pub async fn broadcast_message(&self, message: &Message) -> Result<()> {
        let connections = self.connections.read().await;
        for (peer_id, connection) in connections.iter() {
            if let Err(e) = Self::send_message_to_writer(&connection.writer, message).await {
                error!("Failed to send message to peer {}: {}", peer_id, e);
            }
        }
        Ok(())
    }

    pub async fn connect_to_peer(&self, peer: &Peer) -> Result<()> {
        let addr = SocketAddr::new(peer.ip, peer.port);
        
        // Check if already connected
        {
            let connections = self.connections.read().await;
            if connections.contains_key(&peer.id) {
                debug!("Already connected to peer {}", peer.username);
                return Ok(());
            }
        }
        
        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                info!("Connected to peer {} at {}", peer.username, addr);
                
                let (reader, writer) = tokio::io::split(stream);
                let writer = Arc::new(RwLock::new(writer));
                
                let connection = PeerConnection {
                    peer: peer.clone(),
                    writer: writer.clone(),
                };
                
                // Store the connection
                self.connections.write().await.insert(peer.id, connection);
                
                // Send user join message to establish the connection
                let join_message = Message::user_join(self.username.clone(), self.our_peer_id);
                Self::send_message_to_writer(&writer, &join_message).await?;
                
                // Start handling messages from this peer
                let connections_clone = self.connections.clone();
                let event_sender_clone = self.event_sender.clone();
                let peer_clone = peer.clone();
                
                tokio::spawn(async move {
                    let mut reader = BufReader::new(reader);
                    let mut line = String::new();
                    
                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) => {
                                debug!("Peer {} disconnected", peer_clone.username);
                                connections_clone.write().await.remove(&peer_clone.id);
                                break;
                            }
                            Ok(_) => {
                                let line = line.trim();
                                if !line.is_empty() {
                                    if let Ok(message) = serde_json::from_str::<Message>(line) {
                                        debug!("Received message from {}: {:?}", peer_clone.username, message);
                                        let event = ChatEvent::new(peer_clone.clone(), message);
                                        if let Err(e) = event_sender_clone.send(event) {
                                            error!("Failed to send event: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to read from peer {}: {}", peer_clone.username, e);
                                connections_clone.write().await.remove(&peer_clone.id);
                                break;
                            }
                        }
                    }
                });
                
                Ok(())
            }
            Err(e) => {
                warn!("Failed to connect to peer {} at {}: {}", peer.username, addr, e);
                Err(e.into())
            }
        }
    }

    pub async fn disconnect_peer(&self, peer_id: &Uuid) {
        if let Some(_connection) = self.connections.write().await.remove(peer_id) {
            info!("Disconnected from peer {}", peer_id);
        }
    }

    pub async fn is_connected(&self, peer_id: &Uuid) -> bool {
        self.connections.read().await.contains_key(peer_id)
    }

    pub async fn connected_peers(&self) -> Vec<Peer> {
        self.connections.read().await.values().map(|conn| conn.peer.clone()).collect()
    }
    
    pub async fn get_connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}
