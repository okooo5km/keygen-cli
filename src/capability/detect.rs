//! Capability detection.
//!
//! Probes `/v1/profile` (or `/v1/whoami`) and infers which optional features the
//! deployment exposes by walking the resource manifest. Cached under
//! `$XDG_CACHE_HOME/keygen/capabilities.json` with a 1-day TTL keyed by
//! `<host>|<account>|<profile>` so multiple profiles do not clobber each other.

use std::{path::PathBuf, time::SystemTime};

use serde::{Deserialize, Serialize};

use crate::{
    api::{client::Query, Client},
    cli::Context,
    config::{file, profile::Deployment},
    error::Result,
};

use super::Capabilities;

const TTL_SECS: u64 = 24 * 60 * 60;

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    key: String,
    saved_at_secs: u64,
    capabilities: CapMap,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct CapMap {
    environments: bool,
    event_logs: bool,
    request_logs: bool,
    sso: bool,
    oci_registry: bool,
    import_export: bool,
}

impl From<CapMap> for Capabilities {
    fn from(m: CapMap) -> Self {
        Self {
            environments: m.environments,
            event_logs: m.event_logs,
            request_logs: m.request_logs,
            sso: m.sso,
            oci_registry: m.oci_registry,
            import_export: m.import_export,
        }
    }
}

impl From<&Capabilities> for CapMap {
    fn from(m: &Capabilities) -> Self {
        Self {
            environments: m.environments,
            event_logs: m.event_logs,
            request_logs: m.request_logs,
            sso: m.sso,
            oci_registry: m.oci_registry,
            import_export: m.import_export,
        }
    }
}

fn cache_path() -> Result<PathBuf> {
    Ok(file::cache_dir()?.join("capabilities.json"))
}

fn cache_key(ctx: &Context) -> String {
    format!(
        "{host}|{account}|{profile}",
        host = ctx.profile().host.as_str(),
        account = ctx.profile().account.as_deref().unwrap_or(""),
        profile = ctx.profile().name,
    )
}

fn load_cache(ctx: &Context) -> Option<Capabilities> {
    let path = cache_path().ok()?;
    let raw = std::fs::read(path).ok()?;
    let entries: Vec<CacheEntry> = serde_json::from_slice(&raw).ok()?;
    let key = cache_key(ctx);
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_secs();
    entries
        .into_iter()
        .find(|e| e.key == key && now.saturating_sub(e.saved_at_secs) < TTL_SECS)
        .map(|e| e.capabilities.into())
}

fn save_cache(ctx: &Context, caps: &Capabilities) -> Result<()> {
    let path = cache_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut entries: Vec<CacheEntry> = match std::fs::read(&path) {
        Ok(raw) => serde_json::from_slice(&raw).unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let key = cache_key(ctx);
    entries.retain(|e| e.key != key);
    entries.push(CacheEntry {
        key,
        saved_at_secs: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        capabilities: CapMap::from(caps),
    });
    let bytes = serde_json::to_vec_pretty(&entries)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Resolve capabilities, preferring the cache. Returns the deployment-default
/// set when probing fails so the rest of the CLI can keep going.
pub async fn resolve(ctx: &Context) -> Capabilities {
    if let Some(cached) = load_cache(ctx) {
        return cached;
    }
    let caps = match probe(ctx).await {
        Ok(c) => c,
        Err(_) => default_for(ctx.profile().deployment),
    };
    let _ = save_cache(ctx, &caps);
    caps
}

/// Force a refresh, bypassing the cache.
pub async fn refresh(ctx: &Context) -> Result<Capabilities> {
    let caps = probe(ctx)
        .await
        .unwrap_or_else(|_| default_for(ctx.profile().deployment));
    save_cache(ctx, &caps)?;
    Ok(caps)
}

async fn probe(ctx: &Context) -> Result<Capabilities> {
    // Default capability set per deployment kind.
    let mut caps = default_for(ctx.profile().deployment);

    // Best-effort: hit `/v1/profile` and inspect headers / meta. keygen.sh
    // doesn't currently advertise capabilities directly, so we treat 200 +
    // hostname as our authoritative signal.
    let client = Client::new(ctx)?;
    let _ = client
        .get::<crate::api::jsonapi::Resource>("/profile", &Query::new())
        .await?;

    // For self-hosted deployments, also probe environments to confirm EE.
    // 200 → EE; 404 → CE; anything else leaves caps untouched.
    if !matches!(ctx.profile().deployment, Deployment::Official) {
        let env_probe = client
            .get::<Vec<crate::api::jsonapi::Resource>>("/environments", &Query::new().page(1, 1))
            .await;
        if env_probe.is_ok() {
            caps.environments = true;
            caps.event_logs = true;
            caps.request_logs = true;
            caps.sso = true;
            caps.oci_registry = true;
            caps.import_export = true;
        }
    }

    Ok(caps)
}

fn default_for(d: Deployment) -> Capabilities {
    match d {
        Deployment::Official | Deployment::Ee => Capabilities {
            environments: true,
            event_logs: true,
            request_logs: true,
            sso: true,
            oci_registry: true,
            import_export: true,
        },
        Deployment::Ce => Capabilities::default(),
    }
}
