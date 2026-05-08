//! Format dispatcher. Resources hand off a `Payload` and we render in whatever
//! format the active [`Context`] selected (table / json / yaml / ndjson / tsv).
//!
//! For human-friendly output we consult the per-resource view in
//! `crate::view` so list rows and detail panes stay consistent across the CLI
//! and the TUI.

use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use serde_json::{json, Value};

use crate::{
    cli::{context::LayoutMode, Context},
    error::Result,
    view::{
        self,
        cards::card_block,
        columns::{ColumnWidth, ResourceView},
        view_for_jsonapi_type, ColKind, ColumnDef,
    },
};

use super::OutputFormat;

/// Helper: serialize a single value and emit it.
pub fn single<T: serde::Serialize>(ctx: &Context, data: T) -> Result<()> {
    let v = serde_json::to_value(&data)?;
    emit(ctx, Payload::Single(v))
}

/// Helper: emit a single value alongside JSON:API `meta` (e.g. license
/// validate's `meta.valid` / `meta.code`).
pub fn single_with_meta<T: serde::Serialize>(
    ctx: &Context,
    data: T,
    meta: Option<Value>,
) -> Result<()> {
    let v = serde_json::to_value(&data)?;
    emit(ctx, Payload::WithMeta { data: v, meta })
}

/// Helper: serialize a slice / vec and emit as a list.
pub fn list<T: serde::Serialize>(ctx: &Context, data: &[T]) -> Result<()> {
    let items: Vec<Value> = data
        .iter()
        .map(serde_json::to_value)
        .collect::<std::result::Result<_, _>>()?;
    emit(ctx, Payload::List(items))
}

/// Helper: emit a free-form bag value.
pub fn bag(ctx: &Context, value: Value) -> Result<()> {
    emit(ctx, Payload::Bag(value))
}

/// What the resource handler wants to emit. The dispatcher decides how.
pub enum Payload {
    /// A single resource (typically a JSON:API resource object).
    Single(Value),
    /// A list of resources.
    List(Vec<Value>),
    /// A free-form JSON object (e.g. `{ "deleted": "abc" }`).
    Bag(Value),
    /// A resource alongside the response document's `meta` (used by
    /// validate-style endpoints where the verdict lives in `meta`).
    WithMeta { data: Value, meta: Option<Value> },
}

/// Top-level emit. Wraps payload in the canonical `{ ok, data, meta? }` envelope
/// for AI mode, or pretty-prints for human mode. Quiet mode collapses to a bare
/// id when one is available.
#[allow(clippy::needless_pass_by_value)]
pub fn emit(ctx: &Context, payload: Payload) -> Result<()> {
    emit_ref(ctx, &payload)
}

pub fn emit_ref(ctx: &Context, payload: &Payload) -> Result<()> {
    if ctx.inner.dry_run {
        return Ok(()); // Already printed by the client.
    }

    if ctx.quiet() {
        print_quiet(payload);
        return Ok(());
    }

    match ctx.format() {
        OutputFormat::Json => print_json(payload),
        OutputFormat::Yaml => print_yaml(payload),
        OutputFormat::Ndjson => print_ndjson(payload),
        OutputFormat::Tsv => print_tsv(payload),
        OutputFormat::Table => {
            print_human(payload, ctx.use_color(), ctx.layout());
            Ok(())
        }
    }
}

/// Quiet mode (`-q`): only emit the primary identifier(s). For a list, one
/// id per line; for a single resource, just its id; for `WithMeta`, the
/// resource's id (verdict still surfaces via exit code).
fn print_quiet(payload: &Payload) {
    match payload {
        Payload::List(items) => {
            for item in items {
                if let Some(id) = item.pointer("/id").and_then(Value::as_str) {
                    println!("{id}");
                }
            }
        }
        Payload::Single(v) | Payload::WithMeta { data: v, .. } => {
            if let Some(id) = v.pointer("/id").and_then(Value::as_str) {
                println!("{id}");
            }
        }
        Payload::Bag(v) => {
            if let Some(id) = v.pointer("/id").and_then(Value::as_str) {
                println!("{id}");
            } else if let Some(obj) = v.as_object() {
                if let Some(s) = obj.values().find_map(Value::as_str) {
                    println!("{s}");
                }
            }
        }
    }
}

fn envelope(payload: &Payload) -> Value {
    match payload {
        Payload::Single(v) | Payload::Bag(v) => json!({ "ok": true, "data": v }),
        Payload::List(items) => json!({ "ok": true, "data": items }),
        Payload::WithMeta { data, meta } => match meta {
            Some(m) => json!({ "ok": true, "data": data, "meta": m }),
            None => json!({ "ok": true, "data": data }),
        },
    }
}

fn print_json(payload: &Payload) -> Result<()> {
    let env = envelope(payload);
    println!("{}", serde_json::to_string(&env)?);
    Ok(())
}

fn print_yaml(payload: &Payload) -> Result<()> {
    let env = envelope(payload);
    print!(
        "{}",
        serde_yaml_ng::to_string(&env).map_err(|e| crate::Error::Serde(e.to_string()))?
    );
    Ok(())
}

fn print_ndjson(payload: &Payload) -> Result<()> {
    match payload {
        Payload::List(items) => {
            for item in items {
                println!("{}", serde_json::to_string(item)?);
            }
        }
        Payload::Single(v) | Payload::Bag(v) => {
            println!("{}", serde_json::to_string(v)?);
        }
        Payload::WithMeta { data, .. } => {
            println!("{}", serde_json::to_string(data)?);
        }
    }
    Ok(())
}

fn print_tsv(payload: &Payload) -> Result<()> {
    match payload {
        Payload::List(items) => {
            if let Some(first) = items.first() {
                let view = jsonapi_view(first);
                let cols = columns_for(view, first);
                println!(
                    "{}",
                    cols.iter().map(|c| c.title).collect::<Vec<_>>().join("\t")
                );
                for item in items {
                    let row = cols
                        .iter()
                        .map(|c| view::cell_text(item, c, false))
                        .collect::<Vec<_>>();
                    println!("{}", row.join("\t"));
                }
            }
        }
        Payload::Single(v) | Payload::Bag(v) => {
            println!("{}", serde_json::to_string(v)?);
        }
        Payload::WithMeta { data, .. } => {
            println!("{}", serde_json::to_string(data)?);
        }
    }
    Ok(())
}

fn print_human(payload: &Payload, use_color: bool, layout: LayoutMode) {
    match payload {
        Payload::List(items) => {
            if items.is_empty() {
                println!("(no rows)");
                return;
            }
            let view = jsonapi_view(&items[0]);
            if matches!(layout, LayoutMode::Cards) {
                print_cards(items, view, use_color);
            } else {
                print_table_rows(items, view, use_color);
            }
        }
        Payload::Single(v) | Payload::Bag(v) => {
            print_kv(v, use_color);
        }
        Payload::WithMeta { data, meta } => {
            print_kv(data, use_color);
            if let Some(m) = meta {
                println!();
                println!("meta:");
                print_kv_value(m, use_color);
            }
        }
    }
}

fn print_table_rows(items: &[Value], view: Option<&'static ResourceView>, use_color: bool) {
    let cols = columns_for(view, items.first().unwrap_or(&Value::Null));
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(cols.iter().map(|c| Cell::new(c.title)));
    for item in items {
        table.add_row(
            cols.iter()
                .map(|c| Cell::new(view::cell_text(item, c, use_color))),
        );
    }
    println!("{table}");
}

fn print_cards(items: &[Value], view: Option<&'static ResourceView>, use_color: bool) {
    let width = term_width();
    if let Some(rv) = view {
        for item in items {
            print!("{}", card_block(item, rv, use_color, width));
            println!();
        }
    } else {
        // No view registered for this type — fall back to per-row KV table.
        for item in items {
            print_kv(item, use_color);
            println!();
        }
    }
}

fn print_kv(v: &Value, use_color: bool) {
    if let Some(rv) = jsonapi_view(v) {
        let pairs = view::detail_pairs(v, rv.detail, use_color);
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![Cell::new("Field"), Cell::new("Value")]);
        for (k, val) in pairs {
            table.add_row(vec![Cell::new(k), Cell::new(val)]);
        }
        println!("{table}");
    } else {
        print_kv_value(v, use_color);
    }
}

fn print_kv_value(value: &Value, use_color: bool) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![Cell::new("Field"), Cell::new("Value")]);
    if let Some(id) = value.get("id").and_then(Value::as_str) {
        table.add_row(vec![Cell::new("id"), Cell::new(id)]);
    }
    if let Some(t) = value.get("type").and_then(Value::as_str) {
        table.add_row(vec![Cell::new("type"), Cell::new(t)]);
    }
    if let Some(attrs) = value.get("attributes").and_then(Value::as_object) {
        for (k, v) in attrs {
            push_attr_rows(&mut table, k, v, use_color);
        }
    } else if let Some(obj) = value.as_object() {
        for (k, v) in obj {
            if k == "id" || k == "type" {
                continue;
            }
            push_attr_rows(&mut table, k, v, use_color);
        }
    }
    println!("{table}");
}

/// Add either a single row, or one row per sub-key when the value is a
/// non-empty object (e.g. `metadata`). Relationship refs still collapse to
/// `type:id`.
fn push_attr_rows(table: &mut Table, key: &str, v: &Value, use_color: bool) {
    if let Value::Object(obj) = v {
        let is_ref = obj.get("type").and_then(Value::as_str).is_some()
            && obj.get("id").and_then(Value::as_str).is_some();
        if !obj.is_empty() && !is_ref {
            for (sk, sv) in obj {
                table.add_row(vec![
                    Cell::new(format!("{key}.{sk}")),
                    Cell::new(view::format_value(sv, sk, use_color)),
                ]);
            }
            return;
        }
    }
    table.add_row(vec![
        Cell::new(key),
        Cell::new(view::format_value(v, key, use_color)),
    ]);
}

fn jsonapi_view(v: &Value) -> Option<&'static ResourceView> {
    v.get("type")
        .and_then(Value::as_str)
        .and_then(view_for_jsonapi_type)
}

/// Use the per-resource columns when known; otherwise build a generic set.
fn columns_for(view: Option<&'static ResourceView>, sample: &Value) -> Vec<ColumnDef> {
    if let Some(rv) = view {
        return rv.columns.to_vec();
    }
    generic_columns(sample)
}

fn generic_columns(sample: &Value) -> Vec<ColumnDef> {
    let mut cols: Vec<ColumnDef> = Vec::new();
    cols.push(ColumnDef {
        title: "id",
        pointer: "/id",
        width: ColumnWidth::Fixed(36),
        kind: ColKind::Plain,
        no_wrap: true,
    });
    if sample.get("type").is_some() {
        cols.push(ColumnDef {
            title: "type",
            pointer: "/type",
            width: ColumnWidth::Min(10),
            kind: ColKind::Plain,
            no_wrap: false,
        });
    }
    let attrs = sample.get("attributes").and_then(Value::as_object);
    let known: &[(&str, &str, ColKind)] = &[
        ("name", "/attributes/name", ColKind::Plain),
        ("key", "/attributes/key", ColKind::Plain),
        ("status", "/attributes/status", ColKind::Status),
        ("expiry", "/attributes/expiry", ColKind::Time),
        ("created", "/attributes/created", ColKind::Time),
    ];
    if let Some(attrs) = attrs {
        for (title, pointer, kind) in known {
            let key = pointer.trim_start_matches("/attributes/");
            if attrs.contains_key(key) {
                cols.push(ColumnDef {
                    title,
                    pointer,
                    width: ColumnWidth::Min(12),
                    kind: *kind,
                    no_wrap: false,
                });
            }
        }
    }
    cols
}

fn term_width() -> usize {
    crossterm::terminal::size().map_or(100, |(c, _)| c as usize)
}
