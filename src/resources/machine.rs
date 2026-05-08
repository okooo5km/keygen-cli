//! Resource: machine (license activation). CRUD plus heartbeat / reset /
//! check-out. CRUD `create` is exposed as `activate` and `delete` as
//! `deactivate` to match keygen.sh terminology.

use clap::Subcommand;
use serde_json::json;

use crate::{api::Client, cli::Context, error::Result, resources::common::*};

const CRUD: Crud = Crud::new("machines", "/machines");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    /// Activate (create) a machine for a license.
    Activate {
        #[arg(long)]
        license: String,
        #[arg(long)]
        fingerprint: String,
        #[arg(long)]
        platform: Option<String>,
        #[arg(long, value_name = "K=V")]
        metadata: Vec<String>,
    },
    /// Deactivate (delete) a machine.
    Deactivate {
        id: String,
        #[arg(long)]
        yes: bool,
    },
    Update(UpdateArgs),
    /// Send a heartbeat ping.
    Ping {
        id: String,
    },
    /// Reset the machine's heartbeat counter.
    Reset {
        id: String,
    },
    /// Check out a machine for offline use.
    CheckOut {
        id: String,
        #[arg(long)]
        out: Option<String>,
    },
}

pub async fn dispatch(ctx: &Context, cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::List(args) => emit(CRUD.list(ctx, &args).await?),
        Cmd::Get(args) => emit(CRUD.get(ctx, &args).await?),
        Cmd::Activate {
            license,
            fingerprint,
            platform,
            metadata,
        } => activate(ctx, &license, &fingerprint, platform.as_deref(), &metadata).await,
        Cmd::Deactivate { id, yes } => {
            CRUD.delete(
                ctx,
                &DeleteArgs {
                    id: id.clone(),
                    yes,
                },
            )
            .await?;
            crate::output::json::print(&json!({ "ok": true, "data": { "deactivated": id } }))
        }
        Cmd::Update(args) => emit(CRUD.update(ctx, &args).await?),
        Cmd::Ping { id } => action(ctx, &id, "ping").await,
        Cmd::Reset { id } => action(ctx, &id, "reset").await,
        Cmd::CheckOut { id, out } => check_out(ctx, &id, out.as_deref()).await,
    }
}

async fn activate(
    ctx: &Context,
    license: &str,
    fingerprint: &str,
    platform: Option<&str>,
    metadata: &[String],
) -> Result<()> {
    let mut attributes = serde_json::Map::new();
    attributes.insert("fingerprint".into(), json!(fingerprint));
    if let Some(p) = platform {
        attributes.insert("platform".into(), json!(p));
    }
    if !metadata.is_empty() {
        let mut m = serde_json::Map::new();
        for raw in metadata {
            if let Some((k, v)) = raw.split_once('=') {
                m.insert(k.to_string(), json!(v));
            }
        }
        attributes.insert("metadata".into(), serde_json::Value::Object(m));
    }
    let body = json!({
        "data": {
            "type": "machines",
            "attributes": attributes,
            "relationships": {
                "license": { "data": { "type": "licenses", "id": license } }
            }
        }
    });
    let client = Client::new(ctx)?;
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>("/machines", &body)
        .await?;
    emit(doc.data)
}

async fn action(ctx: &Context, id: &str, action: &str) -> Result<()> {
    let client = Client::new(ctx)?;
    let path = format!("/machines/{id}/actions/{action}");
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>(&path, &json!({}))
        .await?;
    emit(doc.data)
}

async fn check_out(ctx: &Context, id: &str, out: Option<&str>) -> Result<()> {
    let client = Client::new(ctx)?;
    let path = format!("/machines/{id}/actions/check-out");
    let doc = client
        .post::<_, serde_json::Value>(&path, &json!({}))
        .await?;
    if let Some(out_path) = out {
        let payload = doc
            .data
            .pointer("/attributes/certificate")
            .and_then(serde_json::Value::as_str)
            .map_or_else(|| doc.data.to_string(), str::to_string);
        std::fs::write(out_path, payload.as_bytes())?;
        crate::output::json::print(&json!({
            "ok": true,
            "data": { "machine": id, "out": out_path, "size_bytes": payload.len() }
        }))
    } else {
        emit(doc.data)
    }
}

fn emit<T: serde::Serialize>(data: T) -> Result<()> {
    crate::output::json::print(&json!({ "ok": true, "data": data }))
}
