//! Resource: user.

use clap::Subcommand;
use serde_json::json;

use crate::{
    api::{client::Query, Client},
    cli::Context,
    error::Result,
    output::{bag, list, single},
    resources::common::*,
};

const CRUD: Crud = Crud::new("users", "/users");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// Ban (block) a user.
    Ban {
        id: String,
    },
    /// Lift a ban from a user.
    Unban {
        id: String,
    },
    /// Send a password-reset email.
    ResetPassword {
        id: String,
    },
    /// Set the user's password (admin only).
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
    /// List tokens scoped to the user.
    Tokens {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum UserGroupsCmd {
    Attach { id: String, group: String },
    Detach { id: String, group: String },
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
        Cmd::Ban { id } => action(ctx, &id, "ban").await,
        Cmd::Unban { id } => action(ctx, &id, "unban").await,
        Cmd::ResetPassword { id } => action(ctx, &id, "reset-password").await,
        Cmd::UpdatePassword {
            id,
            old_password,
            new_password,
        } => update_password(ctx, &id, old_password.as_deref(), new_password.as_deref()).await,
        Cmd::Groups(sub) => groups(ctx, sub).await,
        Cmd::Tokens { id } => {
            let client = Client::new(ctx)?;
            let path = format!("/users/{id}/tokens");
            let doc = client
                .get::<Vec<crate::api::jsonapi::Resource>>(&path, &Query::new())
                .await?;
            list(ctx, &doc.data)
        }
    }
}

async fn action(ctx: &Context, id: &str, action: &str) -> Result<()> {
    let client = Client::new(ctx)?;
    let path = format!("/users/{id}/actions/{action}");
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>(&path, &json!({}))
        .await?;
    single(ctx, doc.data)
}

async fn update_password(
    ctx: &Context,
    id: &str,
    old_password: Option<&str>,
    new_password: Option<&str>,
) -> Result<()> {
    let new = new_password.ok_or_else(|| {
        crate::Error::user("--new-password is required (use a generator or pipe via stdin)")
    })?;
    let mut meta = json!({ "newPassword": new });
    if let Some(old) = old_password {
        meta["oldPassword"] = json!(old);
    }
    let client = Client::new(ctx)?;
    let path = format!("/users/{id}/actions/update-password");
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>(&path, &json!({ "meta": meta }))
        .await?;
    single(ctx, doc.data)
}

async fn groups(ctx: &Context, cmd: UserGroupsCmd) -> Result<()> {
    let client = Client::new(ctx)?;
    match cmd {
        UserGroupsCmd::List { id } => {
            let path = format!("/users/{id}/groups");
            let doc = client
                .get::<Vec<crate::api::jsonapi::Resource>>(&path, &Query::new())
                .await?;
            list(ctx, &doc.data)
        }
        UserGroupsCmd::Attach { id, group } => {
            let path = format!("/users/{id}/groups");
            let body = json!({ "data": [{ "type": "groups", "id": group }] });
            client.post::<_, serde_json::Value>(&path, &body).await?;
            bag(ctx, json!({ "attached": group, "user": id }))
        }
        UserGroupsCmd::Detach { id, group } => {
            let path = format!("/users/{id}/groups/{group}");
            client.delete(&path).await?;
            bag(ctx, json!({ "detached": group, "user": id }))
        }
    }
}
