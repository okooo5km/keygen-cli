//! Card-style renderer — used by `--layout cards` (CLI) and the TUI `c` toggle.
//!
//! Authored by okooo5km(十里).

use std::fmt::Write;

use owo_colors::OwoColorize;
use serde_json::Value;

use super::{
    columns::ResourceView,
    render::{detail_pairs, resolve_pointer},
};

/// Render a single resource as a card block (string with embedded ANSI when
/// `use_color`).
#[must_use]
pub fn card_block(value: &Value, view: &ResourceView, use_color: bool, width: usize) -> String {
    let id = value
        .pointer("/id")
        .and_then(Value::as_str)
        .unwrap_or("(no id)");

    // Headline — first detail field that isn't ID.
    let headline = headline_for(value, view);

    let mut out = String::new();

    // Top rule with the ID highlighted.
    let bar = "─".repeat(width.saturating_sub(2));
    if use_color {
        let _ = writeln!(
            out,
            "{} {} {}",
            "───".dimmed(),
            id.yellow().bold(),
            bar.dimmed()
        );
    } else {
        let _ = writeln!(out, "─── {id} {bar}");
    }
    if !headline.is_empty() {
        if use_color {
            let _ = writeln!(out, "  {}", headline.bold());
        } else {
            let _ = writeln!(out, "  {headline}");
        }
    }
    out.push('\n');

    let label_w = view.detail.iter().map(|f| f.label.len()).max().unwrap_or(8);

    for (label, val) in detail_pairs(value, view.detail, use_color) {
        // Skip ID — already in the headline.
        if label == "id" {
            continue;
        }
        if val == "—" {
            continue;
        }
        if use_color {
            let dim_label = label.dimmed();
            let _ = writeln!(out, "  {dim_label:<label_w$}  {val}");
        } else {
            let _ = writeln!(out, "  {label:<label_w$}  {val}");
        }
    }

    out
}

fn headline_for(value: &Value, view: &ResourceView) -> String {
    // Prefer name | fullName | filename | event | version — whichever shows up
    // first in the column list (excluding ID).
    for col in view.columns {
        if col.title == "id" {
            continue;
        }
        if let Some(v) = resolve_pointer(value, col.pointer) {
            if let Some(s) = v.as_str() {
                return s.to_string();
            }
        }
    }
    String::new()
}
