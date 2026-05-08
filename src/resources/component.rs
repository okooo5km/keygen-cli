//! Resource: machine component.
//!
//! Full action surface is defined in the plan (`keygen-cli-plan.md` § 2.3).
//! This file currently exposes only the CRUD skeleton; resource-specific
//! actions are added in later implementation steps.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// List machine component.
    List(ListArgs),
    /// Get a machine component by id.
    Get(GetArgs),
    /// Create a machine component.
    Create(CreateArgs),
    /// Update a machine component.
    Update(UpdateArgs),
    /// Delete a machine component.
    Delete(DeleteArgs),
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "component commands not yet implemented (CRUD scaffolding)",
    ))
}
