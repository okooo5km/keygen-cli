//! Cell / detail-pair renderers — turn a JSON value into a string given a
//! `ColKind`. Used by both the CLI table renderer and the TUI list/detail.
//!
//! Authored by okooo5km(十里).

use serde_json::Value;

use crate::render::{status::Status, time::relative};

use super::{
    columns::{ColKind, ColumnDef, DetailField},
    truncate::{format_bytes, tail_chars, truncate_middle},
};

/// Resolve a pointer that may be a `|`-separated list of alternatives.
/// Returns the first non-null value found.
pub fn resolve_pointer<'a>(value: &'a Value, pointer: &str) -> Option<&'a Value> {
    for p in pointer.split('|') {
        if let Some(v) = value.pointer(p) {
            if !v.is_null() {
                return Some(v);
            }
        }
    }
    None
}

/// Render a column cell value for a list row.
#[must_use]
pub fn cell_text(value: &Value, col: &ColumnDef, use_color: bool) -> String {
    let raw = resolve_pointer(value, col.pointer);
    apply_kind(raw, col.kind, col.title, use_color)
}

/// Render a detail field (label-value pair). Object values whose contents are
/// real data (e.g. `metadata`) are flattened into one entry per sub-key so
/// every value is visible in the detail pane / KV table — only relationship
/// refs (`{ "type": ..., "id": ... }`) and empty objects collapse to a single
/// row.
#[must_use]
pub fn detail_pairs(
    value: &Value,
    fields: &[DetailField],
    use_color: bool,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for f in fields {
        let raw = resolve_pointer(value, f.pointer);
        if let Some(Value::Object(obj)) = raw {
            if matches!(f.kind, ColKind::Plain) && !is_type_id_ref(obj) && !obj.is_empty() {
                for (k, v) in obj {
                    let label = format!("{}.{k}", f.label);
                    out.push((label, format_value(v, k, use_color)));
                }
                continue;
            }
        }
        let rendered = apply_kind(raw, f.kind, f.label, use_color);
        out.push((f.label.to_string(), rendered));
    }
    out
}

fn is_type_id_ref(obj: &serde_json::Map<String, Value>) -> bool {
    obj.get("type").and_then(Value::as_str).is_some()
        && obj.get("id").and_then(Value::as_str).is_some()
}

#[allow(clippy::cast_sign_loss)]
fn apply_kind(raw: Option<&Value>, kind: ColKind, hint: &str, use_color: bool) -> String {
    let Some(v) = raw else {
        return "—".into();
    };
    match kind {
        ColKind::Plain => format_value(v, hint, use_color),
        ColKind::Status => match v.as_str() {
            Some(s) => Status::parse(s).pill(s, use_color),
            None => "—".into(),
        },
        ColKind::Time => match v.as_str() {
            Some(s) => match s.parse::<jiff::Timestamp>() {
                Ok(ts) => relative(ts),
                Err(_) => s.to_string(),
            },
            None => "—".into(),
        },
        ColKind::Tail(n) => match v.as_str() {
            Some(s) => tail_chars(s, n),
            None => "—".into(),
        },
        ColKind::Bytes => match v.as_u64() {
            Some(n) => format_bytes(n),
            None => "—".into(),
        },
        ColKind::Bool => match v.as_bool() {
            Some(true) => "✓".into(),
            Some(false) | None => "—".into(),
        },
        ColKind::Count => count_text(v),
        ColKind::Truncate(n) => truncate_middle(&plain_string(v), n),
        ColKind::UrlHost => match v.as_str() {
            Some(s) => url::Url::parse(s)
                .ok()
                .and_then(|u| u.host_str().map(str::to_string))
                .unwrap_or_else(|| truncate_middle(s, 30)),
            None => "—".into(),
        },
    }
}

fn count_text(v: &Value) -> String {
    if let Some(n) = v.pointer("/meta/count").and_then(Value::as_u64) {
        return n.to_string();
    }
    if let Some(arr) = v.pointer("/data").and_then(Value::as_array) {
        return arr.len().to_string();
    }
    if let Some(arr) = v.as_array() {
        return arr.len().to_string();
    }
    if let Some(n) = v.as_u64() {
        return n.to_string();
    }
    "—".into()
}

fn plain_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(a) => format!("[{} items]", a.len()),
        Value::Object(o) => format_object_inline(o),
    }
}

/// Render a JSON object as `k=v, k2=v2` for single-line display
/// (table cells, narrow paragraphs). Nested non-primitives are
/// summarized to keep the output one line.
fn format_object_inline(obj: &serde_json::Map<String, Value>) -> String {
    if obj.is_empty() {
        return "—".into();
    }
    if let (Some(t), Some(id)) = (
        obj.get("type").and_then(Value::as_str),
        obj.get("id").and_then(Value::as_str),
    ) {
        return format!("{t}:{id}");
    }
    obj.iter()
        .map(|(k, v)| format!("{k}={}", inline_scalar(v)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn inline_scalar(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(a) => format!("[{} items]", a.len()),
        Value::Object(o) => format!("{{{} keys}}", o.len()),
    }
}

/// Generic value → string used by detail panes / KV tables.
#[must_use]
pub fn format_value(v: &Value, key: &str, use_color: bool) -> String {
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
            if key.eq_ignore_ascii_case("status") || key.eq_ignore_ascii_case("heartbeatStatus") {
                Status::parse(s).pill(s, use_color)
            } else if looks_like_timestamp(s) {
                s.parse::<jiff::Timestamp>()
                    .map_or_else(|_| s.clone(), relative)
            } else {
                s.clone()
            }
        }
        Value::Array(arr) => {
            // Show up to a few primitives inline; otherwise `[N items]`.
            let primitives: Option<Vec<String>> = arr
                .iter()
                .take(6)
                .map(|x| match x {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    Value::Bool(b) => Some(b.to_string()),
                    _ => None,
                })
                .collect();
            match primitives {
                Some(items) if !items.is_empty() && arr.len() <= 6 => items.join(", "),
                _ => format!("[{} items]", arr.len()),
            }
        }
        Value::Object(obj) => format_object_inline(obj),
    }
}

fn looks_like_timestamp(s: &str) -> bool {
    s.len() >= 19
        && s.as_bytes().get(4) == Some(&b'-')
        && s.as_bytes().get(7) == Some(&b'-')
        && s.as_bytes().get(10) == Some(&b'T')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::columns::DetailField;
    use serde_json::json;

    #[test]
    fn format_value_renders_metadata_inline() {
        let v = json!({ "seat": "enterprise", "plan": "annual" });
        let out = format_value(&v, "metadata", false);
        // Order is preserved by serde_json's Map (BTreeMap-like via feature).
        assert!(out.contains("seat=enterprise"));
        assert!(out.contains("plan=annual"));
        assert!(!out.contains("keys"));
    }

    #[test]
    fn format_value_keeps_relationship_refs_compact() {
        let v = json!({ "type": "licenses", "id": "lic_abc" });
        assert_eq!(format_value(&v, "license", false), "licenses:lic_abc");
    }

    #[test]
    fn format_value_empty_object_is_dash() {
        assert_eq!(format_value(&json!({}), "metadata", false), "—");
    }

    #[test]
    fn detail_pairs_flattens_metadata() {
        let value = json!({
            "id": "lic_1",
            "type": "licenses",
            "attributes": {
                "name": "demo",
                "metadata": { "seat": "pro", "region": "apac" },
            }
        });
        let fields = &[
            DetailField {
                label: "name",
                pointer: "/attributes/name",
                kind: ColKind::Plain,
            },
            DetailField {
                label: "metadata",
                pointer: "/attributes/metadata",
                kind: ColKind::Plain,
            },
        ];
        let pairs = detail_pairs(&value, fields, false);
        // name + 2 expanded metadata.* rows
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0].0, "name");
        assert!(pairs
            .iter()
            .any(|(k, v)| k == "metadata.seat" && v == "pro"));
        assert!(pairs
            .iter()
            .any(|(k, v)| k == "metadata.region" && v == "apac"));
    }

    #[test]
    fn detail_pairs_keeps_empty_metadata_as_single_row() {
        let value = json!({
            "id": "lic_1",
            "type": "licenses",
            "attributes": { "metadata": {} }
        });
        let fields = &[DetailField {
            label: "metadata",
            pointer: "/attributes/metadata",
            kind: ColKind::Plain,
        }];
        let pairs = detail_pairs(&value, fields, false);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "metadata");
        assert_eq!(pairs[0].1, "—");
    }
}
