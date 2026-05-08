use std::io;

use clap::{Args, CommandFactory, ValueEnum};
use clap_complete::{generate as clap_generate, Shell};

use crate::error::Result;

#[derive(Debug, Args)]
pub struct CompletionArgs {
    /// Target shell.
    #[arg(value_enum)]
    pub shell: ShellChoice,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ShellChoice {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
}

impl From<ShellChoice> for Shell {
    fn from(value: ShellChoice) -> Self {
        match value {
            ShellChoice::Bash => Shell::Bash,
            ShellChoice::Zsh => Shell::Zsh,
            ShellChoice::Fish => Shell::Fish,
            ShellChoice::Powershell => Shell::PowerShell,
            ShellChoice::Elvish => Shell::Elvish,
        }
    }
}

pub fn generate(args: &CompletionArgs) -> Result<()> {
    let mut cmd = super::Cli::command();
    let bin = cmd.get_name().to_string();
    clap_generate(Shell::from(args.shell), &mut cmd, bin, &mut io::stdout());
    Ok(())
}
