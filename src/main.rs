use anyhow::Result;

mod commands;
mod ore;
mod sponge_schemas;

use clap::Parser;
use commands::{Cli, OreCommand};
use ore::OreAuth;

/// Main method to dispatch commands
async fn handle_cli(cli: Cli) -> Result<()> {
    // Authorize the ore client
    let ore_client = OreAuth::default().auth().await?;

    // parse command
    match &cli {
        Cli::Search(cmd) => cmd.handle(ore_client, None).await,
        Cli::Plugin(cmd) => cmd.handle(ore_client, None).await,
        Cli::Install(cmd) => cmd.handle(ore_client, None).await,
        Cli::Check(cmd) => cmd.handle(ore_client, None).await,
    }
}

/// Entrypoint for the application
#[tokio::main]
async fn main() {
    if let Err(err) = handle_cli(Cli::parse()).await {
        println!("Error has occured : {}", err)
    }
}
