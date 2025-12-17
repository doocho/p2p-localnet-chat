use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub border: Style,
    pub border_focused: Style,
    pub title: Style,
    pub status_bar: Style,
    pub own_message: Style,
    pub other_message: Style,
    pub system_message: Style,
    pub timestamp: Style,
    pub input: Style,
    pub input_cursor: Style,
    pub mode_normal: Style,
    pub mode_insert: Style,
    pub peer_online: Style,
    pub peer_header: Style,
    pub highlight: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border: Style::default().fg(Color::DarkGray),
            border_focused: Style::default().fg(Color::Cyan),
            title: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            status_bar: Style::default().bg(Color::DarkGray).fg(Color::White),
            own_message: Style::default().fg(Color::Cyan),
            other_message: Style::default().fg(Color::Magenta),
            system_message: Style::default().fg(Color::Yellow),
            timestamp: Style::default().fg(Color::DarkGray),
            input: Style::default().fg(Color::White),
            input_cursor: Style::default().bg(Color::White).fg(Color::Black),
            mode_normal: Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            mode_insert: Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            peer_online: Style::default().fg(Color::Green),
            peer_header: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            highlight: Style::default().bg(Color::DarkGray),
        }
    }
}

impl Theme {
    pub fn user_color(&self, username: &str) -> Color {
        let hash = username.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        let colors = [
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::LightCyan,
        ];
        colors[(hash as usize) % colors.len()]
    }

    pub fn user_style(&self, username: &str) -> Style {
        Style::default().fg(self.user_color(username))
    }
}
