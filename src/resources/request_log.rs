//! Resource: request log (EE only). Read-only.

use clap::Subcommand;

use crate::{
    capability,
    cli::Context,
    config::profile::Deployment,
    error::{Error, Result},
    output::{list, single},
    resources::common::*,
};

const CRUD: Crud = Crud::new("request-logs", "/request-logs");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
}

pub async fn dispatch(ctx: &Context, cmd: Cmd) -> Result<()> {
    let caps = capability::detect::resolve(ctx).await;
    if !caps.request_logs && matches!(ctx.profile().deployment, Deployment::Ce) {
        return Err(Error::capability(
            "request-logs require keygen.sh Official or EE",
        ));
    }
    match cmd {
        Cmd::List(args) => list(ctx, &CRUD.list(ctx, &args).await?),
        Cmd::Get(args) => single(ctx, CRUD.get(ctx, &args).await?),
    }
}
