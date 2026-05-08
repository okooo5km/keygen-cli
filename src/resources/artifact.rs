//! Resource: release artifact (binary blob attached to a release).
//!
//! `upload` and `download` use a two-step keygen.sh flow: a metadata POST/GET
//! returns a redirect URL pointing at object storage; the CLI then PUTs/GETs
//! the actual bytes there.

use std::path::Path;

use clap::Subcommand;
use reqwest::header::CONTENT_LENGTH;
use serde_json::{json, Value};
use tokio::fs;

use crate::{
    api::{client::Query, Client},
    cli::Context,
    error::{Error, Result},
    output::{bag, list, single},
    render::progress::bytes_bar,
    resources::common::*,
};

const CRUD: Crud = Crud::new("artifacts", "/artifacts");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Delete(DeleteArgs),
    /// Upload a binary to a release.
    Upload {
        release: String,
        #[arg(long)]
        file: String,
        #[arg(long)]
        platform: Option<String>,
        #[arg(long)]
        arch: Option<String>,
        #[arg(long)]
        filetype: Option<String>,
        #[arg(long)]
        signature: Option<String>,
        #[arg(long)]
        checksum: Option<String>,
    },
    /// Download a binary by id.
    Download {
        id: String,
        #[arg(long)]
        out: Option<String>,
    },
    /// Yank an artifact.
    Yank {
        id: String,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn dispatch(ctx: &Context, cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::List(args) => list(ctx, &CRUD.list(ctx, &args).await?),
        Cmd::Get(args) => single(ctx, CRUD.get(ctx, &args).await?),
        Cmd::Delete(args) => {
            CRUD.delete(ctx, &args).await?;
            bag(ctx, json!({ "deleted": args.id }))
        }
        Cmd::Upload {
            release,
            file,
            platform,
            arch,
            filetype,
            signature,
            checksum,
        } => {
            upload(
                ctx,
                &release,
                &file,
                platform.as_deref(),
                arch.as_deref(),
                filetype.as_deref(),
                signature.as_deref(),
                checksum.as_deref(),
            )
            .await
        }
        Cmd::Download { id, out } => download(ctx, &id, out.as_deref()).await,
        Cmd::Yank { id, yes } => {
            CRUD.delete(
                ctx,
                &DeleteArgs {
                    id: id.clone(),
                    yes,
                },
            )
            .await?;
            bag(ctx, json!({ "yanked": id }))
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn upload(
    ctx: &Context,
    release: &str,
    file: &str,
    platform: Option<&str>,
    arch: Option<&str>,
    filetype: Option<&str>,
    signature: Option<&str>,
    checksum: Option<&str>,
) -> Result<()> {
    let path = Path::new(file);
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::user("--file path must have a file name"))?;

    let bytes = fs::read(path).await?;
    let total = bytes.len() as u64;

    let mut attributes = serde_json::Map::new();
    attributes.insert("filename".into(), json!(filename));
    attributes.insert("filesize".into(), json!(total));
    if let Some(p) = platform {
        attributes.insert("platform".into(), json!(p));
    }
    if let Some(a) = arch {
        attributes.insert("arch".into(), json!(a));
    }
    if let Some(t) = filetype {
        attributes.insert("filetype".into(), json!(t));
    }
    if let Some(s) = signature {
        attributes.insert("signature".into(), json!(s));
    }
    if let Some(c) = checksum {
        attributes.insert("checksum".into(), json!(c));
    }

    let body = json!({
        "data": {
            "type": "artifacts",
            "attributes": attributes,
            "relationships": {
                "release": { "data": { "type": "releases", "id": release } }
            }
        }
    });

    // Step 1: register the artifact + receive a pre-signed upload URL.
    let client = Client::new(ctx)?;
    let doc = client.post::<_, Value>("/artifacts", &body).await?;
    let upload_url = doc
        .links
        .as_ref()
        .and_then(|l| l.get("redirect"))
        .and_then(Value::as_str)
        .or_else(|| doc.data.pointer("/links/redirect").and_then(Value::as_str))
        .ok_or_else(|| Error::user("upload response missing the upload redirect URL"))?
        .to_string();

    // Step 2: PUT bytes to the storage URL.
    let pb = bytes_bar(total, format!("uploading {filename}"));
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?;
    let resp = http
        .put(&upload_url)
        .header(CONTENT_LENGTH, total)
        .body(bytes)
        .send()
        .await?;
    pb.finish_and_clear();

    if !resp.status().is_success() {
        return Err(Error::user(format!(
            "upload failed: storage returned {}",
            resp.status()
        )));
    }

    bag(
        ctx,
        json!({
            "uploaded": filename,
            "release": release,
            "size_bytes": total,
            "artifact": doc.data.pointer("/id").cloned().unwrap_or(Value::Null),
        }),
    )
}

async fn download(ctx: &Context, id: &str, out: Option<&str>) -> Result<()> {
    let client = Client::new(ctx)?;
    // The artifact GET returns a redirect to the signed download URL.
    let doc = client
        .get::<Value>(&format!("/artifacts/{id}"), &Query::new())
        .await?;
    let download_url = doc
        .links
        .as_ref()
        .and_then(|l| l.get("redirect"))
        .and_then(Value::as_str)
        .or_else(|| doc.data.pointer("/links/redirect").and_then(Value::as_str))
        .ok_or_else(|| Error::user("download response missing the redirect URL"))?
        .to_string();
    let filename = doc
        .data
        .pointer("/attributes/filename")
        .and_then(Value::as_str)
        .unwrap_or(id)
        .to_string();
    let dest = out.map_or_else(|| filename.clone(), str::to_string);

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?;
    let resp = http.get(&download_url).send().await?;
    if !resp.status().is_success() {
        return Err(Error::user(format!(
            "download failed: storage returned {}",
            resp.status()
        )));
    }
    let total = resp.content_length().unwrap_or(0);
    let pb = if total > 0 {
        bytes_bar(total, format!("downloading {filename}"))
    } else {
        crate::render::progress::spinner(format!("downloading {filename}"))
    };
    let bytes = resp.bytes().await?;
    fs::write(&dest, &bytes).await?;
    pb.finish_and_clear();
    bag(
        ctx,
        json!({
            "downloaded": dest,
            "size_bytes": bytes.len(),
            "artifact": id,
        }),
    )
}
