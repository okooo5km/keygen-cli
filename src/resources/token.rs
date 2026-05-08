//! Resource: API token.

use clap::Subcommand;
use serde_json::json;

use crate::{
    api::Client,
    cli::Context,
    error::Result,
    output::{bag, list, single},
    resources::common::*,
};

const CRUD: Crud = Crud::new("tokens", "/tokens");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// Regenerate a token (revokes the old secret).
    Regenerate {
        id: String,
    },
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
        Cmd::Regenerate { id } => regenerate(ctx, &id).await,
    }
}

async fn regenerate(ctx: &Context, id: &str) -> Result<()> {
    let client = Client::new(ctx)?;
    let path = format!("/tokens/{id}/actions/regenerate");
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>(&path, &json!({}))
        .await?;
    single(ctx, doc.data)
}
