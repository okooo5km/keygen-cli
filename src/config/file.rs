use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const QUALIFIER: &str = "sh";
const ORG: &str = "keygen";
const APP: &str = "keygen";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: std::collections::BTreeMap<String, ProfileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEntry {
    pub deployment: super::Deployment,
    pub host: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<super::profile::AccountMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

pub fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from(QUALIFIER, ORG, APP)
        .ok_or_else(|| Error::config("could not determine config directories"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(project_dirs()?.config_dir().join("config.toml"))
}

pub fn cache_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.cache_dir().to_path_buf())
}

pub fn data_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.data_dir().to_path_buf())
}

pub fn load() -> Result<ConfigFile> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(ConfigFile::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    let cfg: ConfigFile = toml::from_str(&raw)?;
    Ok(cfg)
}

pub fn save(cfg: &ConfigFile) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = toml::to_string_pretty(cfg)?;
    std::fs::write(&path, body)?;
    Ok(())
}
