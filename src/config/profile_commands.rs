use clap::Subcommand;
use serde_json::json;

use crate::{
    auth::store,
    cli::Context,
    config::file,
    error::{Error, Result},
    output,
};

#[derive(Debug, Subcommand)]
pub enum ProfileCmd {
    /// List all profiles.
    List,
    /// Mark a profile as the default.
    Use { name: String },
    /// Add a new profile.
    Add {
        name: String,
        #[arg(long, default_value = "https://api.keygen.sh")]
        host: String,
        #[arg(long, default_value = "official")]
        deployment: String,
        #[arg(long)]
        account: Option<String>,
    },
    /// Remove a profile (also clears its keyring entry).
    Remove { name: String },
}

pub async fn dispatch(ctx: &Context, cmd: ProfileCmd) -> Result<()> {
    match cmd {
        ProfileCmd::List => {
            let cfg = file::load()?;
            output::bag(
                ctx,
                json!({
                    "default": cfg.default_profile,
                    "profiles": cfg.profiles,
                }),
            )
        }
        ProfileCmd::Use { name } => {
            let mut cfg = file::load()?;
            if !cfg.profiles.contains_key(&name) {
                return Err(Error::user(format!("profile `{name}` not found")));
            }
            cfg.default_profile = Some(name.clone());
            file::save(&cfg)?;
            output::bag(ctx, json!({ "default_profile": name }))
        }
        ProfileCmd::Add {
            name,
            host,
            deployment,
            account,
        } => {
            let mut cfg = file::load()?;
            let dep = match deployment.as_str() {
                "official" => crate::config::Deployment::Official,
                "ce" => crate::config::Deployment::Ce,
                "ee" => crate::config::Deployment::Ee,
                other => {
                    return Err(Error::user(format!(
                        "deployment must be official|ce|ee, got `{other}`"
                    )))
                }
            };
            cfg.profiles.insert(
                name.clone(),
                file::ProfileEntry {
                    deployment: dep,
                    host,
                    account,
                    env: None,
                    mode: None,
                    output: None,
                    default_layout: None,
                },
            );
            file::save(&cfg)?;
            output::bag(ctx, json!({ "added": name }))
        }
        ProfileCmd::Remove { name } => {
            let mut cfg = file::load()?;
            if cfg.profiles.remove(&name).is_none() {
                return Err(Error::user(format!("profile `{name}` not found")));
            }
            if cfg.default_profile.as_deref() == Some(name.as_str()) {
                cfg.default_profile = None;
            }
            file::save(&cfg)?;
            store::delete_token(&name).ok();
            output::bag(ctx, json!({ "removed": name }))
        }
    }
}
