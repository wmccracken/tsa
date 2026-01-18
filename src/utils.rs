use std::collections::HashSet;
use std::io::{self, Write};

use colored::Colorize;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, ContentArrangement, Table};
use console::Term;
use dialoguer::{Confirm, Input};

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
            "id" => "ID",
            "hostname" => "Hostname",
            "name" => "Name",
            "owner" | "user" => "Owner",
            "os" => "OS",
            "status" => "Status",
            "locked" => "Locked",
            "tags" => "Tags",
            "last_seen" | "lastseen" => "Last Seen",
            "node_key" | "nodekey" => "Node Key",
            "tailnet_lock_key" | "tailnetlockkey" | "lock_key" | "lockkey" => "Tailnet Lock Key",
            "tailnet_lock_error" | "tailnetlockerror" | "lock_error" | "lockerror" => {
                "Tailnet Lock Error"
            }
            "blocks_incoming_connections" | "blocksincomingconnections" | "blocks_incoming" => {
                "Blocks Incoming"
            }
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
                "id" => Cell::new(&device.id),
                "hostname" => Cell::new(&device.hostname),
                "name" => Cell::new(&device.name),
                "owner" | "user" => Cell::new(device.owner()),
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
                "last_seen" | "lastseen" => {
                    if device.last_seen.is_empty() {
                        Cell::new("-").fg(Color::DarkGrey)
                    } else {
                        Cell::new(&device.last_seen)
                    }
                }
                "node_key" | "nodekey" => {
                    if device.node_key.is_empty() {
                        Cell::new("-").fg(Color::DarkGrey)
                    } else {
                        // Truncate long keys for readability
                        let display = if device.node_key.len() > 20 {
                            format!("{}...", &device.node_key[..20])
                        } else {
                            device.node_key.clone()
                        };
                        Cell::new(display)
                    }
                }
                "tailnet_lock_key" | "tailnetlockkey" | "lock_key" | "lockkey" => {
                    if device.tailnet_lock_key.is_empty() {
                        Cell::new("-").fg(Color::DarkGrey)
                    } else {
                        // Truncate long keys for readability
                        let display = if device.tailnet_lock_key.len() > 20 {
                            format!("{}...", &device.tailnet_lock_key[..20])
                        } else {
                            device.tailnet_lock_key.clone()
                        };
                        Cell::new(display)
                    }
                }
                "tailnet_lock_error" | "tailnetlockerror" | "lock_error" | "lockerror" => {
                    if device.tailnet_lock_error.is_empty() {
                        Cell::new("-").fg(Color::DarkGrey)
                    } else {
                        Cell::new(&device.tailnet_lock_error).fg(Color::Red)
                    }
                }
                "blocks_incoming_connections" | "blocksincomingconnections" | "blocks_incoming" => {
                    if device.blocks_incoming_connections {
                        Cell::new("✓ yes").fg(Color::Yellow)
                    } else {
                        Cell::new("-").fg(Color::DarkGrey)
                    }
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

/// Display a numbered list of devices and prompt user to select by numbers.
/// Returns the selected devices.
///
/// Selection format (like yay):
/// - Single numbers: "1 3 5"
/// - Ranges: "1-5"
/// - Combination: "1 3-7 10"
/// - All: "all" or "*"
/// - None: empty input or "none"
pub fn select_devices_interactive(devices: &[Device]) -> io::Result<Vec<Device>> {
    if devices.is_empty() {
        return Ok(Vec::new());
    }

    // Display numbered list
    println!();
    println!("{}", "Select devices:".bold());
    println!();

    for (i, device) in devices.iter().enumerate() {
        let status = if device.is_online() {
            "● online".green()
        } else {
            "○ offline".dimmed()
        };

        let locked = if device.is_locked_out() {
            " [LOCKED]".red()
        } else {
            "".normal()
        };

        println!(
            "  {} {} {} {}{}",
            format!("[{}]", i + 1).cyan().bold(),
            device.hostname.cyan(),
            format!("({})", device.name).dimmed(),
            status,
            locked
        );
    }

    println!();
    println!("{}", "Enter selection:".dimmed());
    println!("  {} Single numbers: 1 3 5", "•".dimmed());
    println!("  {} Ranges: 1-5", "•".dimmed());
    println!("  {} Combination: 1 3-7 10", "•".dimmed());
    println!("  {} All devices: all or *", "•".dimmed());
    println!("  {} Cancel: none or empty", "•".dimmed());
    println!();

    let input: String = Input::new()
        .with_prompt("Selection")
        .allow_empty(true)
        .interact_text()?;

    let selection = input.trim();

    // Handle empty input or "none"
    if selection.is_empty() || selection.eq_ignore_ascii_case("none") {
        return Ok(Vec::new());
    }

    // Handle "all" or "*"
    if selection.eq_ignore_ascii_case("all") || selection == "*" {
        return Ok(devices.to_vec());
    }

    // Parse numbers and ranges
    let mut selected_indices = HashSet::new();

    for part in selection.split_whitespace() {
        if part.contains('-') {
            // Handle range like "1-5"
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() != 2 {
                print_warning(&format!("Invalid range format: {}", part));
                continue;
            }

            let start = match range_parts[0].parse::<usize>() {
                Ok(n) if n > 0 && n <= devices.len() => n,
                _ => {
                    print_warning(&format!("Invalid range start: {}", range_parts[0]));
                    continue;
                }
            };

            let end = match range_parts[1].parse::<usize>() {
                Ok(n) if n > 0 && n <= devices.len() => n,
                _ => {
                    print_warning(&format!("Invalid range end: {}", range_parts[1]));
                    continue;
                }
            };

            if start > end {
                print_warning(&format!("Invalid range: start ({}) > end ({})", start, end));
                continue;
            }

            for i in start..=end {
                selected_indices.insert(i - 1);
            }
        } else {
            // Handle single number
            match part.parse::<usize>() {
                Ok(n) if n > 0 && n <= devices.len() => {
                    selected_indices.insert(n - 1);
                }
                _ => {
                    print_warning(&format!("Invalid selection: {}", part));
                }
            }
        }
    }

    let selected: Vec<Device> = selected_indices
        .into_iter()
        .map(|i| devices[i].clone())
        .collect();

    Ok(selected)
}

/// Resolve devices either from patterns or via interactive selection.
/// If patterns list is empty or contains only "-", prompts for interactive selection.
pub fn resolve_or_select_devices(
    patterns: &[String],
    all_devices: &[Device],
) -> io::Result<Vec<Device>> {
    // Check if we should use interactive mode
    let use_interactive = patterns.is_empty()
        || (patterns.len() == 1 && (patterns[0] == "-" || patterns[0].is_empty()));

    if use_interactive {
        print_info(&format!(
            "Found {} device(s):",
            all_devices.len().to_string().cyan()
        ));
        select_devices_interactive(all_devices)
    } else {
        Ok(resolve_device_patterns(patterns, all_devices))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_device(id: &str, name: &str, hostname: &str) -> Device {
        Device {
            id: id.to_string(),
            name: name.to_string(),
            hostname: hostname.to_string(),
            tags: vec![],
            os: "linux".to_string(),
            user: "test@example.com".to_string(),
            last_seen: String::new(),
            node_key: String::new(),
            tailnet_lock_error: String::new(),
            tailnet_lock_key: String::new(),
            blocks_incoming_connections: false,
        }
    }

    #[test]
    fn test_find_devices_by_pattern_exact_id() {
        let devices = vec![
            create_test_device("123", "device1", "host1"),
            create_test_device("456", "device2", "host2"),
        ];
        let result = find_devices_by_pattern("123", &devices);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "123");
    }

    #[test]
    fn test_find_devices_by_pattern_exact_name() {
        let devices = vec![
            create_test_device("123", "device1", "host1"),
            create_test_device("456", "device2", "host2"),
        ];
        let result = find_devices_by_pattern("device1", &devices);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "device1");
    }

    #[test]
    fn test_find_devices_by_pattern_exact_hostname() {
        let devices = vec![
            create_test_device("123", "device1", "host1"),
            create_test_device("456", "device2", "host2"),
        ];
        let result = find_devices_by_pattern("host1", &devices);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].hostname, "host1");
    }

    #[test]
    fn test_find_devices_by_pattern_case_insensitive() {
        let devices = vec![
            create_test_device("123", "Device1", "Host1"),
            create_test_device("456", "device2", "host2"),
        ];
        let result = find_devices_by_pattern("device1", &devices);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Device1");
    }

    #[test]
    fn test_find_devices_by_pattern_partial_match() {
        let devices = vec![
            create_test_device("123", "web-server-1", "host1"),
            create_test_device("456", "web-server-2", "host2"),
            create_test_device("789", "db-server-1", "host3"),
        ];
        let result = find_devices_by_pattern("web", &devices);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|d| d.name == "web-server-1"));
        assert!(result.iter().any(|d| d.name == "web-server-2"));
    }

    #[test]
    fn test_find_devices_by_pattern_no_match() {
        let devices = vec![
            create_test_device("123", "device1", "host1"),
            create_test_device("456", "device2", "host2"),
        ];
        let result = find_devices_by_pattern("nonexistent", &devices);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_resolve_device_patterns_single_pattern() {
        let devices = vec![
            create_test_device("123", "device1", "host1"),
            create_test_device("456", "device2", "host2"),
        ];
        let patterns = vec!["device1".to_string()];
        let result = resolve_device_patterns(&patterns, &devices);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "device1");
    }

    #[test]
    fn test_resolve_device_patterns_multiple_patterns() {
        let devices = vec![
            create_test_device("123", "device1", "host1"),
            create_test_device("456", "device2", "host2"),
            create_test_device("789", "device3", "host3"),
        ];
        let patterns = vec!["device1".to_string(), "device2".to_string()];
        let result = resolve_device_patterns(&patterns, &devices);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_resolve_device_patterns_deduplication() {
        let devices = vec![
            create_test_device("123", "web-server", "host1"),
            create_test_device("456", "device2", "host2"),
        ];
        // Both patterns match the same device
        let patterns = vec!["web-server".to_string(), "host1".to_string()];
        let result = resolve_device_patterns(&patterns, &devices);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "123");
    }

    #[test]
    fn test_normalize_tag_with_prefix() {
        assert_eq!(normalize_tag("tag:server"), "tag:server");
    }

    #[test]
    fn test_normalize_tag_without_prefix() {
        assert_eq!(normalize_tag("server"), "tag:server");
    }

    #[test]
    fn test_format_tags_empty() {
        let tags: Vec<String> = vec![];
        let result = format_tags(&tags);
        // The result will contain ANSI color codes, so just check it's not empty
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_tags_single() {
        let tags = vec!["tag:server".to_string()];
        let result = format_tags(&tags);
        assert!(result.contains("tag:server"));
    }

    #[test]
    fn test_find_devices_by_pattern_hyphen_prefix_exact() {
        let devices = vec![
            create_test_device("1", "-gpu-1", "host1"),
            create_test_device("2", "-gpu-2", "host2"),
            create_test_device("3", "gpu-3", "host3"),
        ];
        let result = find_devices_by_pattern("-gpu-1", &devices);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "-gpu-1");
    }

    #[test]
    fn test_find_devices_by_pattern_hyphen_prefix_partial() {
        let devices = vec![
            create_test_device("1", "-gpu-1", "host1"),
            create_test_device("2", "-gpu-2", "host2"),
            create_test_device("3", "gpu-3", "host3"),
        ];
        let result = find_devices_by_pattern("-gpu", &devices);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|d| d.name == "-gpu-1"));
        assert!(result.iter().any(|d| d.name == "-gpu-2"));
    }

    #[test]
    fn test_find_devices_by_pattern_multiple_hyphens() {
        let devices = vec![
            create_test_device("1", "--special-device", "host1"),
            create_test_device("2", "-regular-device", "host2"),
            create_test_device("3", "no-prefix-device", "host3"),
        ];
        let result = find_devices_by_pattern("--special-device", &devices);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "--special-device");
    }

    #[test]
    fn test_find_devices_by_pattern_hyphen_hostname() {
        let devices = vec![
            create_test_device("1", "device1", "-special-host"),
            create_test_device("2", "device2", "normal-host"),
        ];
        let result = find_devices_by_pattern("-special-host", &devices);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].hostname, "-special-host");
    }

    #[test]
    fn test_resolve_device_patterns_with_hyphens() {
        let devices = vec![
            create_test_device("1", "-gpu-1", "host1"),
            create_test_device("2", "-gpu-2", "host2"),
            create_test_device("3", "server-01", "host3"),
        ];
        let patterns = vec!["-gpu-1".to_string(), "server-01".to_string()];
        let result = resolve_device_patterns(&patterns, &devices);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|d| d.name == "-gpu-1"));
        assert!(result.iter().any(|d| d.name == "server-01"));
    }

    #[test]
    fn test_format_tags_multiple() {
        let tags = vec!["tag:server".to_string(), "tag:prod".to_string()];
        let result = format_tags(&tags);
        assert!(result.contains("tag:server"));
        assert!(result.contains("tag:prod"));
    }

    #[test]
    fn test_build_devices_table_default_columns() {
        let devices = vec![
            create_test_device("123", "device1", "host1"),
        ];
        let table = build_devices_table(&devices, None);
        assert!(table.contains("host1"));
        assert!(table.contains("Hostname"));
    }

    #[test]
    fn test_build_devices_table_custom_columns() {
        let devices = vec![
            create_test_device("123", "device1", "host1"),
        ];
        let columns = vec!["id".to_string(), "name".to_string()];
        let table = build_devices_table(&devices, Some(&columns));
        assert!(table.contains("123"));
        assert!(table.contains("device1"));
        assert!(table.contains("ID"));
        assert!(table.contains("Name"));
    }

    #[test]
    fn test_build_devices_with_lock_info() {
        let mut device = create_test_device("123", "device1", "host1");
        device.node_key = "nodekey:abc123".to_string();
        device.tailnet_lock_key = "tlpub:def456".to_string();

        let devices = vec![device];
        let table = build_devices_with_lock_info(&devices);

        assert!(table.contains("host1"));
        assert!(table.contains("device1"));
        assert!(table.contains("nodekey:abc123"));
        assert!(table.contains("tlpub:def456"));
    }
}
