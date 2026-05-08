//! Resource: entitlement.

use clap::Subcommand;
use serde_json::json;

use crate::{
    cli::Context,
    error::Result,
    output::{bag, list, single},
    resources::common::*,
};

const CRUD: Crud = Crud::new("entitlements", "/entitlements");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

pub async fn dispatch(ctx: &Context, cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::List(args) => list(ctx, &CRUD.list(ctx, &args).await?),
        Cmd::Get(args) => single(ctx, CRUD.get(ctx, &args).await?),
        Cmd::Create(args) => single(ctx, CRUD.create(ctx, &args).await?),
        Cmd::Update(args) => single(ctx, CRUD.update(ctx, &args).await?),
        Cmd::Delete(args) => {
            CRUD.delete(ctx, &args).await?;
            bag(ctx, json!({ "deleted": args.id }))
        }
    }
}
