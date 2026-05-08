//! Shared building blocks for resource subcommands.
//!
//! Every resource follows the same CRUD shape:
//!
//! ```text
//! keygen <resource> list      [--filter k=v ...] [--limit N] [--page N] [--include rel] [--sort field]
//! keygen <resource> get       <id> [--include rel]
//! keygen <resource> create    [--from-file <json>|-] [--<attr> ...] [--metadata k=v ...]
//! keygen <resource> update    <id> [--from-file <json>|-] [--<attr> ...]
//! keygen <resource> delete    <id> [--yes]
//! ```
//!
//! Resource-specific actions (validate / suspend / publish / ...) are added
//! to the per-resource `Cmd` enum on top of these.

use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    /// Filter rows. May be passed multiple times: `--filter status=ACTIVE`.
    #[arg(long, value_name = "K=V")]
    pub filter: Vec<String>,

    /// Page number (1-based).
    #[arg(long, default_value_t = 1)]
    pub page: u64,

    /// Page size.
    #[arg(long, default_value_t = 50)]
    pub limit: u64,

    /// Sort field (prefix with `-` for descending).
    #[arg(long)]
    pub sort: Option<String>,

    /// Include related resources, comma separated.
    #[arg(long, value_delimiter = ',')]
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct GetArgs {
    /// Resource id.
    pub id: String,

    /// Include related resources, comma separated.
    #[arg(long, value_delimiter = ',')]
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct DeleteArgs {
    /// Resource id.
    pub id: String,

    /// Skip the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CreateArgs {
    /// Read the full JSON body from a file (or `-` for stdin).
    #[arg(long, value_name = "PATH|-")]
    pub from_file: Option<String>,

    /// Metadata entries `k=v`. May be repeated.
    #[arg(long, value_name = "K=V")]
    pub metadata: Vec<String>,

    /// JSONPath-style attribute overrides, e.g. `--set attrs.maxMachines=5`.
    #[arg(long, value_name = "PATH=VALUE")]
    pub set: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct UpdateArgs {
    /// Resource id.
    pub id: String,

    /// Read the full JSON body from a file (or `-` for stdin).
    #[arg(long, value_name = "PATH|-")]
    pub from_file: Option<String>,

    /// Metadata entries `k=v`. May be repeated.
    #[arg(long, value_name = "K=V")]
    pub metadata: Vec<String>,

    /// JSONPath-style attribute overrides.
    #[arg(long, value_name = "PATH=VALUE")]
    pub set: Vec<String>,
}
