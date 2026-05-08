use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    cli::{context::LayoutMode, globals::GlobalArgs},
    error::Result,
};

use super::file;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Deployment {
    #[default]
    Official,
    Ce,
    Ee,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountMode {
    Singleplayer,
    #[default]
    Multiplayer,
}

/// Resolved profile: the merged view of CLI flags + env vars + on-disk config.
#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub deployment: Deployment,
    pub host: Url,
    pub account: Option<String>,
    pub env: Option<String>,
    pub mode: AccountMode,
    pub token_override: Option<String>,
    pub default_layout: Option<LayoutMode>,
}

impl Profile {
    /// Build a runtime profile from CLI globals layered on top of the on-disk
    /// config. Resolution order (highest precedence first):
    ///
    /// 1. CLI flag (e.g. `--host`)
    /// 2. Environment variable (already folded into `globals` by clap)
    /// 3. The named profile in `config.toml`
    /// 4. The `default_profile` in `config.toml`
    /// 5. Hard-coded fallbacks (Official / `https://api.keygen.sh`)
    pub fn resolve(globals: &GlobalArgs) -> Result<Self> {
        let cfg = file::load().unwrap_or_default();
        let name = globals
            .profile
            .clone()
            .or_else(|| cfg.default_profile.clone())
            .unwrap_or_else(|| "default".into());
        let entry = cfg.profiles.get(&name).cloned();

        let host_str = globals
            .host
            .clone()
            .or_else(|| entry.as_ref().map(|e| e.host.clone()))
            .unwrap_or_else(|| "https://api.keygen.sh".to_string());
        let host = Url::parse(&host_str)
            .map_err(|e| crate::Error::config(format!("invalid host {host_str}: {e}")))?;

        let deployment = entry.as_ref().map_or_else(
            || {
                if host.host_str() == Some("api.keygen.sh") {
                    Deployment::Official
                } else {
                    Deployment::Ce
                }
            },
            |e| e.deployment,
        );

        let account = globals
            .account
            .clone()
            .or_else(|| entry.as_ref().and_then(|e| e.account.clone()));
        let env = globals
            .env
            .clone()
            .or_else(|| entry.as_ref().and_then(|e| e.env.clone()));
        let mode = entry.as_ref().and_then(|e| e.mode).unwrap_or(
            if matches!(deployment, Deployment::Official) {
                AccountMode::Multiplayer
            } else {
                AccountMode::Singleplayer
            },
        );

        let default_layout = entry
            .as_ref()
            .and_then(|e| e.default_layout)
            .map(Into::into);

        Ok(Self {
            name,
            deployment,
            host,
            account,
            env,
            mode,
            token_override: globals.token.clone(),
            default_layout,
        })
    }
}
