use clap::Subcommand;

use crate::{cli::Context, error::Result};

#[derive(Debug, Subcommand)]
pub enum ProfileCmd {
    /// List all profiles.
    List,
    /// Mark a profile as the default.
    Use { name: String },
    /// Add a new profile interactively.
    Add { name: String },
    /// Remove a profile (does not touch keyring entries).
    Remove { name: String },
}

pub async fn dispatch(_ctx: &Context, _cmd: ProfileCmd) -> Result<()> {
    Err(crate::Error::user(
        "profile commands not yet implemented (step 1 placeholder)",
    ))
}
