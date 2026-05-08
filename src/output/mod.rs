pub mod dispatch;
pub mod json;
pub mod ndjson;
pub mod table;
pub mod yaml;

pub use dispatch::{bag, emit, list, single, single_with_meta, Payload};

use std::io::IsTerminal;

use crate::error::Error;

/// What `keygen` should print. Resolved from `--output`, `--json`, or — when
/// nothing is supplied — defaults to `Table`.
///
/// `Table` always means a human-friendly table. ANSI colors are disabled
/// automatically when stdout isn't a TTY (or the user passes `--no-color`),
/// but the table layout itself stays — pipes get plain ASCII, not JSON. AI /
/// scripted callers should request JSON explicitly via `--json` (or `--output
/// json`), the same way `gh` does it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Tsv,
    Ndjson,
}

/// Decide whether to emit ANSI colors. Honours `--no-color`, `NO_COLOR=`, and
/// pipe detection on stdout.
pub fn resolve_use_color(no_color_flag: bool) -> bool {
    if no_color_flag {
        return false;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::io::stdout().is_terminal()
}

/// Print a structured error to stderr.
///
/// `want_json` is what the caller passed via `--json` or `--output json`.
/// Resolved at the entry point and threaded down because errors may surface
/// before Context construction succeeds.
pub fn report_error(err: &Error, want_json: bool) {
    if want_json {
        let payload = serde_json::json!({
            "ok": false,
            "error": format_ai(err),
        });
        eprintln!("{}", serde_json::to_string(&payload).unwrap_or_default());
    } else {
        eprintln!("✗ {err}");
    }
}

fn format_ai(err: &Error) -> serde_json::Value {
    match err {
        Error::Api {
            status,
            code,
            title,
            detail,
            pointer,
            request_id,
            hint,
        } => serde_json::json!({
            "kind": "api",
            "http_status": status,
            "code": code,
            "title": title,
            "detail": detail,
            "source": pointer,
            "request_id": request_id,
            "hint": hint,
        }),
        other => serde_json::json!({
            "kind": classify(other),
            "message": other.to_string(),
        }),
    }
}

fn classify(err: &Error) -> &'static str {
    match err {
        Error::User(_) | Error::Serde(_) => "user",
        Error::Auth(_) => "auth",
        Error::Network(_) => "network",
        Error::Capability(_) => "capability",
        Error::Config(_) => "config",
        Error::Io(_) => "io",
        Error::Other(_) => "other",
        Error::Api { .. } => "api",
    }
}
