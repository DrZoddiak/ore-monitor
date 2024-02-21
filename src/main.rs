mod commands;
mod ore;
mod paginated_project_result;

use anyhow::{Ok, Result};
use clap::Parser;
use commands::{Cli, SubCommands};
use ore::OreAuth;

async fn handle_cli(cli: Cli) -> Result<()> {
    //Authorize the ore client
    let ore_client = OreAuth::default().auth().await?;

    //parse command
    match &cli {
        Cli::Projects { subcommand } => match subcommand {
            SubCommands::Search(cmd) => Ok(cmd.handle(&ore_client).await?),
            SubCommands::Plugin(cmd) => Ok(cmd.handle(&ore_client).await?),
        },
    }
}

#[tokio::main]
async fn main() {
    handle_cli(Cli::parse()).await.expect("No error")
}
