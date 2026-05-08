//! JSON renderer. Stable schema:
//!
//! - single: `{ "ok": true, "data": <object> }`
//! - list:   `{ "ok": true, "data": [...], "meta": { "page", "limit", "total" } }`

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Envelope<T: Serialize> {
    pub ok: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}

#[derive(Debug, Serialize)]
pub struct Meta {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub total: Option<u64>,
}

pub fn print<T: Serialize>(value: &T) -> crate::error::Result<()> {
    let stdout = std::io::stdout().lock();
    serde_json::to_writer(stdout, value)?;
    println!();
    Ok(())
}
