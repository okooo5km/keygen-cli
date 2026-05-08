//! Resource: release artifact.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Delete(DeleteArgs),
    /// Upload a binary to a release.
    Upload {
        release: String,
        #[arg(long)]
        file: String,
        #[arg(long)]
        platform: Option<String>,
        #[arg(long)]
        arch: Option<String>,
    },
    /// Download a binary by id.
    Download {
        id: String,
        #[arg(long)]
        out: Option<String>,
    },
    /// Yank an artifact.
    Yank {
        id: String,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "artifact commands not yet implemented (CRUD + actions scaffolding)",
    ))
}
