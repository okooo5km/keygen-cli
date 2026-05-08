use clap::{Args, ValueEnum};

use crate::{cli::Context, error::Result};

#[derive(Debug, Args)]
pub struct SchemaArgs {
    /// Schema output format.
    #[arg(long, value_enum, default_value_t = SchemaFormat::Json)]
    pub format: SchemaFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SchemaFormat {
    Json,
    Yaml,
}

pub async fn dump(_ctx: &Context, _args: SchemaArgs) -> Result<()> {
    Err(crate::Error::user(
        "schema export not yet implemented (step 13 placeholder)",
    ))
}
