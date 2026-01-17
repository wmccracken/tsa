use anyhow::Result;
use colored::Colorize;

use crate::api::TailscaleClient;
use crate::utils::{
    build_devices_table, build_devices_with_lock_info, display_with_pager, print_warning,
};

pub async fn run_list(
    client: &TailscaleClient,
    locked: bool,
    columns: Option<Vec<String>>,
    no_paging: bool,
    json: bool,
) -> Result<()> {
    let devices = client.list_devices().await?;

    // If JSON output is requested, print JSON and return
    if json {
        if locked {
            let locked_devices: Vec<_> = devices
                .iter()
                .filter(|d| d.is_locked_out())
                .cloned()
                .collect();
            println!("{}", serde_json::to_string_pretty(&locked_devices)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&devices)?);
        }
        return Ok(());
    }

    // Otherwise, display as table with paging
    if locked {
        let locked_devices: Vec<_> = devices
            .iter()
            .filter(|d| d.is_locked_out())
            .cloned()
            .collect();

        if locked_devices.is_empty() {
            print_warning("No locked-out devices found.");
        } else {
            let header = format!(
                "Found {} locked-out device(s):",
                locked_devices.len().to_string().cyan()
            );
            let table = build_devices_with_lock_info(&locked_devices);

            let output = format!("{} {}\n\n{}", "ℹ".blue().bold(), header, table);

            display_with_pager(&output, no_paging)?;
        }
    } else {
        let header = format!(
            "Found {} device(s):",
            devices.len().to_string().cyan()
        );
        let table = build_devices_table(&devices, columns.as_deref());

        let output = format!("{} {}\n\n{}", "ℹ".blue().bold(), header, table);

        display_with_pager(&output, no_paging)?;
    }

    Ok(())
}
