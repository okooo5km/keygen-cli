pub mod json;
pub mod ndjson;
pub mod table;
pub mod yaml;

use std::io::IsTerminal;

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Human,
    Ai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Tsv,
    Ndjson,
}

#[derive(Debug, Clone, Copy)]
pub struct ModeInputs {
    pub ai_flag: bool,
    pub human_flag: bool,
    pub no_color: bool,
}

impl Mode {
    pub fn resolve(input: ModeInputs) -> Self {
        if input.ai_flag {
            return Self::Ai;
        }
        if input.human_flag {
            return Self::Human;
        }
        if std::env::var_os("CI").is_some() {
            return Self::Ai;
        }
        if std::io::stdout().is_terminal() {
            Self::Human
        } else {
            Self::Ai
        }
    }

    pub fn default_format(self) -> OutputFormat {
        match self {
            Self::Human => OutputFormat::Table,
            Self::Ai => OutputFormat::Json,
        }
    }

    pub fn use_color(self) -> bool {
        matches!(self, Self::Human)
    }
}

/// Print a structured error to stderr in the format dictated by the active
/// mode. AI mode emits JSON `{ ok:false, error:{...} }`; human mode emits a
/// pretty miette-style diagnostic.
pub fn report_error(err: &Error) {
    let mode = Mode::resolve(ModeInputs {
        ai_flag: std::env::var_os("KEYGEN_AI").is_some(),
        human_flag: false,
        no_color: false,
    });
    match mode {
        Mode::Ai => {
            let payload = serde_json::json!({
                "ok": false,
                "error": format_ai(err),
            });
            eprintln!("{}", serde_json::to_string(&payload).unwrap_or_default());
        }
        Mode::Human => {
            eprintln!("✗ {err}");
        }
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
