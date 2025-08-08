use crate::message::{ChatEvent, Message, Peer};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub is_own_message: bool,
}

pub struct App {
    pub username: String,
    pub peers: HashMap<Uuid, Peer>,
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub should_quit: bool,
    pub status: String,
    event_receiver: mpsc::UnboundedReceiver<ChatEvent>,
    message_sender: mpsc::UnboundedSender<String>,
    connection_sender: Option<mpsc::UnboundedSender<Peer>>,
}

impl App {
    pub fn new(
        username: String,
        event_receiver: mpsc::UnboundedReceiver<ChatEvent>,
        message_sender: mpsc::UnboundedSender<String>,
        connection_sender: Option<mpsc::UnboundedSender<Peer>>,
    ) -> Self {
        Self {
            username,
            peers: HashMap::new(),
            messages: Vec::new(),
            input: String::new(),
            should_quit: false,
            status: "Starting...".to_string(),
            event_receiver,
            message_sender,
            connection_sender,
        }
    }

    pub fn update_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn add_message(&mut self, sender: String, content: String, is_own_message: bool) {
        let message = ChatMessage {
            sender,
            content,
            timestamp: Utc::now(),
            is_own_message,
        };
        
        self.messages.push(message);
        
        // Keep only last 100 messages to prevent memory issues
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }

    pub fn send_message(&mut self) {
        if !self.input.trim().is_empty() {
            let content = self.input.trim().to_string();
            
            // Add to our own message history
            self.add_message(self.username.clone(), content.clone(), true);
            
            // Send to network
            if let Err(e) = self.message_sender.send(content) {
                self.update_status(format!("Failed to send message: {}", e));
            }
            
            self.input.clear();
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn add_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn remove_char(&mut self) {
        self.input.pop();
    }

    pub async fn handle_events(&mut self) {
        // Non-blocking check for new events
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_chat_event(event);
        }
    }

    fn handle_chat_event(&mut self, event: ChatEvent) {
        match event.message {
            Message::Discovery { username, .. } => {
                self.peers.insert(event.peer.id, event.peer);
                self.update_status(format!("Discovered peer: {}", username));
            }
            Message::DiscoveryResponse { username, .. } => {
                self.peers.insert(event.peer.id, event.peer.clone());
                self.update_status(format!("Found peer: {}", username));
                
                // Trigger TCP connection to this peer
                if let Some(ref connection_sender) = self.connection_sender {
                    if let Err(e) = connection_sender.send(event.peer.clone()) {
                        self.update_status(format!("Failed to trigger connection to {}: {}", username, e));
                    } else {
                        self.update_status(format!("Connecting to {}...", username));
                    }
                }
            }
            Message::ChatMessage { sender, content, .. } => {
                self.add_message(sender, content, false);
            }
            Message::UserJoin { username, .. } => {
                self.peers.insert(event.peer.id, event.peer);
                self.update_status(format!("{} joined via TCP", username));
            }
            Message::UserLeave { username, .. } => {
                self.peers.remove(&event.peer.id);
                self.update_status(format!("{} left the chat", username));
            }
            Message::Heartbeat { .. } => {
                // Update peer's last seen time
                if let Some(peer) = self.peers.get_mut(&event.peer.id) {
                    *peer = event.peer;
                }
            }
        }
    }

    pub fn get_peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn get_peer_list(&self) -> Vec<String> {
        self.peers
            .values()
            .map(|peer| format!("{} ({})", peer.username, peer.ip))
            .collect()
    }
}
