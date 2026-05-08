//! Resource: machine process (heartbeat-tracked sub-process).

use clap::Subcommand;
use serde_json::json;

use crate::{
    api::Client,
    cli::Context,
    error::Result,
    output::{bag, list, single},
    resources::common::*,
};

const CRUD: Crud = Crud::new("processes", "/processes");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    /// Spawn a new process for a machine.
    Spawn {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        pid: String,
    },
    /// Kill (delete) a process.
    Kill {
        id: String,
        #[arg(long)]
        yes: bool,
    },
    /// Send a heartbeat ping for a process.
    Ping {
        id: String,
    },
}

pub async fn dispatch(ctx: &Context, cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::List(args) => list(ctx, &CRUD.list(ctx, &args).await?),
        Cmd::Get(args) => single(ctx, CRUD.get(ctx, &args).await?),
        Cmd::Spawn { machine, pid } => spawn(ctx, &machine, &pid).await,
        Cmd::Kill { id, yes } => {
            CRUD.delete(
                ctx,
                &DeleteArgs {
                    id: id.clone(),
                    yes,
                },
            )
            .await?;
            bag(ctx, json!({ "killed": id }))
        }
        Cmd::Ping { id } => {
            let client = Client::new(ctx)?;
            let path = format!("/processes/{id}/actions/ping");
            let doc = client
                .post::<_, crate::api::jsonapi::Resource>(&path, &json!({}))
                .await?;
            single(ctx, doc.data)
        }
    }
}

async fn spawn(ctx: &Context, machine: &str, pid: &str) -> Result<()> {
    let body = json!({
        "data": {
            "type": "processes",
            "attributes": { "pid": pid },
            "relationships": {
                "machine": { "data": { "type": "machines", "id": machine } }
            }
        }
    });
    let client = Client::new(ctx)?;
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>("/processes", &body)
        .await?;
    single(ctx, doc.data)
}
