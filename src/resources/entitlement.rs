//! Resource: entitlement.
//!
//! Full action surface is defined in the plan (`keygen-cli-plan.md` § 2.3).
//! This file currently exposes only the CRUD skeleton; resource-specific
//! actions are added in later implementation steps.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// List entitlement.
    List(ListArgs),
    /// Get a entitlement by id.
    Get(GetArgs),
    /// Create a entitlement.
    Create(CreateArgs),
    /// Update a entitlement.
    Update(UpdateArgs),
    /// Delete a entitlement.
    Delete(DeleteArgs),
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "entitlement commands not yet implemented (CRUD scaffolding)",
    ))
}
