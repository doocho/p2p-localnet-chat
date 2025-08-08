mod config;
mod message;
mod network;
mod ui;

use anyhow::Result;
use config::Config;
use message::ChatEvent;
use network::{DiscoveryService, PeerManager};
use std::env;
use tokio::sync::mpsc;
use tracing::{error, info};
use ui::{App, TerminalUI};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting Local Chat v1.0.0");
    
    // Get username from command line args or use default
    let args: Vec<String> = env::args().collect();
    let username = if args.len() > 1 {
        args[1].clone()
    } else {
        whoami::username()
    };
    
    // Create configuration
    let config = Config::new().with_username(username.clone());
    info!("Starting as user: {}", config.username);
    
    // Create channels for communication between components
    let (event_sender, event_receiver) = mpsc::unbounded_channel::<ChatEvent>();
    let (message_sender, mut message_receiver) = mpsc::unbounded_channel::<String>();
    
    // Create peer manager
    let peer_manager = std::sync::Arc::new(PeerManager::new(
        config.tcp_port_range.0,
        event_sender.clone(),
        username.clone(),
        uuid::Uuid::new_v4(),
    ).await?);
    
    // Create channels for peer connection coordination
    let (connection_sender, mut connection_receiver) = mpsc::unbounded_channel::<message::Peer>();
    
    // Create event channels for peer connection coordination
    let (connection_event_sender, mut connection_event_receiver) = mpsc::unbounded_channel::<ChatEvent>();
    
    // Create the app with connection sender for auto-connection
    let app = App::new(username.clone(), event_receiver, message_sender.clone(), Some(connection_sender.clone()));
    let mut terminal_ui = TerminalUI::new(app);
    
    // Get the actual TCP port from PeerManager
    let tcp_port = peer_manager.get_tcp_port()?;
    info!("Using TCP port {} for peer discovery", tcp_port);
    
    // Start discovery service in background
    let discovery_config = config.clone();
    let discovery_task = tokio::spawn(async move {
        match DiscoveryService::new(discovery_config, event_sender, tcp_port).await {
            Ok(discovery_service) => {
                info!("Discovery service created, starting...");
                if let Err(e) = discovery_service.start_discovery().await {
                    error!("Discovery service failed: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to create discovery service: {}", e);
            }
        }
    });
    
    // Start peer manager in background
    let peer_manager_clone = peer_manager.clone();
    let peer_manager_task = tokio::spawn(async move {
        if let Err(e) = peer_manager_clone.start().await {
            error!("Peer manager failed: {}", e);
        }
    });
    
    // Handle peer connections
    let peer_manager_for_connections = peer_manager.clone();
    let connection_task = tokio::spawn(async move {
        while let Some(peer) = connection_receiver.recv().await {
            info!("Attempting to connect to peer: {}", peer.username);
            if let Err(e) = peer_manager_for_connections.connect_to_peer(&peer).await {
                error!("Failed to connect to peer {}: {}", peer.username, e);
            }
        }
    });
    
    // Handle outgoing messages
    let peer_manager_for_messages = peer_manager.clone();
    let username_for_messages = username.clone();
    let message_task = tokio::spawn(async move {
        while let Some(message_content) = message_receiver.recv().await {
            info!("Broadcasting message: {}", message_content);
            
            // Create a chat message
            let chat_message = message::Message::chat_message(
                username_for_messages.clone(),
                "all".to_string(),
                message_content
            );
            
            // Broadcast to all connected peers
            if let Err(e) = peer_manager_for_messages.broadcast_message(&chat_message).await {
                error!("Failed to broadcast message: {}", e);
            }
        }
    });
    
    // Run the terminal UI (interactive mode)
    let ui_task = tokio::spawn(async move {
        if let Err(e) = terminal_ui.run_interactive().await {
            error!("Terminal UI failed: {}", e);
        }
    });
    
    info!("All components started. Press Ctrl+C to quit.");
    
    // Wait for any task to complete (or user to quit)
            tokio::select! {
            result = discovery_task => {
                if let Err(e) = result {
                    error!("Discovery task panicked: {}", e);
                }
            }
            result = peer_manager_task => {
                if let Err(e) = result {
                    error!("Peer manager task panicked: {}", e);
                }
            }
            result = connection_task => {
                if let Err(e) = result {
                    error!("Connection task panicked: {}", e);
                }
            }
            result = message_task => {
                if let Err(e) = result {
                    error!("Message task panicked: {}", e);
                }
            }
            result = ui_task => {
                if let Err(e) = result {
                    error!("UI task panicked: {}", e);
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
            }
        }
    
    info!("Local Chat shutting down. Goodbye! 👋");
    Ok(())
}
