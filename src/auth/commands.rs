use clap::Args;

use crate::{cli::Context, error::Result};

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Use the supplied token instead of running the interactive flow.
    #[arg(long)]
    pub token: Option<String>,

    /// Token kind hint (admin / product / user / environment / license).
    #[arg(long)]
    pub kind: Option<String>,
}

#[derive(Debug, Args)]
pub struct LogoutArgs {
    /// Profile to clear. Defaults to the active profile.
    #[arg(long)]
    pub profile: Option<String>,
}

pub async fn login(_ctx: &Context, _args: LoginArgs) -> Result<()> {
    Err(crate::Error::user(
        "login flow not yet implemented (step 3 placeholder)",
    ))
}

pub async fn logout(_ctx: &Context, _args: LogoutArgs) -> Result<()> {
    Err(crate::Error::user(
        "logout not yet implemented (step 3 placeholder)",
    ))
}

pub async fn whoami(_ctx: &Context) -> Result<()> {
    Err(crate::Error::user(
        "whoami not yet implemented (step 3 placeholder)",
    ))
}
