use anyhow::{Context, Result};
use colored::Colorize;
use comfy_table::{Cell, Color, ContentArrangement, Table, presets::NOTHING};
use std::process::Command;

use crate::api::TailscaleClient;
use crate::models::Device;
use crate::utils::{
    build_devices_table, build_devices_with_lock_info, confirm, display_with_pager, format_tags,
    normalize_tag, print_devices_table, print_devices_with_lock_info, print_error, print_info,
    print_success, print_warning, resolve_device_patterns, resolve_or_select_devices,
    select_devices_interactive,
};

pub async fn run_list(
    client: &TailscaleClient,
    device_patterns: Option<&[String]>,
    locked: bool,
    columns: Option<Vec<String>>,
    no_paging: bool,
    json: bool,
) -> Result<()> {
    let all_devices = client.list_devices().await?;

    // Filter by device patterns if provided
    let devices = if let Some(patterns) = device_patterns {
        let matched = resolve_device_patterns(patterns, &all_devices);
        if matched.is_empty() {
            print_warning("No devices matched the given pattern(s).");
            return Ok(());
        }
        matched
    } else {
        all_devices
    };

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
        let header = format!("Found {} device(s):", devices.len().to_string().cyan());
        let table = build_devices_table(&devices, columns.as_deref());

        let output = format!("{} {}\n\n{}", "ℹ".blue().bold(), header, table);

        display_with_pager(&output, no_paging)?;
    }

    Ok(())
}

pub async fn run_update_tags(
    client: &TailscaleClient,
    device_patterns: Option<&[String]>,
    tags: &[String],
    skip_confirm: bool,
) -> Result<()> {
    let normalized_tags: Vec<String> = tags.iter().map(|t| normalize_tag(t)).collect();
    let all_devices = client.list_devices().await?;

    let matched_devices = resolve_or_select_devices(device_patterns.unwrap_or(&[]), &all_devices)?;

    if matched_devices.is_empty() {
        print_warning("No devices selected.");
        return Ok(());
    }

    print_info(&format!(
        "Found {} device(s) matching pattern:",
        matched_devices.len().to_string().cyan()
    ));
    println!();
    print_devices_table(&matched_devices, None);
    println!();
    println!(
        "{} {}",
        "Tags will be set to:".bold(),
        format_tags(&normalized_tags)
    );
    println!();

    if !skip_confirm && !confirm("Proceed with updating tags?") {
        print_warning("Aborted.");
        return Ok(());
    }

    println!();
    for device in &matched_devices {
        match client
            .update_tags(&device.id, normalized_tags.clone())
            .await
        {
            Ok(()) => print_success(
                &device.hostname,
                &format!("tags set to {}", format_tags(&normalized_tags)),
            ),
            Err(e) => print_error(&device.hostname, &format!("failed: {}", e)),
        }
    }

    println!("\n{}", "Done!".green().bold());
    Ok(())
}

pub async fn run_add_tags(
    client: &TailscaleClient,
    device_patterns: Option<&[String]>,
    tags: &[String],
    skip_confirm: bool,
) -> Result<()> {
    let normalized_tags: Vec<String> = tags.iter().map(|t| normalize_tag(t)).collect();
    let all_devices = client.list_devices().await?;

    let matched_devices = resolve_or_select_devices(device_patterns.unwrap_or(&[]), &all_devices)?;

    if matched_devices.is_empty() {
        print_warning("No devices selected.");
        return Ok(());
    }

    print_info(&format!(
        "Found {} device(s) matching pattern:",
        matched_devices.len().to_string().cyan()
    ));
    println!();
    print_devices_table(&matched_devices, None);
    println!();
    println!(
        "{} {}",
        "Tags to add:".bold(),
        format_tags(&normalized_tags)
    );
    println!();

    if !skip_confirm && !confirm("Proceed with adding tags?") {
        print_warning("Aborted.");
        return Ok(());
    }

    println!();
    for device in &matched_devices {
        let mut new_tags = device.tags.clone();
        for tag in &normalized_tags {
            if !new_tags.contains(tag) {
                new_tags.push(tag.clone());
            }
        }

        match client.update_tags(&device.id, new_tags.clone()).await {
            Ok(()) => print_success(
                &device.hostname,
                &format!("tags now {}", format_tags(&new_tags)),
            ),
            Err(e) => print_error(&device.hostname, &format!("failed: {}", e)),
        }
    }

    println!("\n{}", "Done!".green().bold());
    Ok(())
}

pub async fn run_remove_tags(
    client: &TailscaleClient,
    device_patterns: Option<&[String]>,
    tags: &[String],
    skip_confirm: bool,
) -> Result<()> {
    let normalized_tags: Vec<String> = tags.iter().map(|t| normalize_tag(t)).collect();
    let all_devices = client.list_devices().await?;

    let matched_devices = resolve_or_select_devices(device_patterns.unwrap_or(&[]), &all_devices)?;

    if matched_devices.is_empty() {
        print_warning("No devices selected.");
        return Ok(());
    }

    print_info(&format!(
        "Found {} device(s) matching pattern:",
        matched_devices.len().to_string().cyan()
    ));
    println!();
    print_devices_table(&matched_devices, None);
    println!();
    println!(
        "{} {}",
        "Tags to remove:".bold(),
        format_tags(&normalized_tags)
    );
    println!();

    if !skip_confirm && !confirm("Proceed with removing tags?") {
        print_warning("Aborted.");
        return Ok(());
    }

    println!();
    for device in &matched_devices {
        let new_tags: Vec<String> = device
            .tags
            .iter()
            .filter(|t| !normalized_tags.contains(t))
            .cloned()
            .collect();

        match client.update_tags(&device.id, new_tags.clone()).await {
            Ok(()) => print_success(
                &device.hostname,
                &format!("tags now {}", format_tags(&new_tags)),
            ),
            Err(e) => print_error(&device.hostname, &format!("failed: {}", e)),
        }
    }

    println!("\n{}", "Done!".green().bold());
    Ok(())
}

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
        .filter(|d| d.is_locked_out())
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
                    print_info("Locked devices:");
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
                println!(
                    "  {} Tailnet lock is not enabled on your tailnet",
                    "•".dimmed()
                );
                println!("  {} The API didn't return lock key fields", "•".dimmed());
                return Ok(());
            }
            print_info(&format!(
                "Found {} locked device(s):",
                devices_with_keys.len().to_string().cyan()
            ));

            // Prompt for interactive selection
            let selected = select_devices_interactive(&devices_with_keys)
                .context("Failed to read device selection")?;

            if selected.is_empty() {
                print_warning("No devices selected.");
                return Ok(());
            }

            selected
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

pub async fn run_delete(
    client: &TailscaleClient,
    device_patterns: Option<&[String]>,
    skip_confirm: bool,
) -> Result<()> {
    let all_devices = client.list_devices().await?;

    // Filter devices by pattern if provided, otherwise use all devices
    let filtered_devices = if let Some(patterns) = device_patterns {
        let matched = resolve_device_patterns(patterns, &all_devices);
        if matched.is_empty() {
            print_warning("No devices matched the given pattern(s).");
            return Ok(());
        }
        matched
    } else {
        all_devices
    };

    // Show interactive selection from filtered devices
    print_info(&format!(
        "Found {} device(s):",
        filtered_devices.len().to_string().cyan()
    ));

    let to_delete =
        select_devices_interactive(&filtered_devices).context("Failed to read device selection")?;

    if to_delete.is_empty() {
        print_warning("No devices selected.");
        return Ok(());
    }

    println!(
        "{} {} {}",
        "✗".red().bold(),
        "Devices to delete:".red(),
        to_delete.len().to_string().red().bold()
    );
    println!();
    print_devices_table(&to_delete, None);
    println!();
    println!(
        "{} This action {} and will remove the device(s) from the tailnet.",
        "⚠".red().bold(),
        "CANNOT BE UNDONE".red().bold()
    );
    println!();

    if !skip_confirm && !confirm("Are you sure you want to delete these devices?") {
        print_warning("Aborted.");
        return Ok(());
    }

    println!();
    for device in &to_delete {
        match client.delete_device(&device.id).await {
            Ok(()) => print_success(&device.hostname, "deleted"),
            Err(e) => print_error(&device.hostname, &format!("failed: {}", e)),
        }
    }

    println!("\n{}", "Done!".green().bold());
    Ok(())
}

pub async fn run_info(client: &TailscaleClient, device_pattern: &str, json: bool) -> Result<()> {
    let all_devices = client.list_devices().await?;

    // Find the device by pattern
    let matched = resolve_device_patterns(&[device_pattern.to_string()], &all_devices);

    if matched.is_empty() {
        print_warning(&format!(
            "No device found matching pattern: {}",
            device_pattern
        ));
        return Ok(());
    }

    if matched.len() > 1 {
        print_warning(&format!(
            "Multiple devices matched pattern '{}'. Please be more specific.",
            device_pattern
        ));
        println!();
        print_devices_table(&matched, None);
        return Ok(());
    }

    let device = &matched[0];

    // Get full device details
    let device_details = client.get_device(&device.id).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&device_details)?);
        return Ok(());
    }

    // Display formatted device information
    println!();
    println!(
        "{} {}",
        "Device Information".bold().cyan(),
        format!("({})", device_details.id).dimmed()
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.add_row(vec![
        Cell::new("Name:").fg(Color::Cyan),
        Cell::new(&device_details.name),
    ]);

    table.add_row(vec![
        Cell::new("Hostname:").fg(Color::Cyan),
        Cell::new(&device_details.hostname),
    ]);

    table.add_row(vec![
        Cell::new("Owner:").fg(Color::Cyan),
        Cell::new(device_details.owner()),
    ]);

    table.add_row(vec![
        Cell::new("OS:").fg(Color::Cyan),
        Cell::new(&device_details.os),
    ]);

    let status_value = if device_details.is_online() {
        "● online".green().to_string()
    } else {
        "○ offline".to_string()
    };
    table.add_row(vec![
        Cell::new("Status:").fg(Color::Cyan),
        Cell::new(status_value),
    ]);

    if device_details.is_locked_out() {
        table.add_row(vec![
            Cell::new("Locked:").fg(Color::Cyan),
            Cell::new("✗ yes".red()),
        ]);
    }

    if !device_details.tags.is_empty() {
        table.add_row(vec![
            Cell::new("Tags:").fg(Color::Cyan),
            Cell::new(format_tags(&device_details.tags)),
        ]);
    }

    if !device_details.last_seen.is_empty() {
        table.add_row(vec![
            Cell::new("Last Seen:").fg(Color::Cyan),
            Cell::new(&device_details.last_seen),
        ]);
    }

    if device_details.blocks_incoming_connections {
        table.add_row(vec![
            Cell::new("Blocks Incoming:").fg(Color::Cyan),
            Cell::new("yes").fg(Color::Yellow),
        ]);
    }

    if !device_details.node_key.is_empty() {
        table.add_row(vec![
            Cell::new("Node Key:").fg(Color::Cyan),
            Cell::new(&device_details.node_key).fg(Color::DarkGrey),
        ]);
    }

    if !device_details.tailnet_lock_key.is_empty() {
        table.add_row(vec![
            Cell::new("Tailnet Lock Key:").fg(Color::Cyan),
            Cell::new(&device_details.tailnet_lock_key).fg(Color::DarkGrey),
        ]);
    }

    if !device_details.tailnet_lock_error.is_empty() {
        table.add_row(vec![
            Cell::new("Lock Error:").fg(Color::Cyan),
            Cell::new(&device_details.tailnet_lock_error).fg(Color::Red),
        ]);
    }

    println!("{}", table);
    println!();

    Ok(())
}

pub async fn run_rename(
    client: &TailscaleClient,
    device_pattern: &str,
    new_name: &str,
    skip_confirm: bool,
) -> Result<()> {
    let all_devices = client.list_devices().await?;

    // Find the device by pattern
    let matched = resolve_device_patterns(&[device_pattern.to_string()], &all_devices);

    if matched.is_empty() {
        print_warning(&format!(
            "No device found matching pattern: {}",
            device_pattern
        ));
        return Ok(());
    }

    if matched.len() > 1 {
        print_warning(&format!(
            "Multiple devices matched pattern '{}'. Please be more specific.",
            device_pattern
        ));
        println!();
        print_devices_table(&matched, None);
        return Ok(());
    }

    let device = &matched[0];

    print_info(&format!(
        "Renaming device {} → {}",
        device.name.cyan(),
        new_name.green()
    ));
    println!();
    println!("  {} {}", "ID:".dimmed(), device.id.dimmed());
    println!("  {} {}", "Hostname:".dimmed(), device.hostname.dimmed());
    println!();

    if !skip_confirm && !confirm("Proceed with rename?") {
        print_warning("Aborted.");
        return Ok(());
    }

    match client.rename_device(&device.id, new_name.to_string()).await {
        Ok(()) => {
            println!();
            print_success(
                &device.hostname,
                &format!("renamed to {}", new_name.green()),
            );
        }
        Err(e) => print_error(&device.hostname, &format!("failed: {}", e)),
    }

    Ok(())
}
