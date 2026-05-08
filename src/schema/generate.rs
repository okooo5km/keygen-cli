//! Walk the clap `Command` tree and emit a stable JSON schema describing
//! every command, subcommand, positional, flag, and enum value.
//!
//! AI agents call `keygen schema` once and use the result to drive tool
//! selection without parsing `--help` text.

use clap::{Arg, ArgAction, Command};
use serde_json::{json, Value};

pub fn dump(root: &Command) -> Value {
    json!({
        "ok": true,
        "data": {
            "version": env!("CARGO_PKG_VERSION"),
            "binary": root.get_name(),
            "exit_codes": exit_codes(),
            "command": describe(root),
        }
    })
}

fn describe(cmd: &Command) -> Value {
    let mut subcommands: Vec<Value> = Vec::new();
    for sub in cmd.get_subcommands() {
        subcommands.push(describe(sub));
    }

    let mut args: Vec<Value> = Vec::new();
    for arg in cmd.get_arguments() {
        args.push(describe_arg(arg));
    }

    json!({
        "name": cmd.get_name(),
        "about": cmd.get_about().map(ToString::to_string),
        "long_about": cmd.get_long_about().map(ToString::to_string),
        "args": args,
        "subcommands": subcommands,
    })
}

fn describe_arg(arg: &Arg) -> Value {
    let id = arg.get_id().to_string();
    let kind = match arg.get_action() {
        ArgAction::SetTrue | ArgAction::SetFalse => "flag",
        ArgAction::Count => "counter",
        ArgAction::Append => "list",
        _ if arg.is_positional() => "positional",
        _ => "option",
    };
    let possible: Vec<String> = arg
        .get_possible_values()
        .iter()
        .map(|p| p.get_name().to_string())
        .collect();
    json!({
        "name": id,
        "long": arg.get_long(),
        "short": arg.get_short().map(|c| c.to_string()),
        "value_name": arg.get_value_names()
            .and_then(|n| n.first())
            .map(ToString::to_string),
        "kind": kind,
        "required": arg.is_required_set(),
        "default": arg.get_default_values()
            .iter()
            .map(|v| v.to_string_lossy().to_string())
            .collect::<Vec<_>>(),
        "help": arg.get_help().map(ToString::to_string),
        "long_help": arg.get_long_help().map(ToString::to_string),
        "possible_values": possible,
        "env": arg.get_env().and_then(|e| e.to_str()).map(ToString::to_string),
        "global": arg.is_global_set(),
    })
}

fn exit_codes() -> Value {
    json!([
        { "code": 0, "name": "ok", "meaning": "command succeeded" },
        { "code": 1, "name": "user_error", "meaning": "user supplied bad input or hit a 4xx (non-401/403)" },
        { "code": 2, "name": "server_error", "meaning": "keygen returned 5xx" },
        { "code": 3, "name": "network_error", "meaning": "request failed before a response arrived" },
        { "code": 4, "name": "auth_error", "meaning": "token missing / invalid (401 / 403)" },
        { "code": 5, "name": "capability", "meaning": "command not supported on this deployment" }
    ])
}
