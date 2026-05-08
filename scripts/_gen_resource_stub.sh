#!/usr/bin/env bash
slug="$1"
title="$2"
cat <<INNER
//! Resource: ${title}.
//!
//! Full action surface is defined in the plan (\`keygen-cli-plan.md\` § 2.3).
//! This file currently exposes only the CRUD skeleton; resource-specific
//! actions are added in later implementation steps.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// List ${title}.
    List(ListArgs),
    /// Get a ${title} by id.
    Get(GetArgs),
    /// Create a ${title}.
    Create(CreateArgs),
    /// Update a ${title}.
    Update(UpdateArgs),
    /// Delete a ${title}.
    Delete(DeleteArgs),
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "${slug} commands not yet implemented (CRUD scaffolding)",
    ))
}
INNER
