mod commands;
mod ore;
mod sponge_schemas;

use anyhow::Result;
use clap::Parser;
use commands::{Cli, OreCommand};
use ore::OreAuth;

/// Test
async fn handle_cli(cli: Cli) -> Result<()> {
    //Authorize the ore client
    let ore_client = OreAuth::default().auth().await?;

    //parse command
    let cmd: &dyn OreCommand = match &cli {
        Cli::Search(cmd) => cmd,
        Cli::Plugin(cmd) => cmd,
        Cli::Install(cmd) => cmd,
    };

    cmd.handle(ore_client, None).await
}

#[tokio::main]
async fn main() {
    if let Err(err) = handle_cli(Cli::parse()).await {
        println!("Error has occured : {}", err)
    }
}
