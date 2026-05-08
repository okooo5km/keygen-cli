//! Resource: request log.
//!
//! Full action surface is defined in the plan (`keygen-cli-plan.md` § 2.3).
//! This file currently exposes only the CRUD skeleton; resource-specific
//! actions are added in later implementation steps.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// List request log.
    List(ListArgs),
    /// Get a request log by id.
    Get(GetArgs),
    /// Create a request log.
    Create(CreateArgs),
    /// Update a request log.
    Update(UpdateArgs),
    /// Delete a request log.
    Delete(DeleteArgs),
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "request_log commands not yet implemented (CRUD scaffolding)",
    ))
}
