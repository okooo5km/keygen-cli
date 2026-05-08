//! Resource: group.

use clap::Subcommand;
use serde_json::json;

use crate::{
    api::{client::Query, Client},
    cli::Context,
    error::Result,
    output::{bag, list, single},
    resources::common::*,
};

const CRUD: Crud = Crud::new("groups", "/groups");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// Manage user membership for a group.
    #[command(subcommand)]
    Users(GroupUsersCmd),
    /// List licenses owned by the group.
    Licenses {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum GroupUsersCmd {
    Attach { id: String, user: String },
    Detach { id: String, user: String },
    List { id: String },
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
        Cmd::Users(sub) => users(ctx, sub).await,
        Cmd::Licenses { id } => {
            let client = Client::new(ctx)?;
            let path = format!("/groups/{id}/licenses");
            let doc = client
                .get::<Vec<crate::api::jsonapi::Resource>>(&path, &Query::new())
                .await?;
            list(ctx, &doc.data)
        }
    }
}

async fn users(ctx: &Context, cmd: GroupUsersCmd) -> Result<()> {
    let client = Client::new(ctx)?;
    match cmd {
        GroupUsersCmd::List { id } => {
            let path = format!("/groups/{id}/users");
            let doc = client
                .get::<Vec<crate::api::jsonapi::Resource>>(&path, &Query::new())
                .await?;
            list(ctx, &doc.data)
        }
        GroupUsersCmd::Attach { id, user } => {
            let path = format!("/groups/{id}/users");
            let body = json!({ "data": [{ "type": "users", "id": user }] });
            client.post::<_, serde_json::Value>(&path, &body).await?;
            bag(ctx, json!({ "attached": user, "group": id }))
        }
        GroupUsersCmd::Detach { id, user } => {
            let path = format!("/groups/{id}/users/{user}");
            client.delete(&path).await?;
            bag(ctx, json!({ "detached": user, "group": id }))
        }
    }
}
