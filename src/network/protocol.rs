use crate::config::Config;
use crate::message::{ChatEvent, Message};
use crate::network::{DiscoveryService, PeerManager};
use anyhow::Result;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tracing::{error, info};

pub struct NetworkProtocol {
    config: Config,
    discovery_service: DiscoveryService,
    peer_manager: PeerManager,
    event_receiver: mpsc::UnboundedReceiver<ChatEvent>,
    message_receiver: mpsc::UnboundedReceiver<(Message, SocketAddr)>,
}

impl NetworkProtocol {
    pub async fn new(config: Config) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        
        let discovery_service = DiscoveryService::new(config.clone(), event_sender).await?;
        let peer_manager = PeerManager::new(config.tcp_port_range.0, message_sender).await?;
        
        Ok(Self {
            config,
            discovery_service,
            peer_manager,
            event_receiver,
            message_receiver,
        })
    }

    pub async fn start(self) -> Result<()> {
        info!("Starting network protocol...");
        
        // Split self into components
        let discovery_service = self.discovery_service;
        let mut peer_manager = self.peer_manager;
        let mut event_receiver = self.event_receiver;
        let mut message_receiver = self.message_receiver;
        
        // Start discovery service
        let discovery_task = async move {
            if let Err(e) = discovery_service.start_discovery().await {
                error!("Discovery service failed: {}", e);
            }
        };
        
        // Start peer manager
        let peer_task = async move {
            if let Err(e) = peer_manager.start().await {
                error!("Peer manager failed: {}", e);
            }
        };
        
        // Handle events and messages
        let event_task = async move {
            loop {
                tokio::select! {
                    Some(event) = event_receiver.recv() => {
                        Self::handle_chat_event_static(event).await;
                    }
                    Some((message, addr)) = message_receiver.recv() => {
                        Self::handle_peer_message_static(message, addr).await;
                    }
                    else => break,
                }
            }
        };
        
        // Run all tasks concurrently
        tokio::select! {
            _ = discovery_task => {
                error!("Discovery task ended unexpectedly");
            }
            _ = peer_task => {
                error!("Peer task ended unexpectedly");
            }
            _ = event_task => {
                error!("Event task ended unexpectedly");
            }
        }
        
        Ok(())
    }

    async fn handle_chat_event_static(event: ChatEvent) {
        match event.message {
            Message::DiscoveryResponse { .. } => {
                // In a more complete implementation, we would connect to the peer here
                info!("Discovery response from peer: {}", event.peer.username);
            }
            Message::ChatMessage { .. } => {
                // Message will be handled by the UI
                info!("Chat message event: {:?}", event.message);
            }
            _ => {
                info!("Received event: {:?}", event.message);
            }
        }
    }

    async fn handle_peer_message_static(message: Message, _addr: SocketAddr) {
        match message {
            Message::ChatMessage { .. } => {
                // Forward to UI or message handler
                info!("Received chat message: {:?}", message);
            }
            Message::UserLeave { .. } => {
                info!("User left: {:?}", message);
            }
            _ => {
                info!("Received peer message: {:?}", message);
            }
        }
    }

    pub fn get_discovered_peers(&self) -> &std::collections::HashMap<uuid::Uuid, crate::message::Peer> {
        self.discovery_service.get_peers()
    }

    pub fn get_connected_peers(&self) -> Vec<&crate::message::Peer> {
        self.peer_manager.connected_peers()
    }
}
