//! Resource: release.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    Publish {
        id: String,
    },
    Yank {
        id: String,
        #[arg(long)]
        yes: bool,
    },
    /// Compute the upgrade target for a product + current version.
    Upgrade {
        #[arg(long)]
        product: String,
        #[arg(long)]
        current: String,
        #[arg(long)]
        constraint: Option<String>,
        #[arg(long)]
        channel: Option<String>,
    },
    /// Manage release constraints (entitlements).
    #[command(subcommand)]
    Constraints(ReleaseConstraintsCmd),
}

#[derive(Debug, Subcommand)]
pub enum ReleaseConstraintsCmd {
    Attach { id: String, entitlement: String },
    Detach { id: String, entitlement: String },
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "release commands not yet implemented (CRUD + actions scaffolding)",
    ))
}
