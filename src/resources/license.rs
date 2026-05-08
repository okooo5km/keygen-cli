//! Resource: license.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// Validate a license by id.
    Validate {
        id: String,
    },
    /// Validate a license by its key.
    ValidateKey {
        key: String,
        #[arg(long)]
        fingerprint: Option<String>,
    },
    /// Suspend a license.
    Suspend {
        id: String,
    },
    /// Reinstate a suspended license.
    Reinstate {
        id: String,
    },
    /// Renew a license.
    Renew {
        id: String,
    },
    /// Revoke a license.
    Revoke {
        id: String,
        #[arg(long)]
        yes: bool,
    },
    /// Check out a signed `.lic` blob (offline verification).
    CheckOut {
        id: String,
        #[arg(long)]
        out: Option<String>,
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,
    },
    /// Cancel an outstanding check-out.
    CheckIn {
        id: String,
    },
    /// Manage license usage counters.
    #[command(subcommand)]
    Usage(LicenseUsageCmd),
    /// List tokens scoped to a license.
    Tokens {
        id: String,
    },
    /// Transfer a license to a different user or policy.
    Transfer {
        id: String,
        #[arg(long)]
        to_user: Option<String>,
        #[arg(long)]
        to_policy: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum LicenseUsageCmd {
    Incr {
        id: String,
        #[arg(long, default_value_t = 1)]
        amount: u64,
    },
    Decr {
        id: String,
        #[arg(long, default_value_t = 1)]
        amount: u64,
    },
    Reset {
        id: String,
    },
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "license commands not yet implemented (CRUD + actions scaffolding)",
    ))
}
