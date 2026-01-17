use anyhow::Result;
use colored::Colorize;

use crate::api::TailscaleClient;
use crate::utils::{
    confirm, format_tags, normalize_tag, print_devices_table, print_error, print_info,
    print_success, print_warning, resolve_device_patterns,
};

pub async fn run_update_tags(
    client: &TailscaleClient,
    device_patterns: &[String],
    tags: &[String],
    skip_confirm: bool,
) -> Result<()> {
    let normalized_tags: Vec<String> = tags.iter().map(|t| normalize_tag(t)).collect();
    let all_devices = client.list_devices().await?;
    let matched_devices = resolve_device_patterns(device_patterns, &all_devices);

    if matched_devices.is_empty() {
        print_warning("No devices matched the given pattern(s).");
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
    device_patterns: &[String],
    tags: &[String],
    skip_confirm: bool,
) -> Result<()> {
    let normalized_tags: Vec<String> = tags.iter().map(|t| normalize_tag(t)).collect();
    let all_devices = client.list_devices().await?;
    let matched_devices = resolve_device_patterns(device_patterns, &all_devices);

    if matched_devices.is_empty() {
        print_warning("No devices matched the given pattern(s).");
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
    device_patterns: &[String],
    tags: &[String],
    skip_confirm: bool,
) -> Result<()> {
    let normalized_tags: Vec<String> = tags.iter().map(|t| normalize_tag(t)).collect();
    let all_devices = client.list_devices().await?;
    let matched_devices = resolve_device_patterns(device_patterns, &all_devices);

    if matched_devices.is_empty() {
        print_warning("No devices matched the given pattern(s).");
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
