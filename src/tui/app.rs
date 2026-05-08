//! TUI top-level state and event loop.
//!
//! The loop is fully async — `crossterm::EventStream` for keystrokes and a
//! `tokio::mpsc` channel for fetch results — so a slow API call never blocks
//! the redraw or input handling. Tab switches kick off a background `tokio::spawn`
//! that posts back when the response lands.

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::{
    api::{client::Query, jsonapi::Resource, Client},
    cli::Context,
    error::Result,
    Error,
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
    /// Monotonic counter so a slow fetch from a previously selected tab is
    /// discarded if the user has already moved on.
    fetch_seq: u64,
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
            fetch_seq: 0,
        }
    }

    fn move_left(&mut self) {
        if self.selected_resource > 0 {
            self.selected_resource -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.selected_resource + 1 < RESOURCES.len() {
            self.selected_resource += 1;
        }
    }

    fn move_down(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let i = self
            .list_state
            .selected()
            .map_or(0, |i| (i + 1) % self.rows.len());
        self.list_state.select(Some(i));
    }

    fn move_up(&mut self) {
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

/// Result coming back from a spawned fetch task.
struct FetchResult {
    seq: u64,
    resource_idx: usize,
    payload: std::result::Result<Vec<Resource>, String>,
}

pub async fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    ctx: &Context,
) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<FetchResult>();
    let mut events = EventStream::new();
    let mut state = AppState::new();

    // Kick off the initial load for the first tab.
    state.loading = true;
    state.status = format!("loading {}…", RESOURCES[state.selected_resource].2);
    spawn_fetch(ctx, state.selected_resource, state.fetch_seq, tx.clone());

    loop {
        terminal
            .draw(|f| ui(f, &mut state))
            .map_err(|e| Error::user(format!("tui draw: {e}")))?;

        tokio::select! {
            // Fetched data arrives.
            Some(res) = rx.recv() => {
                apply_fetch(&mut state, res);
            }
            // Keystroke.
            Some(event) = events.next() => {
                let event = event.map_err(|e| Error::user(format!("tui read: {e}")))?;
                if let Event::Key(key) = event {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('r') => {
                            kick_fetch(&mut state, ctx, &tx);
                        }
                        KeyCode::Tab | KeyCode::Right => {
                            state.move_right();
                            kick_fetch(&mut state, ctx, &tx);
                        }
                        KeyCode::BackTab | KeyCode::Left => {
                            state.move_left();
                            kick_fetch(&mut state, ctx, &tx);
                        }
                        KeyCode::Down | KeyCode::Char('j') => state.move_down(),
                        KeyCode::Up | KeyCode::Char('k') => state.move_up(),
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

fn kick_fetch(state: &mut AppState, ctx: &Context, tx: &mpsc::UnboundedSender<FetchResult>) {
    state.fetch_seq = state.fetch_seq.wrapping_add(1);
    state.loading = true;
    state.error = None;
    state.rows.clear();
    state.list_state.select(None);
    state.status = format!("loading {}…", RESOURCES[state.selected_resource].2);
    spawn_fetch(ctx, state.selected_resource, state.fetch_seq, tx.clone());
}

fn apply_fetch(state: &mut AppState, res: FetchResult) {
    // Discard responses for an older fetch (the user already moved on).
    if res.seq != state.fetch_seq || res.resource_idx != state.selected_resource {
        return;
    }
    state.loading = false;
    match res.payload {
        Ok(rows) => {
            state.status = format!(
                "loaded {n} rows from {p}",
                n = rows.len(),
                p = RESOURCES[res.resource_idx].2
            );
            state
                .list_state
                .select(if rows.is_empty() { None } else { Some(0) });
            state.rows = rows;
        }
        Err(msg) => {
            state.error = Some(msg);
            state.status = format!("failed to load {}", RESOURCES[res.resource_idx].2);
        }
    }
}

fn spawn_fetch(
    ctx: &Context,
    resource_idx: usize,
    seq: u64,
    tx: mpsc::UnboundedSender<FetchResult>,
) {
    let ctx = ctx.clone();
    let path = RESOURCES[resource_idx].2;
    tokio::spawn(async move {
        let payload = match Client::new(&ctx) {
            Ok(client) => client
                .get::<Vec<Resource>>(path, &Query::new().page(1, 50))
                .await
                .map(|d| d.data)
                .map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };
        let _ = tx.send(FetchResult {
            seq,
            resource_idx,
            payload,
        });
    });
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
