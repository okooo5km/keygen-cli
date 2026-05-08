//! Right-side detail pane: K/V rendering of the selected row.
//!
//! Authored by okooo5km.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use serde_json::Value;

use crate::{
    tui::state::AppState,
    view::{self, columns::ResourceView},
};

pub fn draw(f: &mut Frame, area: Rect, state: &mut AppState, view: Option<&'static ResourceView>) {
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

pub fn generic_detail_pairs(value: &Value) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    if let Some(id) = value.pointer("/id").and_then(Value::as_str) {
        pairs.push(("id".into(), id.into()));
    }
    if let Some(t) = value.pointer("/type").and_then(Value::as_str) {
        pairs.push(("type".into(), t.into()));
    }
    if let Some(attrs) = value.pointer("/attributes").and_then(Value::as_object) {
        for (k, v) in attrs {
            // Flatten object attributes (e.g. `metadata`) into one row per
            // sub-key so the detail pane shows actual content rather than
            // `{N keys}`. Relationship refs still collapse to `type:id`.
            if let Value::Object(obj) = v {
                let is_ref = obj.get("type").and_then(Value::as_str).is_some()
                    && obj.get("id").and_then(Value::as_str).is_some();
                if !obj.is_empty() && !is_ref {
                    for (sk, sv) in obj {
                        pairs.push((format!("{k}.{sk}"), view::format_value(sv, sk, false)));
                    }
                    continue;
                }
            }
            pairs.push((k.clone(), view::format_value(v, k, false)));
        }
    }
    pairs
}
