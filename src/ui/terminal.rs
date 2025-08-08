use crate::ui::App;
use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, stdout, Write};
use tokio::time::{sleep, Duration};

pub struct TerminalUI {
    app: App,
}

impl TerminalUI {
    pub fn new(app: App) -> Self {
        Self { app }
    }
    
    pub async fn run_interactive(&mut self) -> Result<()> {
        // Enable raw mode for real-time input
        terminal::enable_raw_mode()?;
        
        // Clear screen and set up initial display
        execute!(
            stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        
        self.display_header()?;
        
        // Main input loop
        loop {
            // Handle any pending chat events
            self.app.handle_events().await;
            
            // Redraw UI
            self.redraw_ui()?;
            
            // Check for keyboard input (non-blocking)
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key_event) => {
                        if self.handle_key_event(key_event).await? {
                            break; // User wants to quit
                        }
                    }
                    _ => {}
                }
            }
            
            // Small delay to prevent excessive CPU usage
            sleep(Duration::from_millis(50)).await;
        }
        
        // Restore terminal
        terminal::disable_raw_mode()?;
        execute!(stdout(), cursor::Show)?;
        println!("\n\nGoodbye! 👋");
        
        Ok(())
    }
    
    fn display_header(&self) -> Result<()> {
        execute!(
            stdout(),
            SetForegroundColor(Color::Cyan),
            Print("🚀 Local Chat v1.0.0\n"),
            SetForegroundColor(Color::Green),
            Print(format!("Connected as: {}\n", self.app.username)),
            ResetColor,
            Print("Press Ctrl+C to quit | Type your message and press Enter to send\n"),
            Print("─".repeat(60)),
            Print("\n")
        )?;
        Ok(())
    }
    
    fn redraw_ui(&self) -> Result<()> {
        // Move to message area (line 5)
        execute!(stdout(), cursor::MoveTo(0, 4))?;
        
        // Clear from cursor to end of screen
        execute!(stdout(), terminal::Clear(ClearType::FromCursorDown))?;
        
        // Display peer status
        let peer_count = self.app.get_peer_count();
        if peer_count > 0 {
            execute!(
                stdout(),
                SetForegroundColor(Color::Green),
                Print(format!("🟢 {} peers connected\n", peer_count)),
                ResetColor
            )?;
            
            // Show peer list
            for peer_info in self.app.get_peer_list() {
                execute!(stdout(), Print(format!("  └─ {}\n", peer_info)))?;
            }
        } else {
            execute!(
                stdout(),
                SetForegroundColor(Color::Yellow),
                Print("🔍 Searching for peers...\n"),
                ResetColor
            )?;
        }
        
        println!();
        
        // Display recent messages (last 10)
        let recent_messages: Vec<_> = self.app.messages
            .iter()
            .rev()
            .take(10)
            .rev()
            .collect();
        
        if !recent_messages.is_empty() {
            execute!(
                stdout(),
                SetForegroundColor(Color::Cyan),
                Print("💬 Recent Messages:\n"),
                ResetColor
            )?;
            
            for msg in recent_messages {
                let time = msg.timestamp.format("%H:%M:%S");
                if msg.is_own_message {
                    execute!(
                        stdout(),
                        SetForegroundColor(Color::Blue),
                        Print(format!("[{}] You: {}\n", time, msg.content)),
                        ResetColor
                    )?;
                } else {
                    execute!(
                        stdout(),
                        SetForegroundColor(Color::Magenta),
                        Print(format!("[{}] {}: {}\n", time, msg.sender, msg.content)),
                        ResetColor
                    )?;
                }
            }
            println!();
        }
        
        // Display input line
        execute!(
            stdout(),
            SetForegroundColor(Color::White),
            Print("─".repeat(60)),
            Print("\n> "),
            Print(&self.app.input),
            ResetColor
        )?;
        
        stdout().flush()?;
        Ok(())
    }
    
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool> {
        // Only handle key press events (not release)
        if key_event.kind != KeyEventKind::Press {
            return Ok(false);
        }
        
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                // Ctrl+C to quit
                return Ok(true);
            }
            KeyCode::Enter => {
                // Send message
                if !self.app.input.trim().is_empty() {
                    self.app.send_message();
                    self.app.input.clear();
                }
            }
            KeyCode::Backspace => {
                // Remove last character
                self.app.remove_char();
            }
            KeyCode::Char(c) => {
                // Add character to input
                self.app.add_char(c);
            }
            _ => {}
        }
        
        Ok(false)
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
        println!("Listening on UDP port 7878 for peer discovery...");
        println!();
        
        let mut last_peer_count = 0;
        let mut status_counter = 0;
        
        // Enhanced message loop for peer discovery
        loop {
            self.app.handle_events().await;
            
            let peer_count = self.app.get_peer_count();
            
            // Show status every 10 iterations or when peer count changes
            if status_counter % 10 == 0 || peer_count != last_peer_count {
                println!("Status: {} peers discovered | {}", peer_count, self.app.status);
                
                if peer_count > 0 {
                    println!("🟢 Discovered peers:");
                    for peer_info in self.app.get_peer_list() {
                        println!("  - {}", peer_info);
                    }
                    println!();
                }
                
                last_peer_count = peer_count;
            }
            
            // Show any new messages
            if !self.app.messages.is_empty() {
                let last_msg = self.app.messages.last().unwrap();
                let time = last_msg.timestamp.format("%H:%M");
                if last_msg.is_own_message {
                    println!("[{}] You: {}", time, last_msg.content);
                } else {
                    println!("[{}] {}: {}", time, last_msg.sender, last_msg.content);
                }
            }
            
            sleep(Duration::from_millis(500)).await;
            status_counter += 1;
            
            // Run for 2 minutes to see discovery in action
            if status_counter > 240 {
                println!("Demo completed. Discovered {} peers total.", peer_count);
                break;
            }
        }
        
        Ok(())
    }
}
