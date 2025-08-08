mod config;
mod message;
mod network;
mod ui;

use anyhow::Result;
use config::Config;
use message::ChatEvent;
use network::DiscoveryService;
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
    
    // Create the app
    let app = App::new(username, event_receiver, message_sender);
    let mut terminal_ui = TerminalUI::new(app);
    
    // Start discovery service in background
    let discovery_config = config.clone();
    let discovery_task = tokio::spawn(async move {
        match DiscoveryService::new(discovery_config, event_sender).await {
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
    
    // Handle outgoing messages
    let message_task = tokio::spawn(async move {
        while let Some(message) = message_receiver.recv().await {
            // In a full implementation, this would send the message through the network
            info!("Would send message: {}", message);
        }
    });
    
    // Run the terminal UI
    let ui_task = tokio::spawn(async move {
        if let Err(e) = terminal_ui.run_simple().await {
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
