//! Resource: policy.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// Manage entitlements attached to a policy.
    #[command(subcommand)]
    Entitlements(PolicyEntitlementsCmd),
}

#[derive(Debug, Subcommand)]
pub enum PolicyEntitlementsCmd {
    Attach { id: String, entitlement: String },
    Detach { id: String, entitlement: String },
    List { id: String },
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "policy commands not yet implemented (CRUD scaffolding)",
    ))
}
