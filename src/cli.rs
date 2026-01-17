use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tsa")]
#[command(version)]
#[command(about = "Tailscale API CLI tool for managing device tags and signing locked nodes", long_about = None)]
pub struct Cli {
    /// Tailscale API key (can also be set via TAILSCALE_API_KEY env var)
    #[arg(short, long, env = "TAILSCALE_API_KEY")]
    pub api_key: String,

    /// Tailnet name (e.g., your-domain.com or user@gmail.com, or use "-" for default)
    #[arg(short = 'n', long, env = "TAILSCALE_TAILNET", default_value = "-")]
    pub tailnet: String,

    /// Skip confirmation prompts
    #[arg(short, long, default_value = "false")]
    pub yes: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all devices in the tailnet
    List {
        /// Show only locked-out devices
        #[arg(long)]
        locked: bool,

        /// Columns to display (comma-separated: hostname,name,owner,os,status,locked,tags)
        #[arg(long, value_delimiter = ',')]
        columns: Option<Vec<String>>,

        /// Disable paging (show all results at once)
        #[arg(long)]
        no_paging: bool,

        /// Output as JSON instead of a table
        #[arg(long)]
        json: bool,
    },

    /// Update tags on specified devices (replaces existing tags)
    UpdateTags {
        /// Device patterns to match (comma-separated or multiple flags)
        #[arg(short, long, value_delimiter = ',', required = true)]
        devices: Vec<String>,

        /// Tags to set on the devices (e.g., tag:server,tag:prod)
        #[arg(short, long, value_delimiter = ',', required = true)]
        tags: Vec<String>,
    },

    /// Add tags to specified devices (keeps existing tags)
    AddTags {
        /// Device patterns to match (comma-separated or multiple flags)
        #[arg(short, long, value_delimiter = ',', required = true)]
        devices: Vec<String>,

        /// Tags to add to the devices (e.g., tag:server,tag:prod)
        #[arg(short, long, value_delimiter = ',', required = true)]
        tags: Vec<String>,
    },

    /// Remove tags from specified devices
    RemoveTags {
        /// Device patterns to match (comma-separated or multiple flags)
        #[arg(short, long, value_delimiter = ',', required = true)]
        devices: Vec<String>,

        /// Tags to remove from the devices (e.g., tag:server,tag:prod)
        #[arg(short, long, value_delimiter = ',', required = true)]
        tags: Vec<String>,
    },

    /// Sign locked-out devices using tailnet lock (requires this machine to be a signing node)
    Sign {
        /// Device patterns to match (comma-separated or multiple flags). If not specified, all devices with lock keys will be shown.
        #[arg(short, long, value_delimiter = ',')]
        devices: Option<Vec<String>>,
    },
}
