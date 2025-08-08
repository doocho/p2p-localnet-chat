use crate::ui::App;
use anyhow::Result;
use std::io::{self, stdout};
use tokio::time::{sleep, Duration};

pub struct TerminalUI {
    app: App,
}

impl TerminalUI {
    pub fn new(app: App) -> Self {
        Self { app }
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("🚀 Local Chat v1.0.0");
        println!("Connected as: {}", self.app.username);
        println!("Discovering peers on local network...");
        println!();
        println!("Commands:");
        println!("  Type a message and press Enter to send");
        println!("  Type 'quit' or 'exit' to leave");
        println!("  Ctrl+C to force quit");
        println!("{}", "=".repeat(50));
        
        self.app.update_status("Searching for peers...".to_string());

        loop {
            // Handle any pending events
            self.app.handle_events().await;
            
            // Display current status
            self.display_status();
            
            // Check if we should quit
            if self.app.should_quit {
                break;
            }
            
            // Check for user input (non-blocking)
            if let Some(input) = self.read_input_non_blocking()? {
                self.handle_input(input);
            }
            
            // Small delay to prevent busy waiting
            sleep(Duration::from_millis(100)).await;
        }
        
        println!("\nGoodbye! 👋");
        Ok(())
    }

    fn display_status(&self) {
        // Clear the current line and move cursor to beginning
        print!("\r{}", " ".repeat(80));
        print!("\r");
        
        // Display peer count and status
        let peer_count = self.app.get_peer_count();
        if peer_count > 0 {
            print!("🟢 Online ({}) | {}", peer_count, self.app.status);
        } else {
            print!("🔍 Searching... | {}", self.app.status);
        }
        
        // Display recent messages (last 3)
        if !self.app.messages.is_empty() {
            println!();
            let recent_messages: Vec<_> = self.app.messages
                .iter()
                .rev()
                .take(3)
                .rev()
                .collect();
            
            for msg in recent_messages {
                let time = msg.timestamp.format("%H:%M");
                if msg.is_own_message {
                    println!("[{}] You: {}", time, msg.content);
                } else {
                    println!("[{}] {}: {}", time, msg.sender, msg.content);
                }
            }
        }
        
        print!("> {}", self.app.input);
        io::Write::flush(&mut stdout()).unwrap();
    }

    fn read_input_non_blocking(&self) -> Result<Option<String>> {
        // This is a simplified implementation
        // In a real application, you'd want to use a proper async input handling
        // For now, we'll simulate input checking
        Ok(None)
    }

    fn handle_input(&mut self, input: String) {
        let input = input.trim();
        
        if input.is_empty() {
            return;
        }
        
        match input.to_lowercase().as_str() {
            "quit" | "exit" => {
                self.app.quit();
            }
            "help" => {
                println!();
                println!("Available commands:");
                println!("  help - Show this help message");
                println!("  peers - List connected peers");
                println!("  quit/exit - Leave the chat");
                println!();
            }
            "peers" => {
                println!();
                let peers = self.app.get_peer_list();
                if peers.is_empty() {
                    println!("No peers connected.");
                } else {
                    println!("Connected peers:");
                    for peer in peers {
                        println!("  - {}", peer);
                    }
                }
                println!();
            }
            _ => {
                // Treat as a message to send
                self.app.input = input.to_string();
                self.app.send_message();
            }
        }
    }
}

// Simplified terminal input handling for demo
// In production, you'd want to use crossterm or similar for proper async terminal handling
impl TerminalUI {
    pub async fn run_simple(&mut self) -> Result<()> {
        println!("🚀 Local Chat v1.0.0");
        println!("Connected as: {}", self.app.username);
        println!("Discovering peers on local network...");
        println!();
        
        // Simple message loop for demonstration
        loop {
            self.app.handle_events().await;
            
            let peer_count = self.app.get_peer_count();
            println!("Status: {} peers online | {}", peer_count, self.app.status);
            
            if !self.app.messages.is_empty() {
                let last_msg = self.app.messages.last().unwrap();
                let time = last_msg.timestamp.format("%H:%M");
                if last_msg.is_own_message {
                    println!("[{}] You: {}", time, last_msg.content);
                } else {
                    println!("[{}] {}: {}", time, last_msg.sender, last_msg.content);
                }
            }
            
            sleep(Duration::from_secs(2)).await;
            
            // For demo, quit after 30 seconds
            if self.app.messages.len() > 15 {
                break;
            }
        }
        
        Ok(())
    }
}
