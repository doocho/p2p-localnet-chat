use super::types::{Message, Peer, ChatEvent};
use anyhow::Result;
use std::collections::HashMap;
use std::net::IpAddr;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub struct MessageHandler {
    peers: HashMap<Uuid, Peer>,
    peer_id: Uuid,
    username: String,
    event_sender: mpsc::UnboundedSender<ChatEvent>,
}

impl MessageHandler {
    pub fn new(
        username: String,
        event_sender: mpsc::UnboundedSender<ChatEvent>,
    ) -> Self {
        Self {
            peers: HashMap::new(),
            peer_id: Uuid::new_v4(),
            username,
            event_sender,
        }
    }

    pub fn peer_id(&self) -> Uuid {
        self.peer_id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn peers(&self) -> &HashMap<Uuid, Peer> {
        &self.peers
    }

    pub fn add_peer(&mut self, peer: Peer) {
        info!("Adding peer: {} ({})", peer.username, peer.ip);
        self.peers.insert(peer.id, peer);
    }

    pub fn remove_peer(&mut self, peer_id: &Uuid) {
        if let Some(peer) = self.peers.remove(peer_id) {
            info!("Removing peer: {} ({})", peer.username, peer.ip);
        }
    }

    pub fn update_peer_last_seen(&mut self, peer_id: &Uuid) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.update_last_seen();
        }
    }

    pub fn handle_message(&mut self, message: Message, sender_ip: IpAddr) -> Result<()> {
        match &message {
            Message::Discovery { username, port, peer_id } => {
                debug!("Received discovery from {} at {}:{}", username, sender_ip, port);
                
                let peer = Peer::new(username.clone(), sender_ip, *port);
                let peer_with_id = Peer {
                    id: *peer_id,
                    ..peer
                };
                
                self.add_peer(peer_with_id.clone());
                
                // Send discovery response
                let response = Message::discovery_response(
                    self.username.clone(),
                    8000, // TODO: Use actual listening port
                    self.peer_id,
                );
                
                let event = ChatEvent::new(peer_with_id, response);
                if let Err(e) = self.event_sender.send(event) {
                    warn!("Failed to send discovery response event: {}", e);
                }
            }
            
            Message::DiscoveryResponse { username, port, peer_id } => {
                debug!("Received discovery response from {} at {}:{}", username, sender_ip, port);
                
                let peer = Peer::new(username.clone(), sender_ip, *port);
                let peer_with_id = Peer {
                    id: *peer_id,
                    ..peer
                };
                
                self.add_peer(peer_with_id.clone());
                
                // Send both discovery response and connect events
                let response_event = ChatEvent::new(peer_with_id.clone(), message.clone());
                if let Err(e) = self.event_sender.send(response_event) {
                    warn!("Failed to send discovery response event: {}", e);
                }
                
                // Send a connect trigger event
                let connect_message = Message::user_join(username.clone(), *peer_id);
                let connect_event = ChatEvent::new(peer_with_id, connect_message);
                if let Err(e) = self.event_sender.send(connect_event) {
                    warn!("Failed to send connect event: {}", e);
                }
            }
            
            Message::ChatMessage { sender, .. } => {
                debug!("Received chat message from {}", sender);
                
                if let Some(peer) = self.peers.values().find(|p| p.username == *sender).cloned() {
                    let event = ChatEvent::new(peer, message);
                    if let Err(e) = self.event_sender.send(event) {
                        warn!("Failed to send chat message event: {}", e);
                    }
                }
            }
            
            Message::UserJoin { username, peer_id, .. } => {
                debug!("User {} joined", username);
                self.update_peer_last_seen(peer_id);
                
                if let Some(peer) = self.peers.get(peer_id).cloned() {
                    let event = ChatEvent::new(peer, message);
                    if let Err(e) = self.event_sender.send(event) {
                        warn!("Failed to send user join event: {}", e);
                    }
                }
            }
            
            Message::UserLeave { username, peer_id, .. } => {
                debug!("User {} left", username);
                self.remove_peer(peer_id);
                
                if let Some(peer) = self.peers.get(peer_id).cloned() {
                    let event = ChatEvent::new(peer, message);
                    if let Err(e) = self.event_sender.send(event) {
                        warn!("Failed to send user leave event: {}", e);
                    }
                }
            }
            
            Message::Heartbeat { peer_id, .. } => {
                debug!("Received heartbeat from peer {}", peer_id);
                self.update_peer_last_seen(peer_id);
            }
        }
        
        Ok(())
    }
}
