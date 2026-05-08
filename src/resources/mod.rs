//! One module per keygen.sh resource. Each module exposes a `Cmd` enum
//! (clap subcommand surface) and a `dispatch` function. The shared CRUD
//! pattern lives in `common`.

pub mod artifact;
pub mod common;
pub mod component;
pub mod entitlement;
pub mod event_log;
pub mod group;
pub mod license;
pub mod machine;
pub mod package;
pub mod policy;
pub mod process;
pub mod product;
pub mod release;
pub mod request_log;
pub mod token;
pub mod user;
pub mod webhook;

use crate::{cli::resources::ResourceCommand, cli::Context, error::Result};

pub async fn dispatch(ctx: &Context, cmd: ResourceCommand) -> Result<()> {
    match cmd {
        ResourceCommand::Token(c) => token::dispatch(ctx, c).await,
        ResourceCommand::Product(c) => product::dispatch(ctx, c).await,
        ResourceCommand::Policy(c) => policy::dispatch(ctx, c).await,
        ResourceCommand::License(c) => license::dispatch(ctx, c).await,
        ResourceCommand::Entitlement(c) => entitlement::dispatch(ctx, c).await,
        ResourceCommand::User(c) => user::dispatch(ctx, c).await,
        ResourceCommand::Group(c) => group::dispatch(ctx, c).await,
        ResourceCommand::Machine(c) => machine::dispatch(ctx, c).await,
        ResourceCommand::Component(c) => component::dispatch(ctx, c).await,
        ResourceCommand::Process(c) => process::dispatch(ctx, c).await,
        ResourceCommand::Release(c) => release::dispatch(ctx, c).await,
        ResourceCommand::Artifact(c) => artifact::dispatch(ctx, c).await,
        ResourceCommand::Package(c) => package::dispatch(ctx, c).await,
        ResourceCommand::Webhook(c) => webhook::dispatch(ctx, c).await,
        ResourceCommand::RequestLog(c) => request_log::dispatch(ctx, c).await,
        ResourceCommand::EventLog(c) => event_log::dispatch(ctx, c).await,
    }
}
