use anyhow::Result;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, ContentArrangement, Table};

use crate::api::TailscaleClient;
use crate::utils::print_info;

pub async fn run_list_contacts(client: &TailscaleClient, json: bool) -> Result<()> {
    let contacts = client.get_contacts().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&contacts)?);
        return Ok(());
    }

    print_info("Tailnet contacts:");
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Type").fg(Color::Cyan),
            Cell::new("Email").fg(Color::Cyan),
            Cell::new("Fallback Email").fg(Color::Cyan),
        ]);

    table.add_row(vec![
        Cell::new("Account"),
        Cell::new(&contacts.account.email),
        Cell::new(if contacts.account.fallback_email.is_empty() {
            "-"
        } else {
            &contacts.account.fallback_email
        })
        .fg(Color::DarkGrey),
    ]);

    table.add_row(vec![
        Cell::new("Support"),
        Cell::new(&contacts.support.email),
        Cell::new(if contacts.support.fallback_email.is_empty() {
            "-"
        } else {
            &contacts.support.fallback_email
        })
        .fg(Color::DarkGrey),
    ]);

    table.add_row(vec![
        Cell::new("Security"),
        Cell::new(&contacts.security.email),
        Cell::new(if contacts.security.fallback_email.is_empty() {
            "-"
        } else {
            &contacts.security.fallback_email
        })
        .fg(Color::DarkGrey),
    ]);

    println!("{}", table);

    Ok(())
}
