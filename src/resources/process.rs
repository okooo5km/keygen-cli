//! Resource: machine process.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    /// Spawn a new process for a machine.
    Spawn {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        pid: String,
    },
    /// Kill (delete) a process.
    Kill {
        id: String,
        #[arg(long)]
        yes: bool,
    },
    /// Send a heartbeat ping for a process.
    Ping {
        id: String,
    },
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "process commands not yet implemented (CRUD + actions scaffolding)",
    ))
}
