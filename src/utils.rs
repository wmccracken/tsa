use std::collections::HashSet;
use std::io::{self, Write};

use colored::Colorize;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, ContentArrangement, Table};
use console::Term;
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

pub fn print_devices_table(devices: &[Device], columns: Option<&[String]>) {
    println!("{}", build_devices_table(devices, columns));
}

pub fn build_devices_table(devices: &[Device], columns: Option<&[String]>) -> String {
    // Default columns if none specified
    let default_columns = vec![
        "hostname".to_string(),
        "owner".to_string(),
        "os".to_string(),
        "status".to_string(),
        "locked".to_string(),
        "tags".to_string(),
    ];
    let selected_columns = columns.unwrap_or(&default_columns);

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);

    // Build header based on selected columns
    let mut headers = Vec::new();
    for col in selected_columns {
        let header = match col.to_lowercase().as_str() {
            "hostname" => "Hostname",
            "name" => "Name",
            "owner" => "Owner",
            "os" => "OS",
            "status" => "Status",
            "locked" => "Locked",
            "tags" => "Tags",
            _ => continue, // Skip unknown columns
        };
        headers.push(Cell::new(header).fg(Color::Cyan));
    }
    table.set_header(headers);

    // Build rows
    for device in devices {
        let mut row = Vec::new();

        for col in selected_columns {
            let cell = match col.to_lowercase().as_str() {
                "hostname" => Cell::new(&device.hostname),
                "name" => Cell::new(&device.name),
                "owner" => Cell::new(device.owner()),
                "os" => Cell::new(&device.os),
                "status" => {
                    if device.is_online() {
                        Cell::new("● online").fg(Color::Green)
                    } else {
                        Cell::new("○ offline").fg(Color::DarkGrey)
                    }
                }
                "locked" => {
                    if device.is_locked_out() {
                        Cell::new("✗ locked").fg(Color::Red)
                    } else {
                        Cell::new("-").fg(Color::DarkGrey)
                    }
                }
                "tags" => {
                    let tags_str = if device.tags.is_empty() {
                        "-".to_string()
                    } else {
                        device.tags.join(", ")
                    };
                    Cell::new(tags_str)
                }
                _ => continue, // Skip unknown columns
            };
            row.push(cell);
        }

        table.add_row(row);
    }

    table.to_string()
}

pub fn print_devices_with_lock_info(devices: &[Device]) {
    println!("{}", build_devices_with_lock_info(devices));
}

pub fn build_devices_with_lock_info(devices: &[Device]) -> String {
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

    table.to_string()
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

/// Display text with interactive paging (like less/more)
/// If the content fits on one screen, displays it directly
/// Otherwise, shows it page by page with prompts
pub fn display_with_pager(content: &str, no_paging: bool) -> io::Result<()> {
    let lines: Vec<&str> = content.lines().collect();

    if no_paging {
        println!("{}", content);
        return Ok(());
    }

    let term = Term::stdout();
    let (_, rows) = term.size();

    // VSCode and some terminals report incorrect sizes
    // Use a conservative page size that works better in practice
    // Most terminals show 20-50 lines comfortably
    let lines_per_page = if rows > 100 {
        // If terminal reports huge size (like 183), it's probably wrong
        // Use a reasonable default instead
        40
    } else if rows > 10 {
        (rows as usize).saturating_sub(5)
    } else {
        rows as usize
    };

    // If content fits on one screen, just print it directly
    if lines.len() <= lines_per_page {
        println!("{}", content);
        return Ok(());
    }

    // Switch to alternate screen (like less does)
    print!("\x1b[?1049h");
    io::stdout().flush()?;

    // If we get here, content requires paging
    // Display content page by page
    let mut current_line = 0;

    let pager_result = (|| -> io::Result<()> {
        loop {
            // Clear screen and move to top
            print!("\x1b[2J\x1b[H");
            io::stdout().flush()?;

            // Calculate end line for this page
            let end_line = (current_line + lines_per_page).min(lines.len());

            // Display the current page
            for line in &lines[current_line..end_line] {
                println!("{}", line);
            }

            // Check if we're at the end
            let at_end = end_line >= lines.len();

            // Show prompt at the bottom
            if at_end {
                print!(
                    "{}",
                    format!("-- End -- (press q to quit)")
                        .dimmed()
                );
            } else {
                let remaining = lines.len() - end_line;
                print!(
                    "{}",
                    format!(
                        "-- More -- ({} more lines, press Space/Enter for next page, q to quit)",
                        remaining
                    )
                    .dimmed()
                );
            }
            io::stdout().flush()?;

            // Wait for user input
            let key = term.read_key()?;

            // Clear the prompt line
            term.clear_last_lines(1)?;

            match key {
                console::Key::Char('q') | console::Key::Char('Q') | console::Key::Escape => {
                    return Ok(());
                }
                console::Key::Char(' ') | console::Key::Enter => {
                    if at_end {
                        return Ok(());
                    }
                    current_line = end_line;
                }
                _ => {
                    // Ignore other keys, continue showing same page
                    continue;
                }
            }
        }
    })();

    // Always restore the normal screen before returning
    print!("\x1b[?1049l");
    io::stdout().flush()?;

    pager_result
}
