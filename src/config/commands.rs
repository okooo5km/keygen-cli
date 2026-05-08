use clap::Subcommand;
use serde_json::json;

use crate::{
    cli::Context,
    config::file,
    error::{Error, Result},
    output,
};

#[derive(Debug, Subcommand)]
pub enum ConfigCmd {
    /// Set a config key (e.g. `default_profile`, `profiles.<name>.host`).
    Set { key: String, value: String },
    /// Get a config key.
    Get { key: String },
    /// List all config keys.
    List,
    /// Print the path of the config / credentials / cache files.
    Path,
}

pub async fn dispatch(ctx: &Context, cmd: ConfigCmd) -> Result<()> {
    match cmd {
        ConfigCmd::Path => output::bag(
            ctx,
            json!({
                "config": file::config_path()?.display().to_string(),
                "data":   file::data_dir()?.display().to_string(),
                "cache":  file::cache_dir()?.display().to_string(),
                "keyring_service": "sh.keygen.cli",
            }),
        ),
        ConfigCmd::List => {
            let cfg = file::load()?;
            output::bag(ctx, serde_json::to_value(&cfg)?)
        }
        ConfigCmd::Get { key } => {
            let cfg = file::load()?;
            let value = lookup(&cfg, &key)?;
            output::bag(ctx, json!({ "key": key, "value": value }))
        }
        ConfigCmd::Set { key, value } => {
            let mut cfg = file::load()?;
            assign(&mut cfg, &key, &value)?;
            file::save(&cfg)?;
            output::bag(ctx, json!({ "key": key, "value": value }))
        }
    }
}

fn lookup(cfg: &file::ConfigFile, key: &str) -> Result<serde_json::Value> {
    if key == "default_profile" {
        return Ok(serde_json::to_value(&cfg.default_profile)?);
    }
    if let Some(rest) = key.strip_prefix("profiles.") {
        let (name, field) = rest
            .split_once('.')
            .ok_or_else(|| Error::user(format!("expected profiles.<name>.<field>, got `{key}`")))?;
        let entry = cfg.profiles.get(name).ok_or_else(|| {
            Error::user(format!(
                "profile `{name}` not found in {}",
                file::config_path().map_or("config".into(), |p| p.display().to_string())
            ))
        })?;
        let value = serde_json::to_value(entry)?;
        return Ok(value.get(field).cloned().unwrap_or(serde_json::Value::Null));
    }
    Err(Error::user(format!("unknown config key `{key}`")))
}

fn assign(cfg: &mut file::ConfigFile, key: &str, value: &str) -> Result<()> {
    if key == "default_profile" {
        cfg.default_profile = Some(value.to_string());
        return Ok(());
    }
    if let Some(rest) = key.strip_prefix("profiles.") {
        let (name, field) = rest
            .split_once('.')
            .ok_or_else(|| Error::user(format!("expected profiles.<name>.<field>, got `{key}`")))?;
        let entry = cfg
            .profiles
            .entry(name.to_string())
            .or_insert_with(|| file::ProfileEntry {
                deployment: crate::config::Deployment::default(),
                host: "https://api.keygen.sh".to_string(),
                account: None,
                env: None,
                mode: None,
                output: None,
            });
        match field {
            "deployment" => {
                entry.deployment = match value {
                    "official" => crate::config::Deployment::Official,
                    "ce" => crate::config::Deployment::Ce,
                    "ee" => crate::config::Deployment::Ee,
                    other => {
                        return Err(Error::user(format!(
                            "deployment must be one of official|ce|ee, got `{other}`"
                        )))
                    }
                };
            }
            "host" => entry.host = value.to_string(),
            "account" => entry.account = Some(value.to_string()),
            "env" => entry.env = Some(value.to_string()),
            "mode" => {
                entry.mode = match value {
                    "singleplayer" => Some(crate::config::profile::AccountMode::Singleplayer),
                    "multiplayer" => Some(crate::config::profile::AccountMode::Multiplayer),
                    other => {
                        return Err(Error::user(format!(
                            "mode must be singleplayer|multiplayer, got `{other}`"
                        )))
                    }
                };
            }
            "output" => entry.output = Some(value.to_string()),
            other => return Err(Error::user(format!("unknown profile field `{other}`"))),
        }
        return Ok(());
    }
    Err(Error::user(format!("unknown config key `{key}`")))
}
