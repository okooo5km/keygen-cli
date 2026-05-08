use clap::Subcommand;

use crate::{cli::Context, error::Result};

#[derive(Debug, Subcommand)]
pub enum ConfigCmd {
    /// Set a config key (e.g. `default_profile`, `profiles.<name>.host`).
    Set { key: String, value: String },
    /// Get a config key.
    Get { key: String },
    /// List all config keys.
    List,
    /// Print the path of the config / credentials / cache files.
    Path,
}

pub async fn dispatch(_ctx: &Context, _cmd: ConfigCmd) -> Result<()> {
    Err(crate::Error::user(
        "config commands not yet implemented (step 1 placeholder)",
    ))
}
