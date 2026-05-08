pub mod completion;
pub mod context;
pub mod globals;
pub mod resources;

pub use context::Context;
pub use globals::GlobalArgs;

use clap::{Parser, Subcommand};

use crate::{auth, capability, config, explain, schema};

/// `keygen` — AI-friendly CLI for keygen.sh.
#[derive(Debug, Parser)]
#[command(
    name = "keygen",
    bin_name = "keygen",
    version,
    about = "AI-friendly CLI for keygen.sh — manage products, policies, licenses, machines, releases.",
    long_about = None,
    propagate_version = true,
    arg_required_else_help = true,
)]
pub struct Cli {
    #[command(flatten)]
    pub globals: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Interactive login: pick deployment, host, account, then mint a token.
    Login(auth::commands::LoginArgs),
    /// Remove stored credentials for a profile.
    Logout(auth::commands::LogoutArgs),
    /// Show the current identity and detected capabilities.
    Whoami,
    /// Inspect / mutate persisted configuration.
    #[command(subcommand)]
    Config(config::commands::ConfigCmd),
    /// Manage profiles (named host + account + output combos).
    #[command(subcommand)]
    Profile(config::profile_commands::ProfileCmd),
    /// EE-only: switch the active environment.
    #[command(subcommand)]
    Env(capability::env_commands::EnvCmd),
    /// Probe the configured host: connectivity, token validity, capabilities.
    Doctor(crate::capability::doctor::DoctorArgs),
    /// Explain an API error code (cause + fix).
    #[command(subcommand)]
    Explain(explain::commands::ExplainCmd),
    /// Emit the JSON schema describing every command and flag.
    Schema(schema::commands::SchemaArgs),
    /// Generate shell completion scripts.
    Completion(completion::CompletionArgs),
    /// Launch the full-screen TUI dashboard.
    Tui,
    /// Resource + action commands (license, machine, product, ...).
    #[command(flatten)]
    Resource(resources::ResourceCommand),
}
