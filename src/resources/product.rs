//! Resource: product.

use clap::Subcommand;
use serde_json::json;

use crate::{
    api::{client::Query, Client},
    cli::Context,
    error::Result,
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
        Cmd::List(args) => emit(CRUD.list(ctx, &args).await?),
        Cmd::Get(args) => emit(CRUD.get(ctx, &args).await?),
        Cmd::Create(args) => emit(CRUD.create(ctx, &args).await?),
        Cmd::Update(args) => emit(CRUD.update(ctx, &args).await?),
        Cmd::Delete(args) => {
            CRUD.delete(ctx, &args).await?;
            crate::output::json::print(&json!({ "ok": true, "data": { "deleted": args.id } }))
        }
        Cmd::Tokens { id } => {
            let client = Client::new(ctx)?;
            let path = format!("/products/{id}/tokens");
            let doc = client
                .get::<Vec<crate::api::jsonapi::Resource>>(&path, &Query::new())
                .await?;
            emit(doc.data)
        }
    }
}

fn emit<T: serde::Serialize>(data: T) -> Result<()> {
    crate::output::json::print(&json!({ "ok": true, "data": data }))
}
