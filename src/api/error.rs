use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct ErrorDoc {
    pub errors: Vec<ApiErrorEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorEntry {
    pub title: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub source: Option<ErrorSource>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorSource {
    #[serde(default)]
    pub pointer: Option<String>,
    #[serde(default)]
    pub parameter: Option<String>,
}

/// Convert a JSON:API error document + HTTP status into our typed [`Error`].
/// Adds a best-effort `hint` for known error codes so AI agents can self-recover.
pub fn map_error(status: u16, request_id: Option<String>, body: &[u8]) -> Error {
    let parsed: serde_json::Result<ErrorDoc> = serde_json::from_slice(body);
    let entry = parsed.ok().and_then(|d| d.errors.into_iter().next());
    let (code, title, detail, pointer) = match entry {
        Some(e) => (e.code, e.title, e.detail, e.source.and_then(|s| s.pointer)),
        None => (None, format!("HTTP {status}"), None, None),
    };
    let hint = code.as_deref().and_then(hint_for_code).map(str::to_string);
    Error::Api {
        status,
        code,
        title,
        detail,
        pointer,
        request_id,
        hint,
    }
}

fn hint_for_code(code: &str) -> Option<&'static str> {
    crate::explain::lookup(code).and_then(|entry| entry.fix.first().copied())
}
