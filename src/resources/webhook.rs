//! Resource: webhook endpoints + events.

use clap::Subcommand;

use crate::{cli::Context, error::Result, resources::common::*};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Manage webhook endpoints.
    #[command(subcommand)]
    Endpoint(EndpointCmd),
    /// Inspect webhook events.
    #[command(subcommand)]
    Event(EventCmd),
}

#[derive(Debug, Subcommand)]
pub enum EndpointCmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// Send a test event to an endpoint.
    Test {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum EventCmd {
    List(ListArgs),
    Get(GetArgs),
    /// Retry a failed event.
    Retry {
        id: String,
    },
}

pub async fn dispatch(_ctx: &Context, _cmd: Cmd) -> Result<()> {
    Err(crate::Error::user(
        "webhook commands not yet implemented (CRUD + actions scaffolding)",
    ))
}
