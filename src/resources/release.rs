//! Resource: release.

use clap::Subcommand;
use serde_json::json;

use crate::{
    api::{client::Query, Client},
    cli::Context,
    error::Result,
    output::{bag, list, single},
    resources::common::*,
};

const CRUD: Crud = Crud::new("releases", "/releases");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// Publish a release.
    Publish {
        id: String,
    },
    /// Yank a release (mark as withdrawn).
    Yank {
        id: String,
        #[arg(long)]
        yes: bool,
    },
    /// Compute the upgrade target for a product + current version.
    Upgrade {
        #[arg(long)]
        product: String,
        #[arg(long)]
        current: String,
        #[arg(long)]
        constraint: Option<String>,
        #[arg(long)]
        channel: Option<String>,
    },
    /// Manage release constraints (entitlements).
    #[command(subcommand)]
    Constraints(ReleaseConstraintsCmd),
    /// Manage release packages.
    #[command(subcommand)]
    Packages(ReleasePackagesCmd),
}

#[derive(Debug, Subcommand)]
pub enum ReleaseConstraintsCmd {
    Attach { id: String, entitlement: String },
    Detach { id: String, entitlement: String },
    List { id: String },
}

#[derive(Debug, Subcommand)]
pub enum ReleasePackagesCmd {
    Attach { id: String, package: String },
    Detach { id: String, package: String },
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
        Cmd::Publish { id } => action(ctx, &id, "publish").await,
        Cmd::Yank { id, yes } => yank(ctx, &id, yes).await,
        Cmd::Upgrade {
            product,
            current,
            constraint,
            channel,
        } => {
            upgrade(
                ctx,
                &product,
                &current,
                constraint.as_deref(),
                channel.as_deref(),
            )
            .await
        }
        Cmd::Constraints(sub) => relationship(ctx, sub.into()).await,
        Cmd::Packages(sub) => relationship(ctx, sub.into()).await,
    }
}

async fn action(ctx: &Context, id: &str, action: &str) -> Result<()> {
    let client = Client::new(ctx)?;
    let path = format!("/releases/{id}/actions/{action}");
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>(&path, &json!({}))
        .await?;
    single(ctx, doc.data)
}

async fn yank(ctx: &Context, id: &str, yes: bool) -> Result<()> {
    if !yes && std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        let typed = inquire::Text::new(&format!("Yank release {id}? Type the id to confirm: "))
            .prompt()
            .map_err(|e| crate::Error::user(format!("aborted: {e}")))?;
        if typed != id {
            return Err(crate::Error::user("yank cancelled (id did not match)"));
        }
    }
    action(ctx, id, "yank").await?;
    bag(ctx, json!({ "yanked": id }))
}

async fn upgrade(
    ctx: &Context,
    product: &str,
    current: &str,
    constraint: Option<&str>,
    channel: Option<&str>,
) -> Result<()> {
    let mut q = Query::new()
        .pair("product", product)
        .pair("version", current);
    if let Some(c) = constraint {
        q = q.pair("constraint", c);
    }
    if let Some(c) = channel {
        q = q.pair("channel", c);
    }
    let client = Client::new(ctx)?;
    let doc = client
        .get::<crate::api::jsonapi::Resource>("/releases/actions/upgrade", &q)
        .await?;
    single(ctx, doc.data)
}

#[derive(Debug)]
struct RelOp {
    relation: &'static str,
    related_type: &'static str,
    target_id: String,
    op: RelOpKind,
    parent_id: String,
}

#[derive(Debug)]
enum RelOpKind {
    Attach,
    Detach,
    List,
}

impl From<ReleaseConstraintsCmd> for RelOp {
    fn from(cmd: ReleaseConstraintsCmd) -> Self {
        match cmd {
            ReleaseConstraintsCmd::Attach { id, entitlement } => Self {
                relation: "constraints",
                related_type: "constraints",
                target_id: entitlement,
                op: RelOpKind::Attach,
                parent_id: id,
            },
            ReleaseConstraintsCmd::Detach { id, entitlement } => Self {
                relation: "constraints",
                related_type: "constraints",
                target_id: entitlement,
                op: RelOpKind::Detach,
                parent_id: id,
            },
            ReleaseConstraintsCmd::List { id } => Self {
                relation: "constraints",
                related_type: "constraints",
                target_id: String::new(),
                op: RelOpKind::List,
                parent_id: id,
            },
        }
    }
}

impl From<ReleasePackagesCmd> for RelOp {
    fn from(cmd: ReleasePackagesCmd) -> Self {
        match cmd {
            ReleasePackagesCmd::Attach { id, package } => Self {
                relation: "packages",
                related_type: "packages",
                target_id: package,
                op: RelOpKind::Attach,
                parent_id: id,
            },
            ReleasePackagesCmd::Detach { id, package } => Self {
                relation: "packages",
                related_type: "packages",
                target_id: package,
                op: RelOpKind::Detach,
                parent_id: id,
            },
            ReleasePackagesCmd::List { id } => Self {
                relation: "packages",
                related_type: "packages",
                target_id: String::new(),
                op: RelOpKind::List,
                parent_id: id,
            },
        }
    }
}

async fn relationship(ctx: &Context, op: RelOp) -> Result<()> {
    let client = Client::new(ctx)?;
    let path = format!("/releases/{}/{}", op.parent_id, op.relation);
    match op.op {
        RelOpKind::List => {
            let doc = client
                .get::<Vec<crate::api::jsonapi::Resource>>(&path, &Query::new())
                .await?;
            list(ctx, &doc.data)
        }
        RelOpKind::Attach => {
            let body = json!({ "data": [{ "type": op.related_type, "id": op.target_id }] });
            client.post::<_, serde_json::Value>(&path, &body).await?;
            bag(
                ctx,
                json!({ "attached": op.target_id, "release": op.parent_id }),
            )
        }
        RelOpKind::Detach => {
            let leaf = format!("{path}/{}", op.target_id);
            client.delete(&leaf).await?;
            bag(
                ctx,
                json!({ "detached": op.target_id, "release": op.parent_id }),
            )
        }
    }
}
