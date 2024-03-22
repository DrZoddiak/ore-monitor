mod commands;
mod ore;
mod sponge_schemas;

use anyhow::Result;
use clap::Parser;
use commands::core_command::Cli;
use ore::ore_auth::OreAuth;

/// Entrypoint for the application
#[tokio::main]
async fn main() -> Result<()> {
    // Authorize the ore client
    let ore_client = OreAuth::default().auth().await?;

    Ok(Cli::parse().cmd_value().handle(ore_client, None).await?)
}
