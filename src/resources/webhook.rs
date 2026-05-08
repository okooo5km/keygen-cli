//! Resource: webhook endpoints + events.

use clap::Subcommand;
use serde_json::json;

use crate::{
    api::Client,
    cli::Context,
    error::Result,
    output::{bag, list, single},
    resources::common::*,
};

const ENDPOINT_CRUD: Crud = Crud::new("webhook-endpoints", "/webhook-endpoints");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Manage webhook endpoints.
    #[command(subcommand)]
    Endpoint(EndpointCmd),
    /// Inspect webhook events.
    #[command(subcommand)]
    Event(EventCmd),
}

#[derive(Debug, Subcommand)]
pub enum EndpointCmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// Send a test event to an endpoint.
    Test {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum EventCmd {
    List(ListArgs),
    Get(GetArgs),
    /// Retry a failed event.
    Retry {
        id: String,
    },
}

pub async fn dispatch(ctx: &Context, cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::Endpoint(sub) => endpoint(ctx, sub).await,
        Cmd::Event(sub) => event(ctx, sub).await,
    }
}

async fn endpoint(ctx: &Context, cmd: EndpointCmd) -> Result<()> {
    match cmd {
        EndpointCmd::List(args) => list(ctx, &ENDPOINT_CRUD.list(ctx, &args).await?),
        EndpointCmd::Get(args) => single(ctx, ENDPOINT_CRUD.get(ctx, &args).await?),
        EndpointCmd::Create(args) => single(ctx, ENDPOINT_CRUD.create(ctx, &args).await?),
        EndpointCmd::Update(args) => single(ctx, ENDPOINT_CRUD.update(ctx, &args).await?),
        EndpointCmd::Delete(args) => {
            ENDPOINT_CRUD.delete(ctx, &args).await?;
            bag(ctx, json!({ "deleted": args.id }))
        }
        EndpointCmd::Test { id } => {
            let client = Client::new(ctx)?;
            let path = format!("/webhook-endpoints/{id}/actions/test");
            let doc = client
                .post::<_, crate::api::jsonapi::Resource>(&path, &json!({}))
                .await?;
            single(ctx, doc.data)
        }
    }
}

async fn event(ctx: &Context, cmd: EventCmd) -> Result<()> {
    let crud = Crud::new("webhook-events", "/webhook-events");
    match cmd {
        EventCmd::List(args) => list(ctx, &crud.list(ctx, &args).await?),
        EventCmd::Get(args) => single(ctx, crud.get(ctx, &args).await?),
        EventCmd::Retry { id } => {
            let client = Client::new(ctx)?;
            let path = format!("/webhook-events/{id}/actions/retry");
            let doc = client
                .post::<_, crate::api::jsonapi::Resource>(&path, &json!({}))
                .await?;
            single(ctx, doc.data)
        }
    }
}
