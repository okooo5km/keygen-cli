//! Single source of truth for the three-tier permission rules.
//!
//! Mirrors `skills/keygen/references/permissions.md`. Both the TUI action
//! panel (Stage b) and the command palette (Stage d) consult `tier_for_cli`
//! before running any non-read-only operation, so the agent docs and the
//! interactive UX agree.
//!
//! Authored by okooo5km.

#![allow(clippy::enum_glob_use)]

use crate::cli::{Cli, Command};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Read-only inspection. Run freely.
    AutoRun,
    /// Mutating but reversible (or low blast). Preview with `--dry-run`,
    /// then re-run for real after explicit confirmation.
    DryRunConfirm,
    /// Destructive or irreversible. Require an explicit confirmation and
    /// surface the target id + blast radius before sending.
    Explicit,
}

/// Look up the tier for a parsed CLI invocation. Conservative on unknown
/// shapes: if we cannot prove a command is read-only, escalate.
pub fn tier_for_cli(cli: &Cli) -> Tier {
    match &cli.command {
        Command::Whoami | Command::Doctor(_) | Command::Schema(_) | Command::Completion(_) => {
            Tier::AutoRun
        }
        Command::Login(_) | Command::Logout(_) => Tier::DryRunConfirm,
        Command::Config(_) | Command::Profile(_) | Command::Env(_) | Command::Explain(_) => {
            // `config get / list / show`, `profile list / use`, `env list`
            // are read-only; mutating subcommands of these still hit the
            // local config and are reversible. Treat the whole tree as auto.
            Tier::AutoRun
        }
        Command::Tui => Tier::AutoRun,
        Command::Resource(rc) => tier_for_resource(rc),
    }
}

fn tier_for_resource(rc: &crate::cli::resources::ResourceCommand) -> Tier {
    use crate::cli::resources::ResourceCommand as RC;
    match rc {
        RC::Token(c) => tier_for_token(c),
        RC::Product(c) => tier_for_product(c),
        RC::Policy(c) => tier_for_policy(c),
        RC::License(c) => tier_for_license(c),
        RC::Entitlement(c) => tier_for_simple_crud(c),
        RC::User(c) => tier_for_user(c),
        RC::Group(c) => tier_for_group(c),
        RC::Machine(c) => tier_for_machine(c),
        RC::Component(c) => tier_for_simple_crud(c),
        RC::Process(c) => tier_for_process(c),
        RC::Release(c) => tier_for_release(c),
        RC::Artifact(c) => tier_for_artifact(c),
        RC::Package(c) => tier_for_simple_crud(c),
        RC::Webhook(c) => tier_for_webhook(c),
        RC::RequestLog(_) | RC::EventLog(_) => Tier::AutoRun,
    }
}

// ----- per-resource helpers -----

fn tier_for_simple_crud<T>(c: &T) -> Tier
where
    T: SimpleCrud,
{
    if c.is_list_or_get() {
        Tier::AutoRun
    } else if c.is_delete() {
        Tier::Explicit
    } else {
        Tier::DryRunConfirm
    }
}

fn tier_for_token(c: &crate::resources::token::Cmd) -> Tier {
    use crate::resources::token::Cmd::*;
    match c {
        List(_) | Get(_) => Tier::AutoRun,
        Create(_) | Update(_) => Tier::DryRunConfirm,
        Delete(_) | Regenerate { .. } => Tier::Explicit,
    }
}

fn tier_for_product(c: &crate::resources::product::Cmd) -> Tier {
    use crate::resources::product::Cmd::*;
    match c {
        List(_) | Get(_) | Tokens { .. } => Tier::AutoRun,
        Create(_) | Update(_) => Tier::DryRunConfirm,
        Delete(_) => Tier::Explicit,
    }
}

fn tier_for_policy(c: &crate::resources::policy::Cmd) -> Tier {
    use crate::resources::policy::Cmd::*;
    match c {
        List(_) | Get(_) => Tier::AutoRun,
        Create(_) | Update(_) | Entitlements(_) => Tier::DryRunConfirm,
        Delete(_) => Tier::Explicit,
    }
}

fn tier_for_license(c: &crate::resources::license::Cmd) -> Tier {
    use crate::resources::license::Cmd::*;
    match c {
        List(_) | Get(_) | Tokens { .. } | Verify(_) | ValidateKey { .. } => Tier::AutoRun,
        Create(_) | Update(_) | Validate { .. } | CheckOut { .. } | CheckIn { .. } | Usage(_) => {
            Tier::DryRunConfirm
        }
        Delete(_)
        | Suspend { .. }
        | Reinstate { .. }
        | Renew { .. }
        | Revoke { .. }
        | Transfer { .. } => Tier::Explicit,
    }
}

fn tier_for_user(c: &crate::resources::user::Cmd) -> Tier {
    use crate::resources::user::Cmd::*;
    match c {
        List(_) | Get(_) | Tokens { .. } | Groups(_) => Tier::AutoRun,
        Create(_) | Update(_) => Tier::DryRunConfirm,
        Delete(_) | Ban { .. } | Unban { .. } | ResetPassword { .. } | UpdatePassword { .. } => {
            Tier::Explicit
        }
    }
}

fn tier_for_group(c: &crate::resources::group::Cmd) -> Tier {
    use crate::resources::group::Cmd::*;
    match c {
        List(_) | Get(_) | Licenses { .. } | Users(_) => Tier::AutoRun,
        Create(_) | Update(_) => Tier::DryRunConfirm,
        Delete(_) => Tier::Explicit,
    }
}

fn tier_for_machine(c: &crate::resources::machine::Cmd) -> Tier {
    use crate::resources::machine::Cmd::*;
    match c {
        List(_) | Get(_) => Tier::AutoRun,
        Update(_) | Activate { .. } | Ping { .. } | Reset { .. } | CheckOut { .. } => {
            Tier::DryRunConfirm
        }
        Deactivate { .. } => Tier::Explicit,
    }
}

fn tier_for_process(c: &crate::resources::process::Cmd) -> Tier {
    use crate::resources::process::Cmd::*;
    match c {
        List(_) | Get(_) => Tier::AutoRun,
        Spawn { .. } | Ping { .. } => Tier::DryRunConfirm,
        Kill { .. } => Tier::Explicit,
    }
}

fn tier_for_release(c: &crate::resources::release::Cmd) -> Tier {
    use crate::resources::release::Cmd::*;
    match c {
        List(_) | Get(_) | Upgrade { .. } => Tier::AutoRun,
        Create(_) | Update(_) | Constraints(_) | Packages(_) => Tier::DryRunConfirm,
        Delete(_) | Publish { .. } | Yank { .. } => Tier::Explicit,
    }
}

fn tier_for_artifact(c: &crate::resources::artifact::Cmd) -> Tier {
    use crate::resources::artifact::Cmd::*;
    match c {
        List(_) | Get(_) => Tier::AutoRun,
        Upload { .. } | Download { .. } => Tier::DryRunConfirm,
        Delete(_) | Yank { .. } => Tier::Explicit,
    }
}

fn tier_for_webhook(c: &crate::resources::webhook::Cmd) -> Tier {
    use crate::resources::webhook::Cmd::*;
    match c {
        Endpoint(sub) => tier_for_webhook_endpoint(sub),
        Event(sub) => tier_for_webhook_event(sub),
    }
}

fn tier_for_webhook_endpoint(c: &crate::resources::webhook::EndpointCmd) -> Tier {
    use crate::resources::webhook::EndpointCmd::*;
    match c {
        List(_) | Get(_) => Tier::AutoRun,
        Create(_) | Update(_) | Test { .. } => Tier::DryRunConfirm,
        Delete(_) => Tier::Explicit,
    }
}

fn tier_for_webhook_event(c: &crate::resources::webhook::EventCmd) -> Tier {
    use crate::resources::webhook::EventCmd::*;
    match c {
        List(_) | Get(_) => Tier::AutoRun,
        Retry { .. } => Tier::DryRunConfirm,
    }
}

// ----- generic CRUD trait for resources whose Cmd has only the standard five -----

/// Implemented by per-resource `Cmd` enums that follow the canonical CRUD
/// shape (`List / Get / Create / Update / Delete`).
trait SimpleCrud {
    fn is_list_or_get(&self) -> bool;
    fn is_delete(&self) -> bool;
}

impl SimpleCrud for crate::resources::entitlement::Cmd {
    fn is_list_or_get(&self) -> bool {
        matches!(self, Self::List(_) | Self::Get(_))
    }
    fn is_delete(&self) -> bool {
        matches!(self, Self::Delete(_))
    }
}

impl SimpleCrud for crate::resources::component::Cmd {
    fn is_list_or_get(&self) -> bool {
        matches!(self, Self::List(_) | Self::Get(_))
    }
    fn is_delete(&self) -> bool {
        matches!(self, Self::Delete(_))
    }
}

impl SimpleCrud for crate::resources::package::Cmd {
    fn is_list_or_get(&self) -> bool {
        matches!(self, Self::List(_) | Self::Get(_))
    }
    fn is_delete(&self) -> bool {
        matches!(self, Self::Delete(_))
    }
}
