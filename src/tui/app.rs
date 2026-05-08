//! TUI top-level state and event loop.
//!
//! The loop is fully async — `crossterm::EventStream` for keystrokes and a
//! `tokio::mpsc` channel for fetch results — so a slow API call never blocks
//! the redraw or input handling. Tab switches kick off a background `tokio::spawn`
//! that posts back when the response lands.
//!
//! Layout:
//! - top: tab bar (resource selector)
//! - middle: left list (60%) + right detail pane (40%) — toggleable to full
//!   detail or card grid
//! - bottom: status line
//!
//! Authored by okooo5km.

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::{
    api::{client::Query, jsonapi::Resource, Client},
    cli::Context,
    error::Result,
    tui::{
        state::{current_view, AppState, FetchResult, LayoutMode, RESOURCES},
        views::{detail as detail_view, home as home_view, list as list_view},
    },
    view::{self, columns::ResourceView},
    Error,
};

pub async fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    ctx: &Context,
) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<FetchResult>();
    let mut events = EventStream::new();
    let default_layout = match ctx.layout() {
        crate::cli::context::LayoutMode::Cards => LayoutMode::Cards,
        crate::cli::context::LayoutMode::Table => LayoutMode::Split,
    };
    let mut state = AppState::new(default_layout);

    state.loading = true;
    state.status = format!("loading {}…", RESOURCES[state.selected_resource].2);
    spawn_fetch(ctx, state.selected_resource, state.fetch_seq, tx.clone());

    loop {
        terminal
            .draw(|f| ui(f, &mut state))
            .map_err(|e| Error::user(format!("tui draw: {e}")))?;

        tokio::select! {
            Some(res) = rx.recv() => {
                apply_fetch(&mut state, res);
            }
            Some(event) = events.next() => {
                let event = event.map_err(|e| Error::user(format!("tui read: {e}")))?;
                if let Event::Key(key) = event {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    let view = current_view(&state);
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('r') => kick_fetch(&mut state, ctx, &tx),
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
                        KeyCode::Char('J') => state.detail_down(view),
                        KeyCode::Char('K') => state.detail_up(view),
                        KeyCode::Enter | KeyCode::Char('d') => {
                            state.layout = match state.layout {
                                LayoutMode::Split => LayoutMode::DetailFull,
                                LayoutMode::DetailFull | LayoutMode::Cards => LayoutMode::Split,
                            };
                        }
                        KeyCode::Char('c') => {
                            state.layout = match state.layout {
                                LayoutMode::Cards => LayoutMode::Split,
                                _ => LayoutMode::Cards,
                            };
                        }
                        KeyCode::Char('y') => yank_selected(&mut state, view),
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

fn yank_selected(state: &mut AppState, view: Option<&'static ResourceView>) {
    let Some(value) = state.selected_value() else {
        state.flash = Some("nothing selected".into());
        return;
    };
    let text = if let Some(rv) = view {
        let pairs = view::detail_pairs(&value, rv.detail, false);
        pairs
            .get(state.detail_cursor)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    } else {
        value
            .pointer("/id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    if text.is_empty() {
        state.flash = Some("empty value, nothing copied".into());
        return;
    }
    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(text.clone())) {
        Ok(()) => {
            state.flash = Some(format!("copied: {}", trim_for_status(&text, 60)));
        }
        Err(e) => state.flash = Some(format!("copy failed: {e}")),
    }
}

fn trim_for_status(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let head: String = s.chars().take(n - 1).collect();
        format!("{head}…")
    }
}

fn kick_fetch(state: &mut AppState, ctx: &Context, tx: &mpsc::UnboundedSender<FetchResult>) {
    state.fetch_seq = state.fetch_seq.wrapping_add(1);
    state.loading = true;
    state.error = None;
    state.rows.clear();
    state.table_state.select(None);
    state.detail_cursor = 0;
    state.status = format!("loading {}…", RESOURCES[state.selected_resource].2);
    spawn_fetch(ctx, state.selected_resource, state.fetch_seq, tx.clone());
}

fn apply_fetch(state: &mut AppState, res: FetchResult) {
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
                .table_state
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

// -------------------- top-level rendering --------------------

fn ui(f: &mut Frame, state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    home_view::draw(f, chunks[0], state);
    draw_body(f, chunks[1], state);
    draw_status(f, chunks[2], state);
}

fn draw_body(f: &mut Frame, area: ratatui::layout::Rect, state: &mut AppState) {
    if let Some(err) = &state.error {
        let p = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("error"));
        f.render_widget(p, area);
        return;
    }

    let view = current_view(state);
    let label = list_view::current_resource_label(state);

    match state.layout {
        LayoutMode::Split => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);
            list_view::draw_table(f, chunks[0], state, view, label);
            detail_view::draw(f, chunks[1], state, view);
        }
        LayoutMode::DetailFull => {
            detail_view::draw(f, area, state, view);
        }
        LayoutMode::Cards => {
            list_view::draw_cards(f, area, state, view, label);
        }
    }
}

fn draw_status(f: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let text = if let Some(flash) = &state.flash {
        flash.clone()
    } else if state.loading {
        format!("⟳ {}", state.status)
    } else {
        state.status.clone()
    };
    let p = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(p, area);
}
