use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Args)]
pub struct GlobalArgs {
    /// Profile name from the config file.
    #[arg(long, global = true, env = "KEYGEN_PROFILE")]
    pub profile: Option<String>,

    /// Override the API host (e.g. https://api.keygen.sh).
    #[arg(long, global = true, env = "KEYGEN_HOST")]
    pub host: Option<String>,

    /// Override the account id or slug (Official / multiplayer self-hosted).
    #[arg(long, global = true, env = "KEYGEN_ACCOUNT")]
    pub account: Option<String>,

    /// Inject a token (skips keyring lookup).
    #[arg(long, global = true, env = "KEYGEN_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    /// EE: override the active environment id.
    #[arg(long, global = true, env = "KEYGEN_ENV")]
    pub env: Option<String>,

    /// Output format. Defaults: human=table, ai/non-tty=json.
    #[arg(long, global = true, value_enum)]
    pub output: Option<OutputFormat>,

    /// Disable ANSI colors.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Quiet mode — only print key results (id/key/etc.).
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Verbose logging (-v info, -vv debug, -vvv trace).
    #[arg(long, short = 'v', global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Force AI mode (json + hint + no spinner).
    #[arg(long, global = true, conflicts_with = "human")]
    pub ai: bool,

    /// Force human mode (table + colors + spinner).
    #[arg(long, global = true)]
    pub human: bool,

    /// Print the request that would be sent without executing it.
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Idempotency key for write operations.
    #[arg(long, global = true)]
    pub idempotency_key: Option<String>,

    /// Request timeout in seconds.
    #[arg(long, global = true, default_value_t = 30)]
    pub timeout: u64,

    /// Number of retries for idempotent requests.
    #[arg(long, global = true, default_value_t = 2)]
    pub retry: u8,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Tsv,
    Ndjson,
}
