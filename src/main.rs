mod api;
mod cli;
mod commands;
mod models;
mod utils;

use anyhow::Result;
use clap::Parser;

use api::TailscaleClient;
use cli::{Cli, Commands, ContactCommands, DeviceCommands, UserCommands};
use commands::{
    run_add_tags, run_approve_user, run_delete, run_delete_user, run_info, run_list,
    run_list_contacts, run_list_users, run_remove_tags, run_rename, run_restore_user, run_sign,
    run_suspend_user, run_update_tags,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = TailscaleClient::new(cli.api_key, cli.tailnet);
    let skip_confirm = cli.yes;

    match cli.command {
        Commands::Devices(device_cmd) => match device_cmd {
            DeviceCommands::List {
                devices,
                locked,
                columns,
                no_paging,
                json,
            } => {
                run_list(
                    &client,
                    devices.as_deref(),
                    locked,
                    columns,
                    no_paging,
                    json,
                )
                .await?;
            }
            DeviceCommands::UpdateTags { devices, tags } => {
                run_update_tags(&client, devices.as_deref(), &tags, skip_confirm).await?;
            }
            DeviceCommands::AddTags { devices, tags } => {
                run_add_tags(&client, devices.as_deref(), &tags, skip_confirm).await?;
            }
            DeviceCommands::RemoveTags { devices, tags } => {
                run_remove_tags(&client, devices.as_deref(), &tags, skip_confirm).await?;
            }
            DeviceCommands::Sign { devices } => {
                run_sign(&client, devices.as_deref(), skip_confirm).await?;
            }
            DeviceCommands::Delete { devices } => {
                run_delete(&client, devices.as_deref(), skip_confirm).await?;
            }
            DeviceCommands::Info { device, json } => {
                run_info(&client, &device, json).await?;
            }
            DeviceCommands::Rename { device, new_name } => {
                run_rename(&client, &device, &new_name, skip_confirm).await?;
            }
        },
        Commands::Users(user_cmd) => match user_cmd {
            UserCommands::List { json } => {
                run_list_users(&client, json).await?;
            }
            UserCommands::Approve { user } => {
                run_approve_user(&client, &user, skip_confirm).await?;
            }
            UserCommands::Suspend { user } => {
                run_suspend_user(&client, &user, skip_confirm).await?;
            }
            UserCommands::Restore { user } => {
                run_restore_user(&client, &user, skip_confirm).await?;
            }
            UserCommands::Delete { user } => {
                run_delete_user(&client, &user, skip_confirm).await?;
            }
        },
        Commands::Contacts(contact_cmd) => match contact_cmd {
            ContactCommands::List { json } => {
                run_list_contacts(&client, json).await?;
            }
        },
    }

    Ok(())
}
