use crate::message::{Message, Peer};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct PeerManager {
    listener: TcpListener,
    connections: HashMap<Uuid, PeerConnection>,
    message_sender: mpsc::UnboundedSender<(Message, SocketAddr)>,
}

struct PeerConnection {
    peer: Peer,
    stream: TcpStream,
}

impl PeerManager {
    pub async fn new(
        port: u16,
        message_sender: mpsc::UnboundedSender<(Message, SocketAddr)>,
    ) -> Result<Self> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(&addr)
            .await
            .context("Failed to bind TCP listener")?;
        
        info!("Peer manager listening on {}", addr);
        
        Ok(Self {
            listener,
            connections: HashMap::new(),
            message_sender,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting peer manager...");
        
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New peer connection from {}", addr);
                    
                    let message_sender = self.message_sender.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_peer_connection(stream, addr, message_sender).await {
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

    async fn handle_peer_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        message_sender: mpsc::UnboundedSender<(Message, SocketAddr)>,
    ) -> Result<()> {
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        
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
                                
                                if let Err(e) = message_sender.send((message, addr)) {
                                    error!("Failed to forward message: {}", e);
                                    break;
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
        
        Ok(())
    }

    pub async fn send_message_to_peer(&mut self, peer_id: &Uuid, message: &Message) -> Result<()> {
        if let Some(connection) = self.connections.get_mut(peer_id) {
            let data = serde_json::to_string(message)
                .context("Failed to serialize message")?;
            
            connection.stream.write_all(data.as_bytes()).await
                .context("Failed to write message to peer")?;
            
            connection.stream.write_all(b"\n").await
                .context("Failed to write newline to peer")?;
            
            connection.stream.flush().await
                .context("Failed to flush message to peer")?;
            
            debug!("Sent message to peer {}: {:?}", peer_id, message);
        } else {
            warn!("Peer {} not found in connections", peer_id);
        }
        
        Ok(())
    }

    pub async fn connect_to_peer(&mut self, peer: &Peer) -> Result<()> {
        let addr = SocketAddr::new(peer.ip, peer.port);
        
        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                info!("Connected to peer {} at {}", peer.username, addr);
                
                let connection = PeerConnection {
                    peer: peer.clone(),
                    stream,
                };
                
                self.connections.insert(peer.id, connection);
                
                // Send user join message
                let join_message = Message::user_join(peer.username.clone(), peer.id);
                self.send_message_to_peer(&peer.id, &join_message).await?;
                
                Ok(())
            }
            Err(e) => {
                warn!("Failed to connect to peer {} at {}: {}", peer.username, addr, e);
                Err(e.into())
            }
        }
    }

    pub fn disconnect_peer(&mut self, peer_id: &Uuid) {
        if let Some(_connection) = self.connections.remove(peer_id) {
            info!("Disconnected from peer {}", peer_id);
        }
    }

    pub fn is_connected(&self, peer_id: &Uuid) -> bool {
        self.connections.contains_key(peer_id)
    }

    pub fn connected_peers(&self) -> Vec<&Peer> {
        self.connections.values().map(|conn| &conn.peer).collect()
    }
}
