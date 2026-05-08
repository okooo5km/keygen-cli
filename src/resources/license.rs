//! Resource: license. Implements CRUD plus the full action surface defined in
//! the plan (validate / suspend / reinstate / renew / revoke / check-out /
//! check-in / usage / tokens / transfer).

use clap::Subcommand;
use serde_json::{json, Value};

use crate::{
    api::{client::Query, Client},
    cli::Context,
    error::Result,
    output::{bag, list, single},
    resources::common::*,
};

const CRUD: Crud = Crud::new("licenses", "/licenses");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    /// Validate a license by id.
    Validate {
        id: String,
        /// Hardware fingerprint (per machine, when validating activations).
        #[arg(long)]
        fingerprint: Option<String>,
    },
    /// Validate a license by its key (no auth required).
    ValidateKey {
        key: String,
        #[arg(long)]
        fingerprint: Option<String>,
    },
    /// Suspend a license.
    Suspend {
        id: String,
    },
    /// Reinstate a suspended license.
    Reinstate {
        id: String,
    },
    /// Renew a license.
    Renew {
        id: String,
    },
    /// Revoke a license.
    Revoke {
        id: String,
        #[arg(long)]
        yes: bool,
    },
    /// Check out a signed `.lic` blob (offline verification).
    CheckOut {
        id: String,
        #[arg(long)]
        out: Option<String>,
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,
    },
    /// Cancel an outstanding check-out.
    CheckIn {
        id: String,
    },
    /// Manage license usage counters.
    #[command(subcommand)]
    Usage(LicenseUsageCmd),
    /// List tokens scoped to a license.
    Tokens {
        id: String,
    },
    /// Transfer a license to a different user or policy.
    Transfer {
        id: String,
        #[arg(long)]
        to_user: Option<String>,
        #[arg(long)]
        to_policy: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum LicenseUsageCmd {
    /// Increment the usage counter.
    Incr {
        id: String,
        #[arg(long, default_value_t = 1)]
        amount: u64,
    },
    /// Decrement the usage counter.
    Decr {
        id: String,
        #[arg(long, default_value_t = 1)]
        amount: u64,
    },
    /// Reset the usage counter to zero.
    Reset { id: String },
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
        Cmd::Validate { id, fingerprint } => validate(ctx, &id, fingerprint.as_deref()).await,
        Cmd::ValidateKey { key, fingerprint } => {
            validate_key(ctx, &key, fingerprint.as_deref()).await
        }
        Cmd::Suspend { id } => action(ctx, &id, "suspend").await,
        Cmd::Reinstate { id } => action(ctx, &id, "reinstate").await,
        Cmd::Renew { id } => action(ctx, &id, "renew").await,
        Cmd::Revoke { id, yes } => revoke(ctx, &id, yes).await,
        Cmd::CheckOut { id, out, include } => check_out(ctx, &id, out.as_deref(), &include).await,
        Cmd::CheckIn { id } => action(ctx, &id, "check-in").await,
        Cmd::Usage(sub) => usage(ctx, sub).await,
        Cmd::Tokens { id } => tokens(ctx, &id).await,
        Cmd::Transfer {
            id,
            to_user,
            to_policy,
        } => transfer(ctx, &id, to_user.as_deref(), to_policy.as_deref()).await,
    }
}

async fn validate(ctx: &Context, id: &str, fingerprint: Option<&str>) -> Result<()> {
    let client = Client::new(ctx)?;
    let path = format!("/licenses/{id}/actions/validate");
    let body = match fingerprint {
        Some(fp) => json!({ "meta": { "scope": { "fingerprint": fp } } }),
        None => json!({}),
    };
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>(&path, &body)
        .await?;
    single(ctx, doc.data)
}

async fn validate_key(ctx: &Context, key: &str, fingerprint: Option<&str>) -> Result<()> {
    let client = Client::new(ctx)?;
    let mut meta = json!({ "key": key });
    if let Some(fp) = fingerprint {
        meta["scope"] = json!({ "fingerprint": fp });
    }
    let body = json!({ "meta": meta });
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>("/licenses/actions/validate-key", &body)
        .await?;
    single(ctx, doc.data)
}

async fn action(ctx: &Context, id: &str, action: &str) -> Result<()> {
    let client = Client::new(ctx)?;
    let path = format!("/licenses/{id}/actions/{action}");
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>(&path, &json!({}))
        .await?;
    single(ctx, doc.data)
}

async fn revoke(ctx: &Context, id: &str, yes: bool) -> Result<()> {
    if !yes && std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        let typed = inquire::Text::new(&format!("Revoke license {id}? Type the id to confirm: "))
            .prompt()
            .map_err(|e| crate::Error::user(format!("aborted: {e}")))?;
        if typed != id {
            return Err(crate::Error::user("revoke cancelled (id did not match)"));
        }
    }
    let client = Client::new(ctx)?;
    let path = format!("/licenses/{id}");
    client.delete(&path).await?;
    bag(ctx, json!({ "revoked": id }))
}

async fn check_out(ctx: &Context, id: &str, out: Option<&str>, include: &[String]) -> Result<()> {
    let client = Client::new(ctx)?;
    let path = format!("/licenses/{id}/actions/check-out");
    let query = Query::new().include(include.to_vec());
    // POST with query string (`include`) and an empty body.
    let url_path = if include.is_empty() {
        path.clone()
    } else {
        let qs = include.join(",");
        format!("{path}?include={qs}")
    };
    let _ = query;
    let doc = client.post::<_, Value>(&url_path, &json!({})).await?;

    // The check-out response carries the signed `.lic` payload in
    // `data.attributes.certificate` (or as the whole body for jwt schemes).
    if let Some(path) = out {
        let payload = doc
            .data
            .pointer("/attributes/certificate")
            .and_then(Value::as_str)
            .map_or_else(|| doc.data.to_string(), str::to_string);
        std::fs::write(path, payload.as_bytes())?;
        bag(
            ctx,
            json!({ "license": id, "out": path, "size_bytes": payload.len() }),
        )?;
    } else {
        single(ctx, &doc.data)?;
    }
    Ok(())
}

async fn usage(ctx: &Context, cmd: LicenseUsageCmd) -> Result<()> {
    let client = Client::new(ctx)?;
    let (id, action, body) = match cmd {
        LicenseUsageCmd::Incr { id, amount } => (
            id,
            "increment-usage",
            json!({ "meta": { "increment": amount } }),
        ),
        LicenseUsageCmd::Decr { id, amount } => (
            id,
            "decrement-usage",
            json!({ "meta": { "decrement": amount } }),
        ),
        LicenseUsageCmd::Reset { id } => (id, "reset-usage", json!({})),
    };
    let path = format!("/licenses/{id}/actions/{action}");
    let doc = client
        .post::<_, crate::api::jsonapi::Resource>(&path, &body)
        .await?;
    single(ctx, doc.data)
}

async fn tokens(ctx: &Context, id: &str) -> Result<()> {
    let client = Client::new(ctx)?;
    let path = format!("/licenses/{id}/tokens");
    let doc = client
        .get::<Vec<crate::api::jsonapi::Resource>>(&path, &Query::new())
        .await?;
    list(ctx, &doc.data)
}

async fn transfer(
    ctx: &Context,
    id: &str,
    to_user: Option<&str>,
    to_policy: Option<&str>,
) -> Result<()> {
    if to_user.is_none() && to_policy.is_none() {
        return Err(crate::Error::user(
            "transfer requires --to-user <id> or --to-policy <id>",
        ));
    }
    let mut relationships = serde_json::Map::new();
    if let Some(uid) = to_user {
        relationships.insert(
            "user".into(),
            json!({ "data": { "type": "users", "id": uid } }),
        );
    }
    if let Some(pid) = to_policy {
        relationships.insert(
            "policy".into(),
            json!({ "data": { "type": "policies", "id": pid } }),
        );
    }
    let body = json!({
        "data": {
            "type": "licenses",
            "id": id,
            "relationships": relationships,
        }
    });
    let client = Client::new(ctx)?;
    let path = format!("/licenses/{id}");
    let doc = client
        .patch::<_, crate::api::jsonapi::Resource>(&path, &body)
        .await?;
    single(ctx, doc.data)
}
