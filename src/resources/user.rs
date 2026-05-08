//! Resource: user.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    Ban {
        id: String,
    },
    Unban {
        id: String,
    },
    ResetPassword {
        id: String,
    },
    UpdatePassword {
        id: String,
        #[arg(long)]
        old_password: Option<String>,
        #[arg(long)]
        new_password: Option<String>,
    },
    /// Manage group membership for a user.
    #[command(subcommand)]
    Groups(UserGroupsCmd),
}

#[derive(Debug, Subcommand)]
pub enum UserGroupsCmd {
    Attach { id: String, group: String },
    Detach { id: String, group: String },
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "user commands not yet implemented (CRUD + actions scaffolding)",
    ))
}
