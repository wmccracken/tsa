use anyhow::Result;
use colored::Colorize;

use crate::api::TailscaleClient;
use crate::utils::{print_devices_table, print_devices_with_lock_info, print_info, print_warning};

pub async fn run_list(client: &TailscaleClient, locked: bool) -> Result<()> {
    let devices = client.list_devices().await?;

    if locked {
        let locked_devices: Vec<_> = devices
            .iter()
            .filter(|d| d.is_locked_out())
            .cloned()
            .collect();

        if locked_devices.is_empty() {
            print_warning("No locked-out devices found.");
        } else {
            print_info(&format!(
                "Found {} locked-out device(s):",
                locked_devices.len().to_string().cyan()
            ));
            println!();
            print_devices_with_lock_info(&locked_devices);
        }
    } else {
        print_info(&format!(
            "Found {} device(s):",
            devices.len().to_string().cyan()
        ));
        println!();
        print_devices_table(&devices);
    }

    Ok(())
}
