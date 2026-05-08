//! Top-of-screen resource tabs.
//!
//! Authored by okooo5km.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::state::{AppState, RESOURCES};

pub fn draw(f: &mut Frame, area: Rect, state: &AppState) {
    let titles: Vec<Span> = RESOURCES
        .iter()
        .enumerate()
        .map(|(i, (label, _, _))| {
            let style = if i == state.selected_resource {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Span::styled(format!(" {label} "), style)
        })
        .collect();
    let combined = Line::from(titles);
    let p =
        Paragraph::new(combined).block(Block::default().borders(Borders::ALL).title("keygen tui"));
    f.render_widget(p, area);
}
