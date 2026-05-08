use clap::Subcommand;

use crate::{cli::Context, error::Result};

#[derive(Debug, Subcommand)]
pub enum EnvCmd {
    /// List EE environments.
    List,
    /// Switch the active environment.
    Use { id: String },
    /// Print the active environment.
    Current,
}

pub async fn dispatch(_ctx: &Context, _cmd: EnvCmd) -> Result<()> {
    Err(crate::Error::capability(
        "environments require keygen.sh Official or EE",
    ))
}
