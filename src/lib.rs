//! `keygen-cli` — AI-friendly CLI for keygen.sh.
//!
//! Authored by okooo5km(十里).

pub mod api;
pub mod auth;
pub mod capability;
pub mod cli;
pub mod config;
pub mod error;
pub mod exit;
pub mod explain;
pub mod output;
pub mod render;
pub mod resources;
pub mod schema;
pub mod tui;
pub mod view;

pub use error::{Error, Result};

use cli::{Cli, Command};

/// Top-level dispatch. Each command produces a typed payload that the output
/// layer renders in either AI (JSON) or human (table/colored) form.
pub async fn run(cli: Cli) -> Result<()> {
    let ctx = cli::Context::from_globals(&cli.globals)?;

    match cli.command {
        Command::Login(args) => auth::commands::login(&ctx, args).await,
        Command::Logout(args) => auth::commands::logout(&ctx, args).await,
        Command::Whoami => auth::commands::whoami(&ctx).await,
        Command::Config(cmd) => config::commands::dispatch(&ctx, cmd).await,
        Command::Profile(cmd) => config::profile_commands::dispatch(&ctx, cmd).await,
        Command::Env(cmd) => capability::env_commands::dispatch(&ctx, cmd).await,
        Command::Doctor(args) => capability::doctor::run(&ctx, args).await,
        Command::Explain(cmd) => explain::commands::dispatch(&ctx, cmd).await,
        Command::Schema(args) => schema::commands::dump(&ctx, args).await,
        Command::Completion(args) => cli::completion::generate(&args),
        Command::Tui => tui::launch(&ctx).await,
        Command::Resource(res) => resources::dispatch(&ctx, res).await,
    }
}
