use crate::ui::{app::ChatMessage, theme::Theme};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{InputMode, Pane};

pub fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    username: &str,
    channel: &Option<String>,
    peer_count: usize,
    mode: &InputMode,
    theme: &Theme,
) {
    let mode_str = match mode {
        InputMode::Normal => " NORMAL ",
        InputMode::Insert => " INSERT ",
    };
    let mode_style = match mode {
        InputMode::Normal => theme.mode_normal,
        InputMode::Insert => theme.mode_insert,
    };

    let channel_str = channel.as_deref().unwrap_or("(none)");

    let status_text = vec![
        Span::styled(mode_str, mode_style),
        Span::raw(" | "),
        Span::styled("local-chat", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" v0.1.0 | "),
        Span::styled(username, theme.own_message),
        Span::raw(" | #"),
        Span::raw(channel_str),
        Span::raw(" | "),
        Span::styled(format!("{} peers", peer_count), theme.peer_online),
    ];

    let status = Paragraph::new(Line::from(status_text)).style(theme.status_bar);
    frame.render_widget(status, area);
}

pub fn render_peer_list(
    frame: &mut Frame,
    area: Rect,
    peers: &[String],
    focused: bool,
    theme: &Theme,
) {
    let border_style = if focused {
        theme.border_focused
    } else {
        theme.border
    };

    let items: Vec<ListItem> = peers
        .iter()
        .map(|peer| {
            let content = Line::from(vec![
                Span::styled("● ", theme.peer_online),
                Span::raw(peer),
            ]);
            ListItem::new(content)
        })
        .collect();

    let peers_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(" Peers ", theme.peer_header));

    let peers_list = List::new(items).block(peers_block);
    frame.render_widget(peers_list, area);
}

pub fn render_messages(
    frame: &mut Frame,
    area: Rect,
    messages: &[ChatMessage],
    scroll_offset: usize,
    _username: &str,
    focused: bool,
    theme: &Theme,
) {
    let border_style = if focused {
        theme.border_focused
    } else {
        theme.border
    };

    let inner_height = area.height.saturating_sub(2) as usize;
    let total_messages = messages.len();

    let start_idx = if total_messages > inner_height {
        total_messages.saturating_sub(inner_height).saturating_sub(scroll_offset)
    } else {
        0
    };
    let end_idx = total_messages.saturating_sub(scroll_offset);

    let visible_messages: Vec<Line> = messages
        .get(start_idx..end_idx)
        .unwrap_or(&[])
        .iter()
        .map(|msg| {
            let time = msg.timestamp.format("%H:%M");
            let timestamp_span = Span::styled(format!("[{}] ", time), theme.timestamp);

            if msg.is_own_message {
                Line::from(vec![
                    timestamp_span,
                    Span::styled("You: ", theme.own_message.add_modifier(Modifier::BOLD)),
                    Span::styled(&msg.content, theme.own_message),
                ])
            } else {
                let user_style = theme.user_style(&msg.sender);
                Line::from(vec![
                    timestamp_span,
                    Span::styled(format!("{}: ", msg.sender), user_style.add_modifier(Modifier::BOLD)),
                    Span::raw(&msg.content),
                ])
            }
        })
        .collect();

    let scroll_indicator = if scroll_offset > 0 {
        format!(" Messages [{}/{}] ", end_idx, total_messages)
    } else {
        " Messages ".to_string()
    };

    let messages_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(scroll_indicator, theme.title));

    let messages_widget = Paragraph::new(visible_messages)
        .block(messages_block)
        .wrap(Wrap { trim: false });

    frame.render_widget(messages_widget, area);
}

pub fn render_input(
    frame: &mut Frame,
    area: Rect,
    input: &str,
    cursor_pos: usize,
    mode: &InputMode,
    focused: bool,
    theme: &Theme,
) {
    let border_style = if focused {
        theme.border_focused
    } else {
        theme.border
    };

    let mode_indicator = match mode {
        InputMode::Normal => "[NORMAL]",
        InputMode::Insert => "[INSERT]",
    };

    let title = format!(" {} > ", mode_indicator);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(title, theme.title));

    let input_widget = Paragraph::new(input)
        .style(theme.input)
        .block(input_block);

    frame.render_widget(input_widget, area);

    if matches!(mode, InputMode::Insert) && focused {
        let cursor_x = area.x + 1 + cursor_pos as u16;
        let cursor_y = area.y + 1;
        if cursor_x < area.x + area.width - 1 {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

pub fn render_help_bar(frame: &mut Frame, area: Rect, mode: &InputMode, _theme: &Theme) {
    let help_text = match mode {
        InputMode::Normal => {
            "i: insert | j/k: scroll | G: bottom | g: top | Tab: switch pane | q: quit"
        }
        InputMode::Insert => "Esc: normal mode | Enter: send | Ctrl+C: quit",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(ratatui::style::Color::DarkGray));
    frame.render_widget(help, area);
}

pub fn ui(
    frame: &mut Frame,
    username: &str,
    channel: &Option<String>,
    peers: &[String],
    messages: &[ChatMessage],
    input: &str,
    cursor_pos: usize,
    scroll_offset: usize,
    mode: &InputMode,
    focused_pane: &Pane,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Status bar
            Constraint::Min(5),     // Main content
            Constraint::Length(3),  // Input
            Constraint::Length(1),  // Help bar
        ])
        .split(frame.area());

    let status_area = chunks[0];
    let main_area = chunks[1];
    let input_area = chunks[2];
    let help_area = chunks[3];

    let main_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Min(20),        // Messages
            Constraint::Length(20),     // Peers sidebar
        ])
        .split(main_area);

    let messages_area = main_chunks[0];
    let peers_area = main_chunks[1];

    render_status_bar(
        frame,
        status_area,
        username,
        channel,
        peers.len(),
        mode,
        theme,
    );

    render_messages(
        frame,
        messages_area,
        messages,
        scroll_offset,
        username,
        matches!(focused_pane, Pane::Messages),
        theme,
    );

    render_peer_list(
        frame,
        peers_area,
        peers,
        matches!(focused_pane, Pane::Peers),
        theme,
    );

    render_input(
        frame,
        input_area,
        input,
        cursor_pos,
        mode,
        matches!(focused_pane, Pane::Input),
        theme,
    );

    render_help_bar(frame, help_area, mode, theme);
}
