//! Resource: license. Implements CRUD plus the full action surface defined in
//! the plan (validate / suspend / reinstate / renew / revoke / check-out /
//! check-in / usage / tokens / transfer).

use clap::{Args, Subcommand};
use serde_json::{json, Value};

use crate::{
    api::{client::Query, Client},
    cli::Context,
    error::Result,
    output::{bag, list, single, single_with_meta},
    resources::common::*,
};

const CRUD: Crud = Crud::new("licenses", "/licenses");

#[derive(Debug, Subcommand)]
pub enum Cmd {
    List(ListArgs),
    Get(GetArgs),
    Create(LicenseCreateArgs),
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
    /// Offline signature verification of a license key.
    Verify(VerifyArgs),
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

#[derive(Debug, Clone, Args)]
pub struct VerifyArgs {
    /// License key to verify (the `<dataset>.<sig>` string).
    pub key: String,

    /// Verifying / public key. Hex (Ed25519) or PEM (RSA). Mutually exclusive
    /// with `--public-key-file`.
    #[arg(long, value_name = "HEX|PEM")]
    pub public_key: Option<String>,

    /// Path to a verifying key file (raw 32-byte Ed25519, hex Ed25519, or PEM).
    #[arg(long, value_name = "PATH")]
    pub public_key_file: Option<String>,

    /// Signing scheme. One of: ED25519_SIGN, RSA_2048_PKCS1_SIGN_V2.
    /// Defaults to ED25519_SIGN.
    #[arg(long, default_value = "ED25519_SIGN")]
    pub scheme: String,
}

/// `keygen license create` with extra relationship shortcut flags.
#[derive(Debug, Clone, Args)]
pub struct LicenseCreateArgs {
    #[command(flatten)]
    pub base: CreateArgs,

    /// Policy id to attach (sets relationships.policy).
    #[arg(long)]
    pub policy: Option<String>,

    /// User id to attach (sets relationships.user).
    #[arg(long)]
    pub user: Option<String>,

    /// Group id to attach (sets relationships.group).
    #[arg(long)]
    pub group: Option<String>,
}

impl LicenseCreateArgs {
    /// Fold the relationship shortcuts back into the base `--set` overrides.
    fn to_create_args(&self) -> CreateArgs {
        let mut base = self.base.clone();
        if let Some(pid) = &self.policy {
            base.set
                .push("data.relationships.policy.data.type=\"policies\"".into());
            base.set
                .push(format!("data.relationships.policy.data.id=\"{pid}\""));
        }
        if let Some(uid) = &self.user {
            base.set
                .push("data.relationships.user.data.type=\"users\"".into());
            base.set
                .push(format!("data.relationships.user.data.id=\"{uid}\""));
        }
        if let Some(gid) = &self.group {
            base.set
                .push("data.relationships.group.data.type=\"groups\"".into());
            base.set
                .push(format!("data.relationships.group.data.id=\"{gid}\""));
        }
        base
    }
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
        Cmd::Create(args) => {
            let base = args.to_create_args();
            single(ctx, CRUD.create(ctx, &base).await?)
        }
        Cmd::Update(args) => single(ctx, CRUD.update(ctx, &args).await?),
        Cmd::Delete(args) => {
            CRUD.delete(ctx, &args).await?;
            bag(ctx, json!({ "deleted": args.id }))
        }
        Cmd::Validate { id, fingerprint } => validate(ctx, &id, fingerprint.as_deref()).await,
        Cmd::ValidateKey { key, fingerprint } => {
            validate_key(ctx, &key, fingerprint.as_deref()).await
        }
        Cmd::Verify(args) => verify_offline(ctx, &args),
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
    single_with_meta(ctx, doc.data, doc.meta)
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
    single_with_meta(ctx, doc.data, doc.meta)
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

/// Offline signature verification. Splits the key into `<encoded>.<sig>` and
/// runs the appropriate verifier. The convention used by keygen.sh is to sign
/// the bytes `key/<encoded>` with the product's signing key.
fn verify_offline(ctx: &Context, args: &VerifyArgs) -> Result<()> {
    use base64::Engine;

    let (encoded, sig_b64) = args
        .key
        .split_once('.')
        .ok_or_else(|| crate::Error::user("license key must be in `<dataset>.<sig>` form"))?;

    let signing_data = format!("key/{encoded}");
    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(sig_b64)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(sig_b64))
        .map_err(|e| crate::Error::user(format!("signature is not valid base64: {e}")))?;

    let key_pem_or_hex = read_public_key(args)?;

    let scheme = args.scheme.to_ascii_uppercase();
    let outcome = match scheme.as_str() {
        "ED25519_SIGN" => verify_ed25519(&key_pem_or_hex, signing_data.as_bytes(), &signature),
        "RSA_2048_PKCS1_SIGN_V2" => {
            verify_rsa_pkcs1(&key_pem_or_hex, signing_data.as_bytes(), &signature)
        }
        other => Err(format!("unsupported scheme `{other}`")),
    };

    let payload = match base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(encoded))
    {
        Ok(b) => match std::str::from_utf8(&b) {
            Ok(s) => json!({ "raw": s }),
            Err(_) => json!({ "raw_hex": hex_encode(&b) }),
        },
        Err(_) => Value::Null,
    };

    match outcome {
        Ok(()) => crate::output::single(
            ctx,
            json!({
                "valid": true,
                "scheme": scheme,
                "dataset": payload,
            }),
        ),
        Err(reason) => Err(crate::Error::user(format!(
            "signature mismatch ({scheme}): {reason}"
        ))),
    }
}

fn read_public_key(args: &VerifyArgs) -> Result<String> {
    if let Some(s) = &args.public_key {
        return Ok(s.clone());
    }
    if let Some(path) = &args.public_key_file {
        return std::fs::read_to_string(path)
            .map_err(|e| crate::Error::user(format!("cannot read public key file: {e}")));
    }
    Err(crate::Error::user(
        "verify requires --public-key <hex|pem> or --public-key-file <path>",
    ))
}

fn verify_ed25519(key: &str, data: &[u8], sig: &[u8]) -> std::result::Result<(), String> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let trimmed = key.trim();
    let key_bytes = if trimmed.starts_with("-----BEGIN") {
        // PEM
        ed25519_pem_bytes(trimmed)?
    } else {
        // hex (32 bytes / 64 chars)
        hex_decode(trimmed).map_err(|e| format!("invalid hex public key: {e}"))?
    };
    if key_bytes.len() != 32 {
        return Err(format!(
            "ed25519 public key must be 32 bytes (got {})",
            key_bytes.len()
        ));
    }
    let arr: [u8; 32] = key_bytes.try_into().expect("len-checked");
    let vk = VerifyingKey::from_bytes(&arr).map_err(|e| e.to_string())?;
    let sig = Signature::from_slice(sig).map_err(|e| e.to_string())?;
    vk.verify(data, &sig).map_err(|e| e.to_string())
}

fn verify_rsa_pkcs1(pem: &str, data: &[u8], sig: &[u8]) -> std::result::Result<(), String> {
    use rsa::pkcs1v15::{Signature, VerifyingKey};
    use rsa::signature::Verifier;
    use rsa::{pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey, sha2::Sha256, RsaPublicKey};

    let pk = RsaPublicKey::from_public_key_pem(pem)
        .or_else(|_| RsaPublicKey::from_pkcs1_pem(pem))
        .map_err(|e| format!("invalid RSA public key: {e}"))?;
    let vk = VerifyingKey::<Sha256>::new(pk);
    let sig = Signature::try_from(sig).map_err(|e| e.to_string())?;
    vk.verify(data, &sig).map_err(|e| e.to_string())
}

fn ed25519_pem_bytes(pem: &str) -> std::result::Result<Vec<u8>, String> {
    use ed25519_dalek::pkcs8::DecodePublicKey;
    let vk = ed25519_dalek::VerifyingKey::from_public_key_pem(pem)
        .map_err(|e| format!("ed25519 PEM parse: {e}"))?;
    Ok(vk.to_bytes().to_vec())
}

fn hex_decode(s: &str) -> std::result::Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("odd length".into());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string())?;
        out.push(byte);
    }
    Ok(out)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
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
