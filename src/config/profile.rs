use serde::{Deserialize, Serialize};
use url::Url;

use crate::{cli::globals::GlobalArgs, error::Result};

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
}

impl Profile {
    /// Build a runtime profile from CLI globals. The on-disk config layer
    /// (`file::load`) is plugged in here as it lands; for now we honour flags
    /// and env vars only so the skeleton compiles.
    pub fn resolve(globals: &GlobalArgs) -> Result<Self> {
        let host_str = globals
            .host
            .clone()
            .unwrap_or_else(|| "https://api.keygen.sh".to_string());
        let host = Url::parse(&host_str)
            .map_err(|e| crate::Error::config(format!("invalid host {host_str}: {e}")))?;

        let deployment = if host.host_str() == Some("api.keygen.sh") {
            Deployment::Official
        } else {
            Deployment::Ce
        };

        Ok(Self {
            name: globals.profile.clone().unwrap_or_else(|| "default".into()),
            deployment,
            host,
            account: globals.account.clone(),
            env: globals.env.clone(),
            mode: if matches!(deployment, Deployment::Official) {
                AccountMode::Multiplayer
            } else {
                AccountMode::Singleplayer
            },
            token_override: globals.token.clone(),
        })
    }
}
