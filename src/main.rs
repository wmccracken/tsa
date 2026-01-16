mod api;
mod cli;
mod commands;
mod models;
mod utils;

use anyhow::Result;
use clap::Parser;

use api::TailscaleClient;
use cli::{Cli, Commands};
use commands::{run_add_tags, run_list, run_remove_tags, run_sign, run_update_tags};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = TailscaleClient::new(cli.api_key, cli.tailnet);
    let skip_confirm = cli.yes;

    match cli.command {
        Commands::List { locked } => {
            run_list(&client, locked).await?;
        }
        Commands::UpdateTags { devices, tags } => {
            run_update_tags(&client, &devices, &tags, skip_confirm).await?;
        }
        Commands::AddTags { devices, tags } => {
            run_add_tags(&client, &devices, &tags, skip_confirm).await?;
        }
        Commands::RemoveTags { devices, tags } => {
            run_remove_tags(&client, &devices, &tags, skip_confirm).await?;
        }
        Commands::Sign { devices } => {
            run_sign(&client, devices.as_deref(), skip_confirm).await?;
        }
    }

    Ok(())
}
