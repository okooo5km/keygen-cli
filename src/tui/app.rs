//! TUI top-level state and event loop.
//!
//! The loop is fully async — `crossterm::EventStream` for keystrokes and a
//! `tokio::mpsc` channel for background work (list fetches and action
//! executions) — so a slow API call never blocks the redraw or input
//! handling.
//!
//! Input is dispatched by mode, in priority order:
//! 1. confirm overlay (Tier 2/3 mutating action waiting for y/n);
//! 2. action menu (`a` opened the catalogue for the current row);
//! 3. browsing (default — tabs / arrows / d / c / y / r / q).
//!
//! Authored by okooo5km.

use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use serde_json::Value;
use tokio::{sync::mpsc, time::MissedTickBehavior};

use crate::{
    api::{client::Query, jsonapi::Resource, Client},
    cli::Context,
    error::Result,
    tui::{
        state::{current_view, ActionDone, AppMsg, AppState, FetchResult, LayoutMode, RESOURCES},
        views::{
            actions::{actions_for, resource_base_path, Action, HttpMethod},
            detail as detail_view,
            events::{self as events_view, EventEntry},
            home as home_view, list as list_view,
        },
        widgets::{
            action_menu::{self, ActionMenuState},
            confirm::{self, ConfirmFeedback, ConfirmState},
            log_viewer,
        },
    },
    view::{self, columns::ResourceView},
    Error,
};

pub async fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    ctx: &Context,
) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<AppMsg>();
    let mut events = EventStream::new();
    let default_layout = match ctx.layout() {
        crate::cli::context::LayoutMode::Cards => LayoutMode::Cards,
        crate::cli::context::LayoutMode::Table => LayoutMode::Split,
    };
    let mut state = AppState::new(default_layout);

    state.loading = true;
    state.status = format!("loading {}…", RESOURCES[state.selected_resource].2);
    spawn_fetch(ctx, state.selected_resource, state.fetch_seq, tx.clone());
    spawn_events_fetch(ctx, state.events_cursor.clone(), tx.clone());
    state.events_fetching = true;

    let mut events_tick = tokio::time::interval(Duration::from_secs(5));
    events_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    events_tick.tick().await; // discard the first immediate tick — we already kicked one off above

    loop {
        terminal
            .draw(|f| ui(f, &mut state))
            .map_err(|e| Error::user(format!("tui draw: {e}")))?;
        clear_event_freshness(&mut state);

        tokio::select! {
            Some(msg) = rx.recv() => match msg {
                AppMsg::Fetch(res) => apply_fetch(&mut state, res),
                AppMsg::Action(res) => apply_action(&mut state, res),
                AppMsg::Events(res) => apply_events(&mut state, res),
            },
            _ = events_tick.tick() => {
                if !state.events_fetching {
                    state.events_fetching = true;
                    spawn_events_fetch(ctx, state.events_cursor.clone(), tx.clone());
                }
            }
            Some(event) = events.next() => {
                let event = event.map_err(|e| Error::user(format!("tui read: {e}")))?;
                if let Event::Key(key) = event {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    if state.confirm.is_some() {
                        if handle_confirm_key(&mut state, ctx, &tx, key.code) {
                            break;
                        }
                    } else if state.action_menu.is_some() {
                        if handle_menu_key(&mut state, ctx, key.code) {
                            break;
                        }
                    } else if handle_browsing_key(&mut state, ctx, &tx, key.code) {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Returns `true` to request the event loop to break.
fn handle_browsing_key(
    state: &mut AppState,
    ctx: &Context,
    tx: &mpsc::UnboundedSender<AppMsg>,
    code: KeyCode,
) -> bool {
    let view = current_view(state);
    match code {
        KeyCode::Char('q') | KeyCode::Esc => return true,
        KeyCode::Char('r') => kick_fetch(state, ctx, tx),
        KeyCode::Tab | KeyCode::Right => {
            state.move_right();
            kick_fetch(state, ctx, tx);
        }
        KeyCode::BackTab | KeyCode::Left => {
            state.move_left();
            kick_fetch(state, ctx, tx);
        }
        KeyCode::Down | KeyCode::Char('j') => state.move_down(),
        KeyCode::Up | KeyCode::Char('k') => state.move_up(),
        KeyCode::Char('J') => state.detail_down(view),
        KeyCode::Char('K') => state.detail_up(view),
        KeyCode::Enter | KeyCode::Char('d') => {
            state.layout = match state.layout {
                LayoutMode::Split => LayoutMode::DetailFull,
                LayoutMode::DetailFull | LayoutMode::Cards | LayoutMode::EventsFull => {
                    LayoutMode::Split
                }
            };
        }
        KeyCode::Char('c') => {
            state.layout = match state.layout {
                LayoutMode::Cards => LayoutMode::Split,
                _ => LayoutMode::Cards,
            };
        }
        KeyCode::Char('e') => {
            state.layout = match state.layout {
                LayoutMode::EventsFull => LayoutMode::Split,
                _ => LayoutMode::EventsFull,
            };
        }
        KeyCode::Char('y') => yank_selected(state, view),
        KeyCode::Char('a') => open_action_menu(state),
        _ => {}
    }
    false
}

fn handle_menu_key(state: &mut AppState, ctx: &Context, code: KeyCode) -> bool {
    let Some(menu) = state.action_menu.as_mut() else {
        return false;
    };
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            state.action_menu = None;
        }
        KeyCode::Down | KeyCode::Char('j') => menu.move_down(),
        KeyCode::Up | KeyCode::Char('k') => menu.move_up(),
        KeyCode::Enter => {
            if let Some(action) = menu.selected().copied() {
                let id = menu.resource_id.clone();
                let jsonapi_type = menu.jsonapi_type;
                state.action_menu = None;
                begin_confirm(state, ctx, jsonapi_type, &id, action);
            }
        }
        KeyCode::Char(ch) if menu.jump_to_key(ch) => {
            if let Some(action) = menu.selected().copied() {
                let id = menu.resource_id.clone();
                let jsonapi_type = menu.jsonapi_type;
                state.action_menu = None;
                begin_confirm(state, ctx, jsonapi_type, &id, action);
            }
        }
        _ => {}
    }
    false
}

fn handle_confirm_key(
    state: &mut AppState,
    ctx: &Context,
    tx: &mpsc::UnboundedSender<AppMsg>,
    code: KeyCode,
) -> bool {
    let Some(c) = state.confirm.as_mut() else {
        return false;
    };
    if c.in_flight {
        return false;
    }
    match code {
        KeyCode::Char('y') => {
            if c.feedback.is_some() {
                state.confirm = None;
            } else {
                c.in_flight = true;
                spawn_action(
                    ctx,
                    c.action_label,
                    c.method,
                    c.path.clone(),
                    c.body.clone(),
                    tx.clone(),
                );
            }
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            state.confirm = None;
        }
        KeyCode::Enter if c.feedback.is_some() => {
            state.confirm = None;
        }
        _ => {}
    }
    false
}

fn open_action_menu(state: &mut AppState) {
    let jsonapi_type = state.jsonapi_type();
    let Some(actions) = actions_for(jsonapi_type) else {
        state.flash = Some(format!("no actions registered for `{jsonapi_type}`"));
        return;
    };
    let Some(id) = state.selected_id() else {
        state.flash = Some("select a row first".into());
        return;
    };
    state.action_menu = Some(ActionMenuState::new(jsonapi_type, id, actions));
}

fn begin_confirm(
    state: &mut AppState,
    ctx: &Context,
    jsonapi_type: &'static str,
    id: &str,
    action: Action,
) {
    let Some(base) = resource_base_path(jsonapi_type) else {
        state.flash = Some(format!("no base path for `{jsonapi_type}`"));
        return;
    };
    let path = crate::tui::views::actions::action_path(base, id, action.path_suffix);
    let body = action.body.to_value();

    let envelope = match Client::new(ctx) {
        Ok(client) => client.with_dry_run(true).dry_run_envelope(
            &action.method.to_reqwest(),
            &path,
            &Query::new(),
            Some(&body),
        ),
        Err(e) => Err(e),
    };

    match envelope {
        Ok(env) => {
            let pretty = serde_json::to_string_pretty(&env).unwrap_or_default();
            state.confirm = Some(ConfirmState::new(
                action.label,
                action.tier,
                action.method,
                path,
                body,
                pretty,
            ));
        }
        Err(e) => {
            state.flash = Some(format!("dry-run preview failed: {e}"));
        }
    }
}

fn spawn_action(
    ctx: &Context,
    label: &'static str,
    method: HttpMethod,
    path: String,
    body: Value,
    tx: mpsc::UnboundedSender<AppMsg>,
) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let payload = match Client::new(&ctx) {
            Ok(client) => match method {
                HttpMethod::Post => client
                    .post::<Value, crate::api::jsonapi::Resource>(&path, &body)
                    .await
                    .map(|d| serde_json::to_value(&d.data).unwrap_or(Value::Null))
                    .map_err(|e| e.to_string()),
                HttpMethod::Delete => client
                    .delete(&path)
                    .await
                    .map(|()| Value::Object(serde_json::Map::new()))
                    .map_err(|e| e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        };
        let _ = tx.send(AppMsg::Action(ActionDone { label, payload }));
    });
}

fn apply_action(state: &mut AppState, res: ActionDone) {
    let Some(c) = state.confirm.as_mut() else {
        return;
    };
    c.in_flight = false;
    match res.payload {
        Ok(_) => {
            c.feedback = Some(ConfirmFeedback::Ok(format!("{} executed", res.label)));
            state.flash = Some(format!("✓ {} executed", res.label));
        }
        Err(msg) => {
            c.feedback = Some(ConfirmFeedback::Err(msg));
        }
    }
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

fn kick_fetch(state: &mut AppState, ctx: &Context, tx: &mpsc::UnboundedSender<AppMsg>) {
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

fn spawn_fetch(ctx: &Context, resource_idx: usize, seq: u64, tx: mpsc::UnboundedSender<AppMsg>) {
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
        let _ = tx.send(AppMsg::Fetch(FetchResult {
            seq,
            resource_idx,
            payload,
        }));
    });
}

fn spawn_events_fetch(ctx: &Context, since: Option<String>, tx: mpsc::UnboundedSender<AppMsg>) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let result = events_view::fetch(&ctx, since.as_deref())
            .await
            .map_err(|e| e.to_string());
        let _ = tx.send(AppMsg::Events(result));
    });
}

fn apply_events(state: &mut AppState, res: std::result::Result<Vec<EventEntry>, String>) {
    state.events_fetching = false;
    match res {
        Ok(rows) => {
            state.events_error = None;
            let added = events_view::merge(&mut state.events, rows, 200);
            if added {
                state.events_cursor = events_view::latest_cursor(&state.events);
            }
        }
        Err(msg) => {
            state.events_error = Some(msg);
        }
    }
}

fn clear_event_freshness(state: &mut AppState) {
    for e in &mut state.events {
        if e.fresh {
            e.fresh = false;
        }
    }
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

    if let Some(menu) = &state.action_menu {
        action_menu::draw(f, f.area(), menu);
    }
    if let Some(c) = &state.confirm {
        confirm::draw(f, f.area(), c);
    }
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
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);
            list_view::draw_table(f, cols[0], state, view, label);

            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
                .split(cols[1]);
            detail_view::draw(f, right[0], state, view);
            log_viewer::draw(
                f,
                right[1],
                &state.events,
                false,
                state.events_error.as_deref(),
            );
        }
        LayoutMode::DetailFull => {
            detail_view::draw(f, area, state, view);
        }
        LayoutMode::Cards => {
            list_view::draw_cards(f, area, state, view, label);
        }
        LayoutMode::EventsFull => {
            log_viewer::draw(f, area, &state.events, true, state.events_error.as_deref());
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
