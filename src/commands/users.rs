use anyhow::Result;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, ContentArrangement, Table};

use crate::api::TailscaleClient;
use crate::models::User;
use crate::utils::{confirm, print_error, print_info, print_success, print_warning};

pub async fn run_list_users(client: &TailscaleClient, json: bool) -> Result<()> {
    let users = client.list_users().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&users)?);
        return Ok(());
    }

    print_info(&format!(
        "Found {} user(s):",
        users.len().to_string().cyan()
    ));
    println!();

    print_users_table(&users);

    Ok(())
}

pub async fn run_approve_user(
    client: &TailscaleClient,
    user_pattern: &str,
    skip_confirm: bool,
) -> Result<()> {
    let all_users = client.list_users().await?;
    let user = find_user_by_pattern(user_pattern, &all_users)?;

    print_info(&format!(
        "User to approve: {} ({})",
        user.login_name.cyan(),
        user.display_name.dimmed()
    ));
    println!();

    if !skip_confirm && !confirm("Proceed with approval?") {
        print_warning("Aborted.");
        return Ok(());
    }

    match client.approve_user(&user.id).await {
        Ok(()) => {
            println!();
            print_success(&user.login_name, "approved");
        }
        Err(e) => print_error(&user.login_name, &format!("failed: {}", e)),
    }

    Ok(())
}

pub async fn run_suspend_user(
    client: &TailscaleClient,
    user_pattern: &str,
    skip_confirm: bool,
) -> Result<()> {
    let all_users = client.list_users().await?;
    let user = find_user_by_pattern(user_pattern, &all_users)?;

    print_warning(&format!(
        "User to suspend: {} ({})",
        user.login_name.yellow(),
        user.display_name.dimmed()
    ));
    println!();
    println!(
        "{} This will prevent the user from accessing the tailnet.",
        "⚠".yellow().bold()
    );
    println!();

    if !skip_confirm && !confirm("Proceed with suspension?") {
        print_warning("Aborted.");
        return Ok(());
    }

    match client.suspend_user(&user.id).await {
        Ok(()) => {
            println!();
            print_success(&user.login_name, "suspended");
        }
        Err(e) => print_error(&user.login_name, &format!("failed: {}", e)),
    }

    Ok(())
}

pub async fn run_restore_user(
    client: &TailscaleClient,
    user_pattern: &str,
    skip_confirm: bool,
) -> Result<()> {
    let all_users = client.list_users().await?;
    let user = find_user_by_pattern(user_pattern, &all_users)?;

    print_info(&format!(
        "User to restore: {} ({})",
        user.login_name.cyan(),
        user.display_name.dimmed()
    ));
    println!();

    if !skip_confirm && !confirm("Proceed with restoration?") {
        print_warning("Aborted.");
        return Ok(());
    }

    match client.restore_user(&user.id).await {
        Ok(()) => {
            println!();
            print_success(&user.login_name, "restored");
        }
        Err(e) => print_error(&user.login_name, &format!("failed: {}", e)),
    }

    Ok(())
}

pub async fn run_delete_user(
    client: &TailscaleClient,
    user_pattern: &str,
    skip_confirm: bool,
) -> Result<()> {
    let all_users = client.list_users().await?;
    let user = find_user_by_pattern(user_pattern, &all_users)?;

    println!(
        "{} {} {} ({})",
        "✗".red().bold(),
        "User to delete:".red(),
        user.login_name.red().bold(),
        user.display_name.dimmed()
    );
    println!();
    println!(
        "{} This action {} and will remove the user from the tailnet.",
        "⚠".red().bold(),
        "CANNOT BE UNDONE".red().bold()
    );
    println!();

    if !skip_confirm && !confirm("Are you sure you want to delete this user?") {
        print_warning("Aborted.");
        return Ok(());
    }

    match client.delete_user(&user.id).await {
        Ok(()) => {
            println!();
            print_success(&user.login_name, "deleted");
        }
        Err(e) => print_error(&user.login_name, &format!("failed: {}", e)),
    }

    Ok(())
}

fn find_user_by_pattern(pattern: &str, users: &[User]) -> Result<User> {
    // Try exact match on ID
    if let Some(user) = users.iter().find(|u| u.id == pattern) {
        return Ok(user.clone());
    }

    // Try exact match on login name
    if let Some(user) = users.iter().find(|u| u.login_name == pattern) {
        return Ok(user.clone());
    }

    // Try case-insensitive match on login name
    let pattern_lower = pattern.to_lowercase();
    if let Some(user) = users
        .iter()
        .find(|u| u.login_name.to_lowercase() == pattern_lower)
    {
        return Ok(user.clone());
    }

    // Try partial match on login name or display name
    let matches: Vec<&User> = users
        .iter()
        .filter(|u| {
            u.login_name.to_lowercase().contains(&pattern_lower)
                || u.display_name.to_lowercase().contains(&pattern_lower)
        })
        .collect();

    if matches.is_empty() {
        anyhow::bail!("No user found matching pattern: {}", pattern);
    }

    if matches.len() == 1 {
        return Ok(matches[0].clone());
    }

    // Multiple matches found
    println!("{}", "Multiple users matched:".yellow());
    println!();
    let matched_users: Vec<User> = matches.into_iter().cloned().collect();
    print_users_table(&matched_users);
    println!();
    anyhow::bail!(
        "Multiple users matched pattern '{}'. Please use a more specific pattern.",
        pattern
    );
}

fn print_users_table(users: &[User]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Login Name").fg(Color::Cyan),
            Cell::new("Display Name").fg(Color::Cyan),
            Cell::new("Role").fg(Color::Cyan),
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Devices").fg(Color::Cyan),
            Cell::new("Active").fg(Color::Cyan),
        ]);

    for user in users {
        let status_cell = if user.status.is_empty() {
            Cell::new("-").fg(Color::DarkGrey)
        } else if user.status == "active" {
            Cell::new(&user.status).fg(Color::Green)
        } else if user.status == "suspended" {
            Cell::new(&user.status).fg(Color::Red)
        } else {
            Cell::new(&user.status).fg(Color::Yellow)
        };

        let active_cell = if user.currently_active {
            Cell::new("● yes").fg(Color::Green)
        } else {
            Cell::new("○ no").fg(Color::DarkGrey)
        };

        table.add_row(vec![
            Cell::new(&user.login_name),
            Cell::new(if user.display_name.is_empty() {
                "-"
            } else {
                &user.display_name
            })
            .fg(Color::DarkGrey),
            Cell::new(&user.role),
            status_cell,
            Cell::new(user.device_count.to_string()),
            active_cell,
        ]);
    }

    println!("{}", table);
}
