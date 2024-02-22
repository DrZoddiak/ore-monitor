mod commands;
mod models;
mod ore;

use anyhow::Result;
use clap::Parser;
use commands::{Cli, OreCommand};
use ore::OreAuth;

async fn handle_cli(cli: Cli) -> Result<()> {
    //Authorize the ore client
    let ore_client = OreAuth::default().auth().await?;

    //parse command
    let cmd: &dyn OreCommand = match &cli {
        Cli::Search(cmd) => cmd,
        Cli::Plugin(cmd) => cmd,
    };

    cmd.handle(ore_client).await
}

#[tokio::main]
async fn main() {
    let e = handle_cli(Cli::parse()).await;
    if let Err(err) = e {
        println!("Error has occured : {err}")
    };
}
