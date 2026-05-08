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
    config::profile::{AccountMode, Deployment},
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
    let needs_account = !matches!(deployment, Deployment::Ce)
        || Confirm::new("Multiplayer mode (account-scoped paths)?")
            .with_default(false)
            .prompt()
            .unwrap_or(false);
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

    // 5. POST /tokens (Basic auth) → mint a token.
    let token = mint_token(
        &host,
        if needs_account {
            AccountMode::Multiplayer
        } else {
            AccountMode::Singleplayer
        },
        account.as_deref(),
        &email,
        &password,
    )
    .await?;

    // 6. Persist.
    store::save_token(&ctx.profile().name, &token)?;

    println!(
        "\n✓ Login successful. Token stored in OS keyring under profile `{}`.",
        ctx.profile().name
    );
    println!("  Run `keygen whoami` to verify.");
    Ok(())
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
