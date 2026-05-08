//! Events panel — live tail of webhook events / event-log.
//!
//! Pulls the most recent N rows from `/webhook-events` every few seconds,
//! tracks a cursor (the latest seen `created` timestamp) so subsequent polls
//! only fetch deltas, and surfaces the result through `EventEntry` rows. The
//! widget side renders them; this module is the data layer.
//!
//! Authored by okooo5km.

use serde_json::Value;

use crate::{
    api::{client::Query, jsonapi::Resource, Client},
    cli::Context,
    error::Result,
};

#[derive(Debug, Clone)]
pub struct EventEntry {
    pub id: String,
    pub event_type: String,
    pub created: String,
    pub status: Option<String>,
    /// True until the first redraw after this row first appeared — used by
    /// the renderer to flash the row.
    pub fresh: bool,
}

/// Pull the next page of webhook events, newest first. `since` is the most
/// recent `created` timestamp the caller has already seen (or `None` for the
/// first poll).
pub async fn fetch(ctx: &Context, since: Option<&str>) -> Result<Vec<EventEntry>> {
    let client = Client::new(ctx)?;
    let mut query = Query::new()
        .page(1, if since.is_none() { 50 } else { 10 })
        .sort(Some("-created".to_string()));
    if let Some(cursor) = since {
        query = query.pair("filter[after]", cursor);
    }
    let doc = client
        .get::<Vec<Resource>>("/webhook-events", &query)
        .await?;
    Ok(doc.data.into_iter().map(into_entry).collect())
}

fn into_entry(r: Resource) -> EventEntry {
    let attrs = r.attributes.as_object();
    let event_type = attrs
        .and_then(|m| m.get("event"))
        .and_then(Value::as_str)
        .unwrap_or("(event)")
        .to_string();
    let created = attrs
        .and_then(|m| m.get("created"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let status = attrs
        .and_then(|m| m.get("status"))
        .and_then(Value::as_str)
        .map(str::to_string);
    EventEntry {
        id: r.id,
        event_type,
        created,
        status,
        fresh: true,
    }
}

/// Merge `incoming` (newest-first) into the buffer, keeping at most `cap`
/// rows. Returns `true` when something new was appended (drives the flash
/// timer + status-bar updates).
pub fn merge(buf: &mut Vec<EventEntry>, mut incoming: Vec<EventEntry>, cap: usize) -> bool {
    let mut changed = false;
    incoming.reverse(); // ascending so new rows land at the head as we prepend

    for entry in incoming {
        if buf.iter().any(|e| e.id == entry.id) {
            continue;
        }
        buf.insert(0, entry);
        changed = true;
    }

    if buf.len() > cap {
        buf.truncate(cap);
    }
    changed
}

pub fn latest_cursor(buf: &[EventEntry]) -> Option<String> {
    buf.iter()
        .map(|e| &e.created)
        .filter(|c| !c.is_empty())
        .max()
        .cloned()
}
