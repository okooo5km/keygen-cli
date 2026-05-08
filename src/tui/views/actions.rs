//! Static action catalogue for the TUI action panel.
//!
//! Each entry maps a non-CRUD verb (validate, suspend, ping, ...) to the HTTP
//! method, path template, and request body the API expects. The panel renders
//! the list, the event loop runs the selected action against
//! `crate::api::Client`, and the same `Tier` value gates whether to require a
//! `--dry-run` preview before going live.
//!
//! Authored by okooo5km.

use reqwest::Method;
use serde_json::{json, Value};

use crate::tui::permission::Tier;

#[derive(Debug, Clone, Copy)]
pub struct Action {
    /// Single-character shortcut shown in the panel.
    pub key: char,
    /// Label rendered in the menu.
    pub label: &'static str,
    /// Permission tier — drives whether the confirm overlay is required.
    pub tier: Tier,
    /// HTTP method this action issues.
    pub method: HttpMethod,
    /// Path suffix appended to the resource's instance URL.
    /// `""` for a plain instance call (e.g. DELETE /licenses/{id}),
    /// `"/actions/suspend"` for a JSON:API verb endpoint.
    pub path_suffix: &'static str,
    /// Body kind — either an empty `{}` body or one of the documented payloads.
    pub body: ActionBody,
    /// One-line hint shown beside the menu item.
    pub hint: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Post,
    Delete,
}

impl HttpMethod {
    pub fn to_reqwest(self) -> Method {
        match self {
            Self::Post => Method::POST,
            Self::Delete => Method::DELETE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionBody {
    /// `{}` — most action endpoints accept an empty object.
    Empty,
    /// `{ "meta": { "increment": 1 } }`.
    UsageIncrOne,
    /// `{ "meta": { "decrement": 1 } }`.
    UsageDecrOne,
}

impl ActionBody {
    pub fn to_value(self) -> Value {
        match self {
            Self::Empty => json!({}),
            Self::UsageIncrOne => json!({ "meta": { "increment": 1 } }),
            Self::UsageDecrOne => json!({ "meta": { "decrement": 1 } }),
        }
    }
}

/// Build the full URL path for an action, given the resource's base path and
/// the selected row's id.
pub fn action_path(resource_base: &str, id: &str, suffix: &str) -> String {
    let trimmed = resource_base.trim_end_matches('/');
    if suffix.is_empty() {
        format!("{trimmed}/{id}")
    } else {
        format!("{trimmed}/{id}{suffix}")
    }
}

/// Look up the action catalogue for a JSON:API resource type. Returns `None`
/// when the resource has no actions exposed via the panel yet.
pub fn actions_for(jsonapi_type: &str) -> Option<&'static [Action]> {
    match jsonapi_type {
        "licenses" => Some(LICENSE_ACTIONS),
        "machines" => Some(MACHINE_ACTIONS),
        "webhook-endpoints" => Some(WEBHOOK_ENDPOINT_ACTIONS),
        _ => None,
    }
}

const LICENSE_ACTIONS: &[Action] = &[
    Action {
        key: 'v',
        label: "validate",
        tier: Tier::DryRunConfirm,
        method: HttpMethod::Post,
        path_suffix: "/actions/validate",
        body: ActionBody::Empty,
        hint: "server-side validate; counts toward usage",
    },
    Action {
        key: 's',
        label: "suspend",
        tier: Tier::Explicit,
        method: HttpMethod::Post,
        path_suffix: "/actions/suspend",
        body: ActionBody::Empty,
        hint: "block validation; reversible via reinstate",
    },
    Action {
        key: 'r',
        label: "reinstate",
        tier: Tier::Explicit,
        method: HttpMethod::Post,
        path_suffix: "/actions/reinstate",
        body: ActionBody::Empty,
        hint: "lift a suspension",
    },
    Action {
        key: 'n',
        label: "renew",
        tier: Tier::Explicit,
        method: HttpMethod::Post,
        path_suffix: "/actions/renew",
        body: ActionBody::Empty,
        hint: "extend the expiry; no clean undo",
    },
    Action {
        key: 'R',
        label: "revoke",
        tier: Tier::Explicit,
        method: HttpMethod::Delete,
        path_suffix: "",
        body: ActionBody::Empty,
        hint: "DESTRUCTIVE — deletes the license",
    },
    Action {
        key: 'i',
        label: "usage incr (+1)",
        tier: Tier::DryRunConfirm,
        method: HttpMethod::Post,
        path_suffix: "/actions/increment-usage",
        body: ActionBody::UsageIncrOne,
        hint: "increment usage counter by 1",
    },
    Action {
        key: 'd',
        label: "usage decr (-1)",
        tier: Tier::DryRunConfirm,
        method: HttpMethod::Post,
        path_suffix: "/actions/decrement-usage",
        body: ActionBody::UsageDecrOne,
        hint: "decrement usage counter by 1",
    },
    Action {
        key: 'z',
        label: "usage reset",
        tier: Tier::DryRunConfirm,
        method: HttpMethod::Post,
        path_suffix: "/actions/reset-usage",
        body: ActionBody::Empty,
        hint: "reset usage counter to zero",
    },
];

const MACHINE_ACTIONS: &[Action] = &[
    Action {
        key: 'p',
        label: "ping",
        tier: Tier::DryRunConfirm,
        method: HttpMethod::Post,
        path_suffix: "/actions/ping",
        body: ActionBody::Empty,
        hint: "send a heartbeat",
    },
    Action {
        key: 'r',
        label: "reset heartbeat",
        tier: Tier::DryRunConfirm,
        method: HttpMethod::Post,
        path_suffix: "/actions/reset",
        body: ActionBody::Empty,
        hint: "reset heartbeat counter",
    },
    Action {
        key: 'D',
        label: "deactivate",
        tier: Tier::Explicit,
        method: HttpMethod::Delete,
        path_suffix: "",
        body: ActionBody::Empty,
        hint: "DESTRUCTIVE — removes the activation",
    },
];

const WEBHOOK_ENDPOINT_ACTIONS: &[Action] = &[Action {
    key: 't',
    label: "test",
    tier: Tier::DryRunConfirm,
    method: HttpMethod::Post,
    path_suffix: "/actions/test",
    body: ActionBody::Empty,
    hint: "send a test event to this endpoint",
}];

/// Resource base path for a JSON:API type — the URL root the action suffix is
/// appended to. Mirrors `state::RESOURCES`'s third column but for resources
/// the action panel cares about.
pub fn resource_base_path(jsonapi_type: &str) -> Option<&'static str> {
    match jsonapi_type {
        "licenses" => Some("/licenses"),
        "machines" => Some("/machines"),
        "webhook-endpoints" => Some("/webhook-endpoints"),
        _ => None,
    }
}
