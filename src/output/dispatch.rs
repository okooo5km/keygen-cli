//! Format dispatcher. Resources hand off a `Payload` and we render in whatever
//! format the active [`Context`] selected (table / json / yaml / ndjson / tsv).

use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use serde_json::{json, Value};

use crate::{
    cli::Context,
    error::Result,
    render::{status::Status, time::relative},
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
    match ctx.format() {
        OutputFormat::Json => print_json(payload),
        OutputFormat::Yaml => print_yaml(payload),
        OutputFormat::Ndjson => print_ndjson(payload),
        OutputFormat::Tsv => print_tsv(payload),
        OutputFormat::Table => {
            print_table(payload, ctx.use_color());
            Ok(())
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
                let cols = generic_columns(first);
                println!(
                    "{}",
                    cols.iter().map(|c| c.title).collect::<Vec<_>>().join("\t")
                );
                for item in items {
                    let row = cols
                        .iter()
                        .map(|c| cell_string(item, c, false))
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

fn print_table(payload: &Payload, use_color: bool) {
    match payload {
        Payload::List(items) => {
            if items.is_empty() {
                println!("(no rows)");
                return;
            }
            let cols = generic_columns(&items[0]);
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(cols.iter().map(|c| Cell::new(c.title)));
            for item in items {
                table.add_row(
                    cols.iter()
                        .map(|c| Cell::new(cell_string(item, c, use_color))),
                );
            }
            println!("{table}");
        }
        Payload::Single(v) | Payload::Bag(v) => {
            print_kv_table(v, use_color);
        }
        Payload::WithMeta { data, meta } => {
            print_kv_table(data, use_color);
            if let Some(m) = meta {
                println!();
                println!("meta:");
                print_kv_table(m, use_color);
            }
        }
    }
}

fn print_kv_table(value: &Value, use_color: bool) {
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
            table.add_row(vec![Cell::new(k), Cell::new(format_value(v, k, use_color))]);
        }
    } else if let Some(obj) = value.as_object() {
        for (k, v) in obj {
            if k == "id" || k == "type" {
                continue;
            }
            table.add_row(vec![Cell::new(k), Cell::new(format_value(v, k, use_color))]);
        }
    }
    println!("{table}");
}

#[derive(Debug, Clone)]
struct Column {
    title: &'static str,
    pointer: String,
    kind: ColKind,
}

#[derive(Debug, Clone, Copy)]
enum ColKind {
    Plain,
    Status,
    Time,
}

/// Build a default column set by inspecting the first resource. Pulls common
/// keygen.sh fields when present; falls back to the first few attribute keys.
fn generic_columns(sample: &Value) -> Vec<Column> {
    let mut cols = Vec::new();
    cols.push(Column {
        title: "id",
        pointer: "/id".into(),
        kind: ColKind::Plain,
    });
    if sample.get("type").is_some() {
        cols.push(Column {
            title: "type",
            pointer: "/type".into(),
            kind: ColKind::Plain,
        });
    }
    let attrs = sample.get("attributes").and_then(Value::as_object);
    let known: &[(&str, &str, ColKind)] = &[
        ("name", "/attributes/name", ColKind::Plain),
        ("key", "/attributes/key", ColKind::Plain),
        ("status", "/attributes/status", ColKind::Status),
        ("expiry", "/attributes/expiry", ColKind::Time),
        ("expires", "/attributes/expires", ColKind::Time),
        ("created", "/attributes/created", ColKind::Time),
    ];
    if let Some(attrs) = attrs {
        for (title, pointer, kind) in known {
            let key = pointer.trim_start_matches("/attributes/");
            if attrs.contains_key(key) {
                cols.push(Column {
                    title,
                    pointer: (*pointer).into(),
                    kind: *kind,
                });
            }
        }
    }
    cols
}

fn cell_string(value: &Value, col: &Column, use_color: bool) -> String {
    let raw = value.pointer(&col.pointer).cloned().unwrap_or(Value::Null);
    match col.kind {
        ColKind::Plain => format_value(&raw, col.title, use_color),
        ColKind::Status => match raw.as_str() {
            Some(s) => Status::parse(s).pill(s, use_color),
            None => "—".into(),
        },
        ColKind::Time => match raw.as_str() {
            Some(s) => match s.parse::<jiff::Timestamp>() {
                Ok(ts) => relative(ts),
                Err(_) => s.to_string(),
            },
            None => "—".into(),
        },
    }
}

fn format_value(v: &Value, key: &str, use_color: bool) -> String {
    match v {
        Value::Null => "—".into(),
        Value::Bool(b) => {
            if *b {
                "✓".into()
            } else {
                "—".into()
            }
        }
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            if key == "status" {
                Status::parse(s).pill(s, use_color)
            } else if looks_like_timestamp(s) {
                s.parse::<jiff::Timestamp>()
                    .map_or_else(|_| s.clone(), relative)
            } else if s.len() > 60 {
                format!("{}…", &s[..59])
            } else {
                s.clone()
            }
        }
        Value::Array(arr) => format!("[{} items]", arr.len()),
        Value::Object(obj) => format!("{{{} keys}}", obj.len()),
    }
}

fn looks_like_timestamp(s: &str) -> bool {
    s.len() >= 19
        && s.as_bytes().get(4) == Some(&b'-')
        && s.as_bytes().get(7) == Some(&b'-')
        && s.as_bytes().get(10) == Some(&b'T')
}
