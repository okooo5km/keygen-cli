//! Resource: policy.

use clap::Subcommand;
use serde_json::json;

use crate::{
    api::{client::Query, Client},
    cli::Context,
    error::Result,
    resources::common::*,
};

const CRUD: Crud = Crud::new("policies", "/policies");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// Manage entitlements attached to a policy.
    #[command(subcommand)]
    Entitlements(PolicyEntitlementsCmd),
}

#[derive(Debug, Subcommand)]
pub enum PolicyEntitlementsCmd {
    /// Attach an entitlement to a policy.
    Attach { id: String, entitlement: String },
    /// Detach an entitlement from a policy.
    Detach { id: String, entitlement: String },
    /// List entitlements currently attached to a policy.
    List { id: String },
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
        Cmd::Entitlements(sub) => entitlements(ctx, sub).await,
    }
}

async fn entitlements(ctx: &Context, cmd: PolicyEntitlementsCmd) -> Result<()> {
    let client = Client::new(ctx)?;
    match cmd {
        PolicyEntitlementsCmd::List { id } => {
            let path = format!("/policies/{id}/entitlements");
            let doc = client
                .get::<Vec<crate::api::jsonapi::Resource>>(&path, &Query::new())
                .await?;
            emit(doc.data)
        }
        PolicyEntitlementsCmd::Attach { id, entitlement } => {
            let path = format!("/policies/{id}/entitlements");
            let body = json!({
                "data": [{ "type": "entitlements", "id": entitlement }]
            });
            client.post::<_, serde_json::Value>(&path, &body).await?;
            crate::output::json::print(
                &json!({ "ok": true, "data": { "attached": entitlement, "policy": id } }),
            )
        }
        PolicyEntitlementsCmd::Detach { id, entitlement } => {
            let path = format!("/policies/{id}/entitlements/{entitlement}");
            client.delete(&path).await?;
            crate::output::json::print(
                &json!({ "ok": true, "data": { "detached": entitlement, "policy": id } }),
            )
        }
    }
}

fn emit<T: serde::Serialize>(data: T) -> Result<()> {
    crate::output::json::print(&json!({ "ok": true, "data": data }))
}
