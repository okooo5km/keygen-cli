//! Resource: product.

use clap::Subcommand;
use serde_json::json;

use crate::{
    api::{client::Query, Client},
    cli::Context,
    error::Result,
    output::{bag, list, single},
    resources::common::*,
};

const CRUD: Crud = Crud::new("products", "/products");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// List tokens that scope to a product.
    Tokens {
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
        Cmd::Tokens { id } => {
            let client = Client::new(ctx)?;
            let path = format!("/products/{id}/tokens");
            let doc = client
                .get::<Vec<crate::api::jsonapi::Resource>>(&path, &Query::new())
                .await?;
            list(ctx, &doc.data)
        }
    }
}
