use clap::Parser;
use clap::Subcommand;
use color_eyre::Result;

use crate::commands;
use crate::errors::CliError;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Source(commands::source::SourceCommand),
}

impl Cli {
    pub fn execute(self) -> Result<(), CliError> {
        match self.command {
            Commands::Source(cmd) => cmd.execute(),
        }
    }
}
