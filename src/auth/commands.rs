use clap::Args;
use serde_json::json;

use crate::{
    auth::{login as login_flow, store, token::TokenKind},
    cli::Context,
    error::Result,
};

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Use the supplied token instead of running the interactive flow.
    #[arg(long)]
    pub token: Option<String>,

    /// Token kind hint (admin / product / user / environment / license).
    #[arg(long)]
    pub kind: Option<String>,
}

#[derive(Debug, Args)]
pub struct LogoutArgs {
    /// Profile to clear. Defaults to the active profile.
    #[arg(long)]
    pub profile: Option<String>,
}

pub async fn login(ctx: &Context, args: LoginArgs) -> Result<()> {
    if let Some(token) = args.token {
        // Non-interactive: stash the supplied token under the active profile.
        store::save_token(&ctx.profile().name, &token)?;
        crate::output::json::print(&json!({
            "ok": true,
            "data": {
                "profile": ctx.profile().name,
                "stored": "keyring",
                "kind": args.kind,
            }
        }))?;
        return Ok(());
    }
    login_flow::interactive(ctx).await
}

pub async fn logout(ctx: &Context, args: LogoutArgs) -> Result<()> {
    let profile = args.profile.unwrap_or_else(|| ctx.profile().name.clone());
    store::delete_token(&profile)?;
    crate::output::json::print(&json!({
        "ok": true,
        "data": {
            "profile": profile,
            "cleared": true,
        }
    }))?;
    Ok(())
}

pub async fn whoami(ctx: &Context) -> Result<()> {
    let stored_token =
        store::load_token(&ctx.profile().name)?.is_some() || ctx.profile().token_override.is_some();

    let mut payload = json!({
        "profile": ctx.profile().name,
        "deployment": format!("{:?}", ctx.profile().deployment).to_lowercase(),
        "host": ctx.profile().host.as_str(),
        "account": ctx.profile().account,
        "env": ctx.profile().env,
        "mode": format!("{:?}", ctx.profile().mode).to_lowercase(),
        "token": stored_token,
        "token_kind": null,
        "online": false,
    });

    // Best-effort: probe `/v1/profile` (Official) or `/v1/me` (CE/EE) to find the
    // identity behind the active token. Falls back gracefully on any failure so
    // `whoami` is still useful when offline.
    if stored_token {
        if let Ok(client) = crate::api::Client::new(ctx) {
            if let Ok(doc) = client
                .get::<crate::api::jsonapi::Resource>("/profile", &crate::api::client::Query::new())
                .await
            {
                payload["online"] = json!(true);
                payload["identity"] = json!({
                    "id": doc.data.id,
                    "type": doc.data.r#type,
                    "attributes": doc.data.attributes,
                });
                if let Some(kind) = TokenKind::from_jsonapi_type(&doc.data.r#type) {
                    payload["token_kind"] = json!(format!("{kind:?}").to_lowercase());
                }
            }
        }
    }

    crate::output::json::print(&json!({ "ok": true, "data": payload }))?;
    Ok(())
}
