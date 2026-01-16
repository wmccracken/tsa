use std::collections::HashSet;

use colored::Colorize;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, ContentArrangement, Table};
use dialoguer::Confirm;

use crate::models::Device;

/// Find all devices matching the given pattern.
/// Matching priority:
/// 1. Exact match on ID
/// 2. Exact match on name
/// 3. Exact match on hostname
/// 4. Case-insensitive exact match on name
/// 5. Case-insensitive exact match on hostname
/// 6. Partial match (contains) on name or hostname
pub fn find_devices_by_pattern(pattern: &str, devices: &[Device]) -> Vec<Device> {
    // First try exact match on ID
    if let Some(device) = devices.iter().find(|d| d.id == pattern) {
        return vec![device.clone()];
    }

    // Try exact match on name
    if let Some(device) = devices.iter().find(|d| d.name == pattern) {
        return vec![device.clone()];
    }

    // Try exact match on hostname
    if let Some(device) = devices.iter().find(|d| d.hostname == pattern) {
        return vec![device.clone()];
    }

    let pattern_lower = pattern.to_lowercase();

    // Try case-insensitive exact match on name
    if let Some(device) = devices
        .iter()
        .find(|d| d.name.to_lowercase() == pattern_lower)
    {
        return vec![device.clone()];
    }

    // Try case-insensitive exact match on hostname
    if let Some(device) = devices
        .iter()
        .find(|d| d.hostname.to_lowercase() == pattern_lower)
    {
        return vec![device.clone()];
    }

    // Return all partial matches (contains) on name or hostname
    devices
        .iter()
        .filter(|d| {
            d.name.to_lowercase().contains(&pattern_lower)
                || d.hostname.to_lowercase().contains(&pattern_lower)
        })
        .cloned()
        .collect()
}

/// Resolve device patterns to actual devices, collecting all matches.
/// Returns a deduplicated list of matched devices.
pub fn resolve_device_patterns(patterns: &[String], all_devices: &[Device]) -> Vec<Device> {
    let mut matched: Vec<Device> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for pattern in patterns {
        let matches = find_devices_by_pattern(pattern, all_devices);
        for device in matches {
            if seen_ids.insert(device.id.clone()) {
                matched.push(device);
            }
        }
    }

    matched
}

pub fn normalize_tag(tag: &str) -> String {
    if tag.starts_with("tag:") {
        tag.to_string()
    } else {
        format!("tag:{}", tag)
    }
}

pub fn confirm(message: &str) -> bool {
    Confirm::new()
        .with_prompt(message)
        .default(false)
        .interact()
        .unwrap_or(false)
}

pub fn print_success(device: &str, message: &str) {
    println!(
        "  {} {} {}",
        "✓".green().bold(),
        device.cyan(),
        message.green()
    );
}

pub fn print_error(device: &str, message: &str) {
    println!(
        "  {} {} {}",
        "✗".red().bold(),
        device.cyan(),
        message.red()
    );
}

pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message.yellow());
}

pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

pub fn print_devices_table(devices: &[Device]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Hostname").fg(Color::Cyan),
            Cell::new("Name").fg(Color::Cyan),
            Cell::new("Owner").fg(Color::Cyan),
            Cell::new("OS").fg(Color::Cyan),
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Tags").fg(Color::Cyan),
        ]);

    for device in devices {
        let tags_str = if device.tags.is_empty() {
            "-".to_string()
        } else {
            device.tags.join(", ")
        };

        let online_cell = if device.is_online() {
            Cell::new("● online").fg(Color::Green)
        } else {
            Cell::new("○ offline").fg(Color::DarkGrey)
        };

        table.add_row(vec![
            Cell::new(&device.hostname),
            Cell::new(&device.name),
            Cell::new(device.owner()),
            Cell::new(&device.os),
            online_cell,
            Cell::new(tags_str),
        ]);
    }

    println!("{table}");
}

pub fn print_devices_with_lock_info(devices: &[Device]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Hostname").fg(Color::Cyan),
            Cell::new("Name").fg(Color::Cyan),
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Node Key").fg(Color::Cyan),
            Cell::new("Tailnet Lock Key").fg(Color::Cyan),
        ]);

    for device in devices {
        let online_cell = if device.is_online() {
            Cell::new("● online").fg(Color::Green)
        } else {
            Cell::new("○ offline").fg(Color::DarkGrey)
        };

        let node_key_cell = if device.node_key.is_empty() {
            Cell::new("-").fg(Color::DarkGrey)
        } else {
            Cell::new(&device.node_key)
        };

        let lock_key_cell = if device.tailnet_lock_key.is_empty() {
            Cell::new("-").fg(Color::DarkGrey)
        } else {
            Cell::new(&device.tailnet_lock_key)
        };

        table.add_row(vec![
            Cell::new(&device.hostname),
            Cell::new(&device.name),
            online_cell,
            node_key_cell,
            lock_key_cell,
        ]);
    }

    println!("{table}");
}

pub fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        "none".dimmed().to_string()
    } else {
        tags.iter()
            .map(|t| t.cyan().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
