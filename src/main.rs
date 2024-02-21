mod commands;
mod ore;
mod paginated_project_result;

use anyhow::Result;
use clap::Parser;
use commands::{Cli, OreCommand, SubCommands};
use ore::OreAuth;

async fn handle_cli(cli: Cli) -> Result<()> {
    //Authorize the ore client
    let ore_client = OreAuth::default().auth().await?;

    //parse command
    let cmd: &dyn OreCommand = match &cli {
        Cli::Projects { subcommand } => match subcommand {
            SubCommands::Search(cmd) => cmd,
            SubCommands::Plugin(cmd) => cmd,
        },
    };

    cmd.handle(&ore_client).await
}

#[tokio::main]
async fn main() {
    handle_cli(Cli::parse()).await.expect("No error")
}
