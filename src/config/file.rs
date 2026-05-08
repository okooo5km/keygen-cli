use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_layout: Option<DefaultLayout>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DefaultLayout {
    Table,
    Cards,
}

impl From<DefaultLayout> for crate::cli::context::LayoutMode {
    fn from(v: DefaultLayout) -> Self {
        match v {
            DefaultLayout::Table => Self::Table,
            DefaultLayout::Cards => Self::Cards,
        }
    }
}

/// Resolve `~/.config/keygen/` (or `$XDG_CONFIG_HOME/keygen`) on Unix —
/// including macOS, where the `directories` crate would otherwise hand us
/// `~/Library/Application Support/...`. Windows keeps Microsoft's known-folder
/// paths via `directories::ProjectDirs`.
pub fn config_dir() -> Result<PathBuf> {
    xdg_dir_or_fallback("XDG_CONFIG_HOME", ".config")
}

pub fn cache_dir() -> Result<PathBuf> {
    xdg_dir_or_fallback("XDG_CACHE_HOME", ".cache")
}

pub fn data_dir() -> Result<PathBuf> {
    xdg_dir_or_fallback("XDG_DATA_HOME", ".local/share")
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

#[cfg(unix)]
fn xdg_dir_or_fallback(env_var: &str, default_segment: &str) -> Result<PathBuf> {
    use std::env;
    use std::path::Path;

    if let Some(raw) = env::var_os(env_var) {
        let p = Path::new(&raw);
        if p.is_absolute() {
            return Ok(p.join("keygen"));
        }
    }
    let home = env::var_os("HOME").ok_or_else(|| Error::config("HOME env var not set"))?;
    Ok(PathBuf::from(home).join(default_segment).join("keygen"))
}

#[cfg(windows)]
fn xdg_dir_or_fallback(env_var: &str, _default_segment: &str) -> Result<PathBuf> {
    use directories::ProjectDirs;
    const QUALIFIER: &str = "sh";
    const ORG: &str = "keygen";
    const APP: &str = "keygen";

    let dirs = ProjectDirs::from(QUALIFIER, ORG, APP)
        .ok_or_else(|| Error::config("could not determine config directories"))?;
    Ok(match env_var {
        "XDG_CACHE_HOME" => dirs.cache_dir().to_path_buf(),
        "XDG_DATA_HOME" => dirs.data_dir().to_path_buf(),
        _ => dirs.config_dir().to_path_buf(),
    })
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
