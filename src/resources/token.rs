//! Resource: API token.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// List tokens.
    List(ListArgs),
    /// Get a token.
    Get(GetArgs),
    /// Create a token.
    Create(CreateArgs),
    /// Update a token.
    Update(UpdateArgs),
    /// Delete a token.
    Delete(DeleteArgs),
    /// Regenerate a token (revokes the old secret).
    Regenerate { id: String },
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "token commands not yet implemented (CRUD scaffolding)",
    ))
}
