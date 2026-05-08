//! Resource: machine (license activation).

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    /// Activate (create) a machine for a license.
    Activate {
        #[arg(long)]
        license: String,
        #[arg(long)]
        fingerprint: String,
        #[arg(long)]
        platform: Option<String>,
        #[arg(long, value_name = "K=V")]
        metadata: Vec<String>,
    },
    /// Deactivate (delete) a machine.
    Deactivate {
        id: String,
        #[arg(long)]
        yes: bool,
    },
    Update(UpdateArgs),
    /// Send a heartbeat ping.
    Ping {
        id: String,
    },
    /// Reset the machine's heartbeat counter.
    Reset {
        id: String,
    },
    /// Check out a machine for offline use.
    CheckOut {
        id: String,
        #[arg(long)]
        out: Option<String>,
    },
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "machine commands not yet implemented (CRUD + actions scaffolding)",
    ))
}
