//! `keygen explain ...` subcommands.

use clap::Subcommand;
use serde_json::json;

use crate::{
    cli::Context,
    error::{Error, Result},
    output,
};

use super::catalog;

#[derive(Debug, Subcommand)]
pub enum ExplainCmd {
    /// Explain a keygen.sh API error code (cause + fix).
    Error {
        /// Error code (case-insensitive). e.g. LICENSE_SUSPENDED.
        code: Option<String>,
        /// List all known codes.
        #[arg(long)]
        list: bool,
    },
}

pub async fn dispatch(ctx: &Context, cmd: ExplainCmd) -> Result<()> {
    match cmd {
        ExplainCmd::Error { code, list } => {
            if list || code.is_none() {
                return list_all(ctx);
            }
            let code = code.unwrap();
            let entry = catalog::lookup(&code).ok_or_else(|| {
                Error::user(format!(
                    "unknown error code `{code}` (try `keygen explain error --list`)"
                ))
            })?;
            output::single(
                ctx,
                json!({
                    "code": entry.code,
                    "title": entry.title,
                    "cause": entry.cause,
                    "fix": entry.fix,
                    "see_also": entry.see_also,
                }),
            )
        }
    }
}

fn list_all(ctx: &Context) -> Result<()> {
    let items: Vec<serde_json::Value> = catalog::CATALOG
        .iter()
        .map(|e| {
            json!({
                "code": e.code,
                "title": e.title,
            })
        })
        .collect();
    output::list(ctx, &items)
}
