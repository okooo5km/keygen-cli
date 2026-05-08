//! TUI top-level state and event loop.

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use serde_json::Value;

use crate::{
    api::{client::Query, jsonapi::Resource, Client},
    cli::Context,
    error::Result,
};

const RESOURCES: &[(&str, &str, &str)] = &[
    ("Licenses", "licenses", "/licenses"),
    ("Machines", "machines", "/machines"),
    ("Policies", "policies", "/policies"),
    ("Products", "products", "/products"),
    ("Users", "users", "/users"),
    ("Groups", "groups", "/groups"),
    ("Releases", "releases", "/releases"),
    ("Artifacts", "artifacts", "/artifacts"),
    ("Webhooks", "webhook-endpoints", "/webhook-endpoints"),
];

struct AppState {
    selected_resource: usize,
    list_state: ListState,
    rows: Vec<Resource>,
    error: Option<String>,
    loading: bool,
    status: String,
}

impl AppState {
    fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            selected_resource: 0,
            list_state: state,
            rows: Vec::new(),
            error: None,
            loading: false,
            status: "Ready. Tab/Shift-Tab switches resource. q quits.".into(),
        }
    }

    fn on_left(&mut self) {
        if self.selected_resource > 0 {
            self.selected_resource -= 1;
        }
    }

    fn on_right(&mut self) {
        if self.selected_resource + 1 < RESOURCES.len() {
            self.selected_resource += 1;
        }
    }

    fn on_down(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let i = self
            .list_state
            .selected()
            .map_or(0, |i| (i + 1) % self.rows.len());
        self.list_state.select(Some(i));
    }

    fn on_up(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let i =
            self.list_state
                .selected()
                .map_or(0, |i| if i == 0 { self.rows.len() - 1 } else { i - 1 });
        self.list_state.select(Some(i));
    }
}

pub async fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    ctx: &Context,
) -> Result<()> {
    let mut state = AppState::new();
    refresh(ctx, &mut state).await;

    loop {
        terminal
            .draw(|f| ui(f, &mut state))
            .map_err(|e| crate::Error::user(format!("tui draw: {e}")))?;

        if event::poll(Duration::from_millis(200))
            .map_err(|e| crate::Error::user(format!("tui poll: {e}")))?
        {
            if let Event::Key(key) =
                event::read().map_err(|e| crate::Error::user(format!("tui read: {e}")))?
            {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => refresh(ctx, &mut state).await,
                    KeyCode::Tab | KeyCode::Right => {
                        state.on_right();
                        refresh(ctx, &mut state).await;
                    }
                    KeyCode::BackTab | KeyCode::Left => {
                        state.on_left();
                        refresh(ctx, &mut state).await;
                    }
                    KeyCode::Down | KeyCode::Char('j') => state.on_down(),
                    KeyCode::Up | KeyCode::Char('k') => state.on_up(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

async fn refresh(ctx: &Context, state: &mut AppState) {
    state.loading = true;
    state.error = None;
    state.rows.clear();
    let (_, _ty, path) = RESOURCES[state.selected_resource];
    state.status = format!("loading {path}…");
    match Client::new(ctx) {
        Ok(client) => {
            match client
                .get::<Vec<Resource>>(path, &Query::new().page(1, 50))
                .await
            {
                Ok(doc) => {
                    state.rows = doc.data;
                    state
                        .list_state
                        .select(if state.rows.is_empty() { None } else { Some(0) });
                    state.status = format!("loaded {n} rows from {path}", n = state.rows.len());
                }
                Err(e) => state.error = Some(e.to_string()),
            }
        }
        Err(e) => state.error = Some(e.to_string()),
    }
    state.loading = false;
}

fn ui(f: &mut Frame, state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(1),    // body
            Constraint::Length(1), // status bar
        ])
        .split(f.area());

    draw_tabs(f, chunks[0], state);
    draw_body(f, chunks[1], state);
    draw_status(f, chunks[2], state);
}

fn draw_tabs(f: &mut Frame, area: Rect, state: &AppState) {
    let titles: Vec<Line> = RESOURCES
        .iter()
        .enumerate()
        .map(|(i, (label, _, _))| {
            let style = if i == state.selected_resource {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(format!(" {label} "), style))
        })
        .collect();
    let combined = Line::from(
        titles
            .into_iter()
            .flat_map(|l| l.spans.into_iter())
            .collect::<Vec<_>>(),
    );
    let p =
        Paragraph::new(combined).block(Block::default().borders(Borders::ALL).title("keygen tui"));
    f.render_widget(p, area);
}

fn draw_body(f: &mut Frame, area: Rect, state: &mut AppState) {
    if let Some(err) = &state.error {
        let p = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("error"));
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = state
        .rows
        .iter()
        .map(|r| ListItem::new(format_row(r)))
        .collect();

    let title = format!(
        " {} ({} rows) — j/k to navigate, r to refresh, Tab to switch ",
        RESOURCES[state.selected_resource].0,
        state.rows.len()
    );
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, &mut state.list_state);
}

fn draw_status(f: &mut Frame, area: Rect, state: &AppState) {
    let msg = if state.loading {
        format!("⟳ {}", state.status)
    } else {
        state.status.clone()
    };
    let p = Paragraph::new(msg).style(Style::default().fg(Color::DarkGray));
    f.render_widget(p, area);
}

fn format_row(r: &Resource) -> String {
    let id_short = if r.id.len() > 10 { &r.id[..10] } else { &r.id };
    let name = r
        .attributes
        .pointer("/name")
        .and_then(Value::as_str)
        .or_else(|| r.attributes.pointer("/key").and_then(Value::as_str))
        .unwrap_or("—");
    let status = r
        .attributes
        .pointer("/status")
        .and_then(Value::as_str)
        .unwrap_or("—");
    format!("{id_short:10}  {name:30}  {status}")
}
