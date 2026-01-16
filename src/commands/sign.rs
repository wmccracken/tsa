use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

use crate::api::TailscaleClient;
use crate::models::Device;
use crate::utils::{
    confirm, print_devices_with_lock_info, print_error, print_info, print_success, print_warning,
    resolve_device_patterns,
};

fn sign_device(node_key: &str, tailnet_lock_key: &str) -> Result<()> {
    let output = Command::new("tailscale")
        .args(["lock", "sign", node_key, tailnet_lock_key])
        .output()
        .context("Failed to execute 'tailscale lock sign' command. Is tailscale installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("{}", stderr.trim());
    }

    Ok(())
}

pub async fn run_sign(
    client: &TailscaleClient,
    device_patterns: Option<&[String]>,
    skip_confirm: bool,
) -> Result<()> {
    let all_devices = client.list_devices().await?;

    let devices_with_keys: Vec<_> = all_devices
        .iter()
        .filter(|d| d.has_lock_keys())
        .cloned()
        .collect();

    let to_sign: Vec<Device> = match device_patterns {
        Some(patterns) => {
            let matched = resolve_device_patterns(patterns, &devices_with_keys);
            if matched.is_empty() {
                print_warning("No devices matched the given pattern(s).");
                if !devices_with_keys.is_empty() {
                    println!();
                    print_info("Devices with tailnet lock keys:");
                    println!();
                    print_devices_with_lock_info(&devices_with_keys);
                }
                return Ok(());
            }
            matched
        }
        None => {
            if devices_with_keys.is_empty() {
                print_warning("No devices with tailnet lock keys found.");
                println!();
                println!("{}", "This could mean:".dimmed());
                println!("  {} Tailnet lock is not enabled on your tailnet", "•".dimmed());
                println!("  {} The API didn't return lock key fields", "•".dimmed());
                return Ok(());
            }
            print_info("Devices with tailnet lock keys:");
            println!();
            print_devices_with_lock_info(&devices_with_keys);
            println!();
            println!(
                "{} {}",
                "To sign specific devices, run:".dimmed(),
                "tsa sign -d <device-pattern>".cyan()
            );
            return Ok(());
        }
    };

    print_info(&format!(
        "The following {} device(s) will be signed:",
        to_sign.len().to_string().cyan()
    ));
    println!();
    print_devices_with_lock_info(&to_sign);
    println!();

    let missing_keys: Vec<_> = to_sign
        .iter()
        .filter(|d| d.node_key.is_empty() || d.tailnet_lock_key.is_empty())
        .collect();

    if !missing_keys.is_empty() {
        print_warning("The following devices are missing required keys and will be skipped:");
        for d in &missing_keys {
            println!(
                "  {} {} - nodeKey: {}, tailnetLockKey: {}",
                "•".yellow(),
                d.hostname.cyan(),
                if d.node_key.is_empty() {
                    "missing".red().to_string()
                } else {
                    "present".green().to_string()
                },
                if d.tailnet_lock_key.is_empty() {
                    "missing".red().to_string()
                } else {
                    "present".green().to_string()
                }
            );
        }
        println!();
    }

    let signable: Vec<_> = to_sign
        .iter()
        .filter(|d| !d.node_key.is_empty() && !d.tailnet_lock_key.is_empty())
        .collect();

    if signable.is_empty() {
        print_warning("No devices can be signed (missing keys).");
        return Ok(());
    }

    if !skip_confirm && !confirm("Proceed with signing?") {
        print_warning("Aborted.");
        return Ok(());
    }

    println!();
    for device in signable {
        match sign_device(&device.node_key, &device.tailnet_lock_key) {
            Ok(()) => print_success(&device.hostname, "signed"),
            Err(e) => print_error(&device.hostname, &format!("{}", e)),
        }
    }

    println!("\n{}", "Done!".green().bold());
    Ok(())
}
