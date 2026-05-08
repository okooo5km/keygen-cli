//! Vim-style command palette: `:` opens it, type any `keygen` subcommand
//! (without the `keygen` prefix), Tab completes the current token, Enter
//! parses the line via the same clap tree the binary uses and either runs
//! the result for real or — for Tier 2/3 ops — surfaces a confirm overlay.
//!
//! Execution v1 forks the current binary (`std::env::current_exe()`) so it
//! reuses the keyring lookup, idempotency-key handling, and JSON output
//! pipeline exactly as the CLI does. An in-process `OutputSink` refactor is
//! tracked for v2.
//!
//! Authored by okooo5km.

use std::ffi::OsString;

use clap::{CommandFactory, FromArgMatches};

use crate::{
    cli::Cli,
    tui::permission::{self, Tier},
};

#[derive(Default)]
pub struct PaletteState {
    pub input: String,
    pub suggestions: Vec<String>,
    pub error: Option<String>,
    pub output: Option<String>,
    pub in_flight: bool,
    pub awaiting: Option<AwaitingExec>,
}

#[derive(Clone)]
pub struct AwaitingExec {
    pub argv: Vec<OsString>,
    pub display: String,
    pub tier: Tier,
}

pub struct ParsedCommand {
    pub argv: Vec<OsString>,
    pub display: String,
    pub tier: Tier,
}

impl PaletteState {
    pub fn open() -> Self {
        let mut s = Self::default();
        s.refresh_suggestions();
        s
    }

    pub fn push_char(&mut self, ch: char) {
        self.input.push(ch);
        self.error = None;
        self.refresh_suggestions();
    }

    pub fn pop_char(&mut self) {
        self.input.pop();
        self.error = None;
        self.refresh_suggestions();
    }

    pub fn complete(&mut self) {
        if self.suggestions.len() == 1 {
            // Replace the trailing partial token with the suggestion.
            let mut tokens: Vec<&str> = self.input.split_whitespace().collect();
            let trailing_space = self.input.ends_with(' ');
            if trailing_space || tokens.is_empty() {
                tokens.push(&self.suggestions[0]);
            } else {
                let last = tokens.pop().unwrap_or("");
                let _ = last;
                tokens.push(&self.suggestions[0]);
            }
            self.input = tokens.join(" ");
            self.input.push(' ');
            self.refresh_suggestions();
        }
    }

    pub fn parse(&self) -> std::result::Result<ParsedCommand, String> {
        let parts = shell_split(&self.input).map_err(|e| format!("parse: {e}"))?;
        if parts.is_empty() {
            return Err("empty command".into());
        }
        let mut argv: Vec<OsString> = Vec::with_capacity(parts.len() + 1);
        argv.push(OsString::from("keygen"));
        for p in &parts {
            argv.push(OsString::from(p));
        }

        let cmd = Cli::command();
        let matches = cmd
            .clone()
            .try_get_matches_from(&argv)
            .map_err(|e| e.to_string())?;
        let cli = Cli::from_arg_matches(&matches).map_err(|e| e.to_string())?;
        let tier = permission::tier_for_cli(&cli);

        Ok(ParsedCommand {
            argv,
            display: format!("keygen {}", parts.join(" ")),
            tier,
        })
    }

    fn refresh_suggestions(&mut self) {
        self.suggestions.clear();
        let trailing = self.input.ends_with(' ');
        let tokens: Vec<&str> = self.input.split_whitespace().collect();
        let prefix = if trailing {
            ""
        } else {
            tokens.last().copied().unwrap_or("")
        };

        if tokens.is_empty() || (tokens.len() == 1 && !trailing) {
            for top in TOP_LEVEL_COMMANDS {
                if top.starts_with(prefix) {
                    self.suggestions.push((*top).into());
                }
            }
            return;
        }

        // Second token: subcommand suggestions for the resource named in the
        // first token, when we recognise it.
        if (tokens.len() == 2 && !trailing) || (tokens.len() == 1 && trailing) {
            let resource = tokens[0];
            if let Some(subs) = sub_commands_for(resource) {
                for sub in subs {
                    if sub.starts_with(prefix) {
                        self.suggestions.push((*sub).into());
                    }
                }
            }
        }
    }
}

const TOP_LEVEL_COMMANDS: &[&str] = &[
    "license",
    "machine",
    "policy",
    "product",
    "user",
    "group",
    "token",
    "release",
    "artifact",
    "package",
    "entitlement",
    "component",
    "process",
    "webhook",
    "request-log",
    "event-log",
    "whoami",
    "doctor",
    "schema",
    "explain",
    "config",
    "profile",
    "env",
];

fn sub_commands_for(resource: &str) -> Option<&'static [&'static str]> {
    match resource {
        "license" => Some(&[
            "list",
            "get",
            "create",
            "update",
            "delete",
            "validate",
            "validate-key",
            "verify",
            "suspend",
            "reinstate",
            "renew",
            "revoke",
            "check-out",
            "check-in",
            "usage",
            "tokens",
            "transfer",
        ]),
        "machine" => Some(&[
            "list",
            "get",
            "update",
            "activate",
            "deactivate",
            "ping",
            "reset",
            "check-out",
        ]),
        "release" => Some(&[
            "list",
            "get",
            "create",
            "update",
            "delete",
            "publish",
            "yank",
            "upgrade",
            "constraints",
            "packages",
        ]),
        "user" => Some(&[
            "list",
            "get",
            "create",
            "update",
            "delete",
            "ban",
            "unban",
            "reset-password",
            "update-password",
            "groups",
            "tokens",
        ]),
        "policy" | "product" | "group" | "entitlement" | "package" | "token" | "component" => {
            Some(&["list", "get", "create", "update", "delete"])
        }
        "process" => Some(&["list", "get", "spawn", "kill", "ping"]),
        "artifact" => Some(&["list", "get", "delete", "upload", "download", "yank"]),
        "webhook" => Some(&["endpoint", "event"]),
        "request-log" | "event-log" => Some(&["list", "get"]),
        _ => None,
    }
}

/// Minimal POSIX-like split: whitespace-separated, supports double-quoted
/// segments. Good enough for the common case `keygen license create --set
/// "attrs.name=Acme"`. Refuses unmatched quotes.
fn shell_split(s: &str) -> std::result::Result<Vec<String>, &'static str> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut had_token = false;
    for ch in s.chars() {
        match ch {
            '"' => {
                in_quote = !in_quote;
                had_token = true;
            }
            c if c.is_whitespace() && !in_quote => {
                if had_token {
                    out.push(std::mem::take(&mut cur));
                    had_token = false;
                }
            }
            c => {
                cur.push(c);
                had_token = true;
            }
        }
    }
    if in_quote {
        return Err("unmatched quote");
    }
    if had_token {
        out.push(cur);
    }
    Ok(out)
}
