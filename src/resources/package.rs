//! Resource: release package.
//!
//! Full action surface is defined in the plan (`keygen-cli-plan.md` § 2.3).
//! This file currently exposes only the CRUD skeleton; resource-specific
//! actions are added in later implementation steps.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// List release package.
    List(ListArgs),
    /// Get a release package by id.
    Get(GetArgs),
    /// Create a release package.
    Create(CreateArgs),
    /// Update a release package.
    Update(UpdateArgs),
    /// Delete a release package.
    Delete(DeleteArgs),
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "package commands not yet implemented (CRUD scaffolding)",
    ))
}
