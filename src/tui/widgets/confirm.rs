//! Confirmation overlay: dry-run envelope preview plus a Tier-aware banner.
//!
//! Authored by okooo5km.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use serde_json::Value;

use crate::tui::{permission::Tier, views::actions::HttpMethod};

pub struct ConfirmState {
    pub action_label: &'static str,
    pub tier: Tier,
    pub method: HttpMethod,
    pub path: String,
    pub body: Value,
    pub envelope_pretty: String,
    pub in_flight: bool,
    pub feedback: Option<ConfirmFeedback>,
}

pub enum ConfirmFeedback {
    Ok(String),
    Err(String),
}

impl ConfirmState {
    pub fn new(
        action_label: &'static str,
        tier: Tier,
        method: HttpMethod,
        path: String,
        body: Value,
        envelope_pretty: String,
    ) -> Self {
        Self {
            action_label,
            tier,
            method,
            path,
            body,
            envelope_pretty,
            in_flight: false,
            feedback: None,
        }
    }
}

pub fn draw(f: &mut Frame, area: Rect, state: &ConfirmState) {
    let popup = centered(area, 76, 80);
    f.render_widget(Clear, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // banner
            Constraint::Min(5),    // envelope
            Constraint::Length(3), // feedback / hint
        ])
        .split(popup);

    draw_banner(f, chunks[0], state);
    draw_envelope(f, chunks[1], state);
    draw_footer(f, chunks[2], state);
}

fn draw_banner(f: &mut Frame, area: Rect, state: &ConfirmState) {
    let (style, text) = match state.tier {
        Tier::Explicit => (
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
            format!(
                " ⚠ DESTRUCTIVE / IRREVERSIBLE  —  {} {} ",
                method_label(state.method),
                state.action_label
            ),
        ),
        Tier::DryRunConfirm => (
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            format!(
                " confirm  —  {} {} ",
                method_label(state.method),
                state.action_label
            ),
        ),
        Tier::AutoRun => (
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
            format!(
                " safe  —  {} {} ",
                method_label(state.method),
                state.action_label
            ),
        ),
    };
    let p = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::ALL).border_style(style));
    f.render_widget(p, area);
}

fn draw_envelope(f: &mut Frame, area: Rect, state: &ConfirmState) {
    let title = format!(" dry-run envelope  ·  path: {} ", state.path);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));
    let p = Paragraph::new(state.envelope_pretty.as_str())
        .block(block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));
    f.render_widget(p, area);
}

fn draw_footer(f: &mut Frame, area: Rect, state: &ConfirmState) {
    let line = if let Some(fb) = &state.feedback {
        match fb {
            ConfirmFeedback::Ok(msg) => Line::from(vec![Span::styled(
                format!("✓ {msg} — press Esc to close"),
                Style::default().fg(Color::Green),
            )]),
            ConfirmFeedback::Err(msg) => Line::from(vec![Span::styled(
                format!("✗ {msg} — press Esc to close"),
                Style::default().fg(Color::Red),
            )]),
        }
    } else if state.in_flight {
        Line::from(vec![Span::styled(
            "⟳ executing…",
            Style::default().fg(Color::Yellow),
        )])
    } else {
        Line::from(vec![
            Span::styled(
                " y ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" run for real    "),
            Span::styled(
                " n ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" cancel    "),
            Span::styled(" Esc ", Style::default().fg(Color::DarkGray)),
            Span::raw(" close"),
        ])
    };
    let p = Paragraph::new(line).block(Block::default().borders(Borders::TOP));
    f.render_widget(p, area);
}

fn method_label(m: HttpMethod) -> &'static str {
    match m {
        HttpMethod::Post => "POST",
        HttpMethod::Delete => "DELETE",
    }
}

fn centered(area: Rect, pct_x: u16, pct_y: u16) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(v[1])[1]
}
