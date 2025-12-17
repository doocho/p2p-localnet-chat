use crate::ui::{app::InputMode, theme::Theme, tui, widgets, App};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tokio::time::{Duration, Instant};

pub struct TerminalUI {
    app: App,
    theme: Theme,
}

impl TerminalUI {
    pub fn new(app: App) -> Self {
        Self {
            app,
            theme: Theme::default(),
        }
    }

    pub async fn run_interactive(&mut self) -> Result<()> {
        let mut terminal = tui::init()?;

        let tick_rate = Duration::from_millis(50);
        let mut last_tick = Instant::now();

        loop {
            // Handle network events
            self.app.handle_events().await;

            // Draw UI
            terminal.draw(|frame| {
                let peers: Vec<String> = self
                    .app
                    .peers
                    .values()
                    .map(|p| p.username.clone())
                    .collect();

                widgets::ui(
                    frame,
                    &self.app.username,
                    &self.app.channel,
                    &peers,
                    &self.app.messages,
                    &self.app.input,
                    self.app.cursor_pos,
                    self.app.scroll_offset,
                    &self.app.input_mode,
                    &self.app.focused_pane,
                    &self.theme,
                );
            })?;

            // Handle input with timeout
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if self.handle_key_event(key) {
                            break;
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }

            if self.app.should_quit {
                break;
            }
        }

        tui::restore()?;
        println!("Goodbye!");
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.app.input_mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Insert => self.handle_insert_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('i') => self.app.enter_insert_mode(),
            KeyCode::Char('a') => {
                self.app.move_cursor_right();
                self.app.enter_insert_mode();
            }
            KeyCode::Char('A') => {
                self.app.move_cursor_end();
                self.app.enter_insert_mode();
            }
            KeyCode::Char('I') => {
                self.app.move_cursor_start();
                self.app.enter_insert_mode();
            }
            KeyCode::Char('j') | KeyCode::Down => self.app.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.app.scroll_up(),
            KeyCode::Char('G') => self.app.scroll_to_bottom(),
            KeyCode::Char('g') => self.app.scroll_to_top(),
            KeyCode::Tab => self.app.cycle_focus(),
            KeyCode::Char('h') | KeyCode::Left => self.app.move_cursor_left(),
            KeyCode::Char('l') | KeyCode::Right => self.app.move_cursor_right(),
            KeyCode::Char('0') => self.app.move_cursor_start(),
            KeyCode::Char('$') => self.app.move_cursor_end(),
            KeyCode::Char('x') => self.app.delete_char(),
            KeyCode::Enter => {
                if !self.app.input.is_empty() {
                    self.app.send_message();
                }
            }
            _ => {}
        }
        false
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => self.app.enter_normal_mode(),
            KeyCode::Enter => {
                self.app.send_message();
            }
            KeyCode::Backspace => self.app.remove_char(),
            KeyCode::Delete => self.app.delete_char(),
            KeyCode::Left => self.app.move_cursor_left(),
            KeyCode::Right => self.app.move_cursor_right(),
            KeyCode::Home => self.app.move_cursor_start(),
            KeyCode::End => self.app.move_cursor_end(),
            KeyCode::Char(c) => self.app.add_char(c),
            _ => {}
        }
        false
    }

    // Legacy methods kept for compatibility
    pub async fn run(&mut self) -> Result<()> {
        self.run_interactive().await
    }

    pub async fn run_simple(&mut self) -> Result<()> {
        self.run_interactive().await
    }
}
