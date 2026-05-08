//! Interactive login flow.
//!
//! Walks the user through:
//! 1. Pick deployment kind (Official / CE / EE) — pre-selected from the active
//!    profile if any.
//! 2. Confirm host URL.
//! 3. Pick account (only for Official / multiplayer self-hosted).
//! 4. Enter email + password.
//! 5. POST /tokens (Basic auth) → receive an admin/user token.
//! 6. Persist into the OS keyring under the active profile name.
//!
//! Email + password are *never* persisted; only the minted token is stored.

use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use inquire::{Confirm, Password, Select, Text};
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    StatusCode,
};
use serde_json::{json, Value};

use crate::{
    api::error::map_error,
    auth::store,
    cli::Context,
    config::{
        file::{self, ConfigFile, ProfileEntry},
        profile::{AccountMode, Deployment},
    },
    error::{Error, Result},
};

const JSONAPI_TYPE: &str = "application/vnd.api+json";

pub async fn interactive(ctx: &Context) -> Result<()> {
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        return Err(Error::user(
            "login is interactive — pipe `--token <token>` instead, or run from a TTY",
        ));
    }

    println!("→ keygen-cli login");
    println!("  profile: {}\n", ctx.profile().name);

    // 1. Deployment.
    let deployment_choices = ["keygen.sh Official", "Self-hosted CE", "Self-hosted EE"];
    let chosen = Select::new("Deployment target?", deployment_choices.to_vec())
        .with_starting_cursor(match ctx.profile().deployment {
            Deployment::Official => 0,
            Deployment::Ce => 1,
            Deployment::Ee => 2,
        })
        .prompt()
        .map_err(|e| Error::user(format!("aborted: {e}")))?;
    let deployment = match chosen {
        "keygen.sh Official" => Deployment::Official,
        "Self-hosted CE" => Deployment::Ce,
        _ => Deployment::Ee,
    };

    // 2. Host.
    let default_host = if matches!(deployment, Deployment::Official) {
        "https://api.keygen.sh".to_string()
    } else {
        ctx.profile().host.as_str().to_string()
    };
    let host = Text::new("Host URL?")
        .with_default(&default_host)
        .prompt()
        .map_err(|e| Error::user(format!("aborted: {e}")))?;

    // 3. Account (mode).
    //
    // - Official: always multiplayer, must have an account.
    // - EE: defaults to singleplayer (the common shape) but can opt into
    //   multiplayer; ask.
    // - CE: always singleplayer in v0.x (the keygen.sh distro defaults to it
    //   and the multiplayer command surface is a niche use case). Skip the
    //   prompt entirely — Boss confirmed this on 2026-05-08.
    let needs_account = match deployment {
        Deployment::Official => true,
        Deployment::Ce => false,
        Deployment::Ee => Confirm::new("Multiplayer mode (account-scoped paths)?")
            .with_default(false)
            .prompt()
            .unwrap_or(false),
    };
    let account = if needs_account {
        let default = ctx.profile().account.clone().unwrap_or_default();
        Some(
            Text::new("Account id or slug?")
                .with_default(&default)
                .prompt()
                .map_err(|e| Error::user(format!("aborted: {e}")))?,
        )
    } else {
        None
    };

    // 4. Credentials.
    let email = Text::new("Email?")
        .prompt()
        .map_err(|e| Error::user(format!("aborted: {e}")))?;
    let password = Password::new("Password?")
        .without_confirmation()
        .prompt()
        .map_err(|e| Error::user(format!("aborted: {e}")))?;

    let mode = if needs_account {
        AccountMode::Multiplayer
    } else {
        AccountMode::Singleplayer
    };

    // 5. POST /tokens (Basic auth) → mint a token.
    let token = mint_token(&host, mode, account.as_deref(), &email, &password).await?;

    // 6. Persist token + profile config.
    //    The token goes into the OS keyring; everything else (deployment, host,
    //    account, mode) lands in $XDG_CONFIG_HOME/keygen/config.toml so
    //    subsequent commands resolve the same profile without flags.
    store::save_token(&ctx.profile().name, &token)?;
    persist_profile(
        &ctx.profile().name,
        deployment,
        &host,
        account.as_deref(),
        mode,
    )?;

    let cfg_path =
        file::config_path().map_or_else(|_| "config.toml".into(), |p| p.display().to_string());
    println!(
        "\n✓ Login successful. Token stored in OS keyring; profile `{}` saved to {}.",
        ctx.profile().name,
        cfg_path
    );
    println!("  Run `keygen whoami` to verify.");
    Ok(())
}

/// Merge the just-completed login into the on-disk config. Adds the named
/// profile if missing, updates it otherwise, and stamps it as the default
/// profile when no default exists yet.
fn persist_profile(
    name: &str,
    deployment: Deployment,
    host: &str,
    account: Option<&str>,
    mode: AccountMode,
) -> Result<()> {
    let mut cfg: ConfigFile = file::load().unwrap_or_default();
    cfg.profiles.insert(
        name.to_string(),
        ProfileEntry {
            deployment,
            host: host.to_string(),
            account: account.map(str::to_string),
            env: None,
            mode: Some(mode),
            output: None,
        },
    );
    if cfg.default_profile.is_none() {
        cfg.default_profile = Some(name.to_string());
    }
    file::save(&cfg)
}

async fn mint_token(
    host: &str,
    mode: AccountMode,
    account: Option<&str>,
    email: &str,
    password: &str,
) -> Result<String> {
    // Build URL: /v1[/accounts/<id>]/tokens
    let mut url = url::Url::parse(host).map_err(|e| Error::config(format!("invalid host: {e}")))?;
    {
        let mut seg = url
            .path_segments_mut()
            .map_err(|()| Error::config("host cannot be a base"))?;
        seg.pop_if_empty();
        seg.push("v1");
        if matches!(mode, AccountMode::Multiplayer) {
            let acc =
                account.ok_or_else(|| Error::config("multiplayer login requires an account"))?;
            seg.push("accounts");
            seg.push(acc);
        }
        seg.push("tokens");
    }

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static(JSONAPI_TYPE));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static(JSONAPI_TYPE));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(concat!("keygen-cli/", env!("CARGO_PKG_VERSION"))),
    );
    let basic = STANDARD.encode(format!("{email}:{password}"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Basic {basic}"))
            .map_err(|e| Error::user(format!("invalid credential bytes: {e}")))?,
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .build()?;

    let resp = client.post(url).send().await?;
    let status = resp.status();
    let request_id = resp
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = resp.bytes().await?;

    if status == StatusCode::CREATED || status.is_success() {
        let doc: Value = serde_json::from_slice(&bytes)?;
        let token = doc
            .pointer("/data/attributes/token")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::user("login response missing data.attributes.token"))?;
        return Ok(token.to_string());
    }
    Err(map_error(status.as_u16(), request_id, &bytes))
}

// Avoid pulling miette/json-everywhere by declaring the json! macro is in scope.
#[allow(dead_code)]
fn _unused_marker() -> Value {
    json!(null)
}
