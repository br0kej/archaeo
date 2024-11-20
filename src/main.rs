use clap::Parser;
use color_eyre::Result;
mod cli;
mod commands;
mod errors;

fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install()?;

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = cli::Cli::parse();

    // Execute the matched command
    cli.execute()?;

    Ok(())
}
