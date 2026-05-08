use clap::{Args, CommandFactory, ValueEnum};

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

pub async fn dump(_ctx: &Context, args: SchemaArgs) -> Result<()> {
    let cmd = crate::cli::Cli::command();
    let schema = super::generate::dump(&cmd);
    match args.format {
        SchemaFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&schema)?);
        }
        SchemaFormat::Yaml => {
            print!(
                "{}",
                serde_yaml::to_string(&schema).map_err(|e| crate::Error::Serde(e.to_string()))?
            );
        }
    }
    Ok(())
}
