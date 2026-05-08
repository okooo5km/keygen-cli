//! Resource: group.
//!
//! Full action surface is defined in the plan (`keygen-cli-plan.md` § 2.3).
//! This file currently exposes only the CRUD skeleton; resource-specific
//! actions are added in later implementation steps.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// List group.
    List(ListArgs),
    /// Get a group by id.
    Get(GetArgs),
    /// Create a group.
    Create(CreateArgs),
    /// Update a group.
    Update(UpdateArgs),
    /// Delete a group.
    Delete(DeleteArgs),
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "group commands not yet implemented (CRUD scaffolding)",
    ))
}
