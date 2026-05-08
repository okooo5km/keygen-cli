use clap::Subcommand;
use serde_json::json;

use crate::{
    api::{client::Query, Client},
    capability,
    cli::Context,
    config::profile::Deployment,
    error::{Error, Result},
};

#[derive(Debug, Subcommand)]
pub enum EnvCmd {
    /// List EE environments.
    List,
    /// Switch the active environment.
    Use { id: String },
    /// Print the active environment.
    Current,
}

pub async fn dispatch(ctx: &Context, cmd: EnvCmd) -> Result<()> {
    let caps = capability::detect::resolve(ctx).await;
    if !caps.environments && matches!(ctx.profile().deployment, Deployment::Ce) {
        return Err(Error::capability(
            "environments require keygen.sh Official or EE",
        ));
    }

    match cmd {
        EnvCmd::List => {
            let client = Client::new(ctx)?;
            let doc = client
                .get::<Vec<crate::api::jsonapi::Resource>>(
                    "/environments",
                    &Query::new().page(1, 100),
                )
                .await?;
            crate::output::list(ctx, &doc.data)
        }
        EnvCmd::Use { id } => crate::output::bag(
            ctx,
            json!({
                "env": id,
                "hint": "set KEYGEN_ENV or pass --env <id> to use it on subsequent calls",
            }),
        ),
        EnvCmd::Current => crate::output::bag(ctx, json!({ "env": ctx.profile().env })),
    }
}
