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
//! Authored by okooo5km(十里).

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell as TCell, List, ListItem, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::{
    api::{client::Query, jsonapi::Resource, Client},
    cli::Context,
    error::Result,
    view::{self, cards::card_block, columns::ColumnWidth, columns::ResourceView, ColumnDef},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutMode {
    Split,
    DetailFull,
    Cards,
}

struct AppState {
    selected_resource: usize,
    table_state: TableState,
    rows: Vec<Resource>,
    error: Option<String>,
    loading: bool,
    status: String,
    layout: LayoutMode,
    detail_cursor: usize,
    flash: Option<String>,
    fetch_seq: u64,
}

impl AppState {
    fn new(default_layout: LayoutMode) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self {
            selected_resource: 0,
            table_state: state,
            rows: Vec::new(),
            error: None,
            loading: false,
            status: "Ready. Tab/Shift-Tab switch resource. d=detail c=cards y=yank q=quit".into(),
            layout: default_layout,
            detail_cursor: 0,
            flash: None,
            fetch_seq: 0,
        }
    }

    fn selected_value(&self) -> Option<Value> {
        let i = self.table_state.selected()?;
        self.rows.get(i).map(resource_to_value)
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
            .table_state
            .selected()
            .map_or(0, |i| (i + 1) % self.rows.len());
        self.table_state.select(Some(i));
        self.detail_cursor = 0;
    }

    fn move_up(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let i =
            self.table_state
                .selected()
                .map_or(0, |i| if i == 0 { self.rows.len() - 1 } else { i - 1 });
        self.table_state.select(Some(i));
        self.detail_cursor = 0;
    }

    fn detail_down(&mut self, view: Option<&'static ResourceView>) {
        if let Some(rv) = view {
            if rv.detail.is_empty() {
                return;
            }
            self.detail_cursor = (self.detail_cursor + 1) % rv.detail.len();
        }
    }

    fn detail_up(&mut self, view: Option<&'static ResourceView>) {
        if let Some(rv) = view {
            if rv.detail.is_empty() {
                return;
            }
            self.detail_cursor = if self.detail_cursor == 0 {
                rv.detail.len() - 1
            } else {
                self.detail_cursor - 1
            };
        }
    }
}

fn resource_to_value(r: &Resource) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("id".into(), Value::String(r.id.clone()));
    obj.insert("type".into(), Value::String(r.r#type.clone()));
    obj.insert("attributes".into(), r.attributes.clone());
    if let Some(rels) = &r.relationships {
        obj.insert("relationships".into(), rels.clone());
    }
    Value::Object(obj)
}

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

fn current_view(state: &AppState) -> Option<&'static ResourceView> {
    let t = RESOURCES[state.selected_resource].1;
    crate::view::view_for_jsonapi_type(t)
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

// -------------------- rendering --------------------

fn ui(f: &mut Frame, state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_tabs(f, chunks[0], state);
    draw_body(f, chunks[1], state);
    draw_status(f, chunks[2], state);
}

fn draw_tabs(f: &mut Frame, area: Rect, state: &AppState) {
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

fn draw_body(f: &mut Frame, area: Rect, state: &mut AppState) {
    if let Some(err) = &state.error {
        let p = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("error"));
        f.render_widget(p, area);
        return;
    }

    let view = current_view(state);
    let label = RESOURCES[state.selected_resource].0;

    match state.layout {
        LayoutMode::Split => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);
            draw_table(f, chunks[0], state, view, label);
            draw_detail(f, chunks[1], state, view);
        }
        LayoutMode::DetailFull => {
            draw_detail(f, area, state, view);
        }
        LayoutMode::Cards => {
            draw_cards(f, area, state, view, label);
        }
    }
}

fn draw_table(
    f: &mut Frame,
    area: Rect,
    state: &mut AppState,
    view: Option<&'static ResourceView>,
    label: &str,
) {
    let cols: Vec<ColumnDef> = view.map_or_else(
        || crate::view::columns::VIEWS[0].columns.to_vec(),
        |v| v.columns.to_vec(),
    );

    let header = Row::new(cols.iter().map(|c| {
        TCell::from(c.title).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    }));

    let rows: Vec<Row> = state
        .rows
        .iter()
        .map(|r| {
            let v = resource_to_value(r);
            let cells: Vec<TCell> = cols
                .iter()
                .map(|c| {
                    let text = view::cell_text(&v, c, false);
                    let style = column_style(c.title);
                    TCell::from(text).style(style)
                })
                .collect();
            Row::new(cells)
        })
        .collect();

    let widths: Vec<Constraint> = cols.iter().map(|c| width_to_constraint(c.width)).collect();

    let title = format!(
        " {label} ({n}) — d=detail c=cards r=refresh q=quit ",
        n = state.rows.len(),
    );
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(table, area, &mut state.table_state);
}

fn width_to_constraint(w: ColumnWidth) -> Constraint {
    match w {
        ColumnWidth::Fixed(n) => Constraint::Length(n),
        ColumnWidth::Min(n) => Constraint::Min(n),
        ColumnWidth::Pct(n) => Constraint::Percentage(n),
    }
}

fn column_style(title: &str) -> Style {
    match title {
        "id" => Style::default().fg(Color::Yellow),
        "created" | "expiry" | "lastHeartbeat" => Style::default().fg(Color::DarkGray),
        _ => Style::default(),
    }
}

fn draw_detail(
    f: &mut Frame,
    area: Rect,
    state: &mut AppState,
    view: Option<&'static ResourceView>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Detail (J/K scroll · y yank) ");
    let Some(value) = state.selected_value() else {
        let p = Paragraph::new("(no row selected)")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(p, area);
        return;
    };

    let pairs = match view {
        Some(rv) => view::detail_pairs(&value, rv.detail, false),
        None => generic_detail_pairs(&value),
    };

    let label_w = pairs
        .iter()
        .map(|(k, _)| k.chars().count())
        .max()
        .unwrap_or(8);

    let items: Vec<ListItem> = pairs
        .iter()
        .enumerate()
        .map(|(i, (k, v))| {
            let key_style = if i == state.detail_cursor {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let val_style = match k.as_str() {
                "id" => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::White),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {k:<label_w$}  "), key_style),
                Span::styled(v.clone(), val_style),
            ]))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn generic_detail_pairs(value: &Value) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    if let Some(id) = value.pointer("/id").and_then(Value::as_str) {
        pairs.push(("id".into(), id.into()));
    }
    if let Some(t) = value.pointer("/type").and_then(Value::as_str) {
        pairs.push(("type".into(), t.into()));
    }
    if let Some(attrs) = value.pointer("/attributes").and_then(Value::as_object) {
        for (k, v) in attrs {
            pairs.push((k.clone(), view::format_value(v, k, false)));
        }
    }
    pairs
}

fn draw_cards(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    view: Option<&'static ResourceView>,
    label: &str,
) {
    let title = format!(
        " {label} cards (n={n}) — c=table r=refresh q=quit ",
        n = state.rows.len(),
    );
    let block = Block::default().borders(Borders::ALL).title(title);

    let Some(rv) = view else {
        let p = Paragraph::new("(card view unavailable for this resource)")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(p, area);
        return;
    };

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut text = String::new();
    let inner_w = inner.width as usize;
    for r in &state.rows {
        let v = resource_to_value(r);
        text.push_str(&card_block(&v, rv, false, inner_w));
        text.push('\n');
    }
    let p = Paragraph::new(text);
    f.render_widget(p, inner);
}

fn draw_status(f: &mut Frame, area: Rect, state: &AppState) {
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
