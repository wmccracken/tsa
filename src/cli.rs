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
    /// Manage devices in the tailnet
    #[command(subcommand)]
    Devices(DeviceCommands),

    /// Manage users in the tailnet
    #[command(subcommand)]
    Users(UserCommands),

    /// Manage tailnet contacts
    #[command(subcommand)]
    Contacts(ContactCommands),
}

#[derive(Subcommand)]
pub enum DeviceCommands {
    /// List all devices in the tailnet
    List {
        /// Device patterns to match (comma-separated or multiple flags). If not specified, all devices are shown.
        #[arg(short, long, value_delimiter = ',', allow_hyphen_values = true)]
        devices: Option<Vec<String>>,

        /// Show only locked-out devices
        #[arg(long)]
        locked: bool,

        /// Columns to display (comma-separated). Available: id, hostname, name, owner, os, status, locked, tags, last_seen, node_key, tailnet_lock_key, tailnet_lock_error, blocks_incoming_connections
        #[arg(long, value_delimiter = ',')]
        columns: Option<Vec<String>>,

        /// Disable paging (show all results at once)
        #[arg(long)]
        no_paging: bool,

        /// Output as JSON instead of a table
        #[arg(long)]
        json: bool,
    },

    /// Update tags on specified devices (replaces existing tags; use -y before 'devices' to skip confirmation)
    UpdateTags {
        /// Device patterns to match (comma-separated or multiple flags). If not specified, interactive selection will be prompted.
        #[arg(short, long, value_delimiter = ',', allow_hyphen_values = true)]
        devices: Option<Vec<String>>,

        /// Tags to set on the devices (e.g., tag:server,tag:prod)
        #[arg(short, long, value_delimiter = ',', required = true)]
        tags: Vec<String>,
    },

    /// Add tags to specified devices (keeps existing tags; use -y before 'devices' to skip confirmation)
    AddTags {
        /// Device patterns to match (comma-separated or multiple flags). If not specified, interactive selection will be prompted.
        #[arg(short, long, value_delimiter = ',', allow_hyphen_values = true)]
        devices: Option<Vec<String>>,

        /// Tags to add to the devices (e.g., tag:server,tag:prod)
        #[arg(short, long, value_delimiter = ',', required = true)]
        tags: Vec<String>,
    },

    /// Remove tags from specified devices (use -y before 'devices' to skip confirmation)
    RemoveTags {
        /// Device patterns to match (comma-separated or multiple flags). If not specified, interactive selection will be prompted.
        #[arg(short, long, value_delimiter = ',', allow_hyphen_values = true)]
        devices: Option<Vec<String>>,

        /// Tags to remove from the devices (e.g., tag:server,tag:prod)
        #[arg(short, long, value_delimiter = ',', required = true)]
        tags: Vec<String>,
    },

    /// Sign locked-out devices using tailnet lock (requires this machine to be a signing node; use -y before 'devices' to skip confirmation)
    Sign {
        /// Device patterns to match (comma-separated or multiple flags). If not specified, interactive selection will be prompted.
        #[arg(short, long, value_delimiter = ',', allow_hyphen_values = true)]
        devices: Option<Vec<String>>,
    },

    /// Delete devices from the tailnet (use -y before 'devices' to skip confirmation)
    Delete {
        /// Device patterns to match (comma-separated or multiple flags). If not specified, interactive selection will be prompted.
        #[arg(short, long, value_delimiter = ',', allow_hyphen_values = true)]
        devices: Option<Vec<String>>,
    },

    /// Display detailed information about a device
    Info {
        /// Device pattern to match (device ID, name, or hostname)
        #[arg(required = true)]
        device: String,

        /// Output as JSON instead of formatted text
        #[arg(long)]
        json: bool,
    },

    /// Rename a device (use -y before 'devices' to skip confirmation)
    Rename {
        /// Device pattern to match (device ID, name, or hostname)
        #[arg(required = true)]
        device: String,

        /// New name for the device
        #[arg(required = true)]
        new_name: String,
    },
}

#[derive(Subcommand)]
pub enum UserCommands {
    /// List users in the tailnet
    List {
        /// Output as JSON instead of a table
        #[arg(long)]
        json: bool,
    },

    /// Approve a user (use -y before 'users' to skip confirmation)
    Approve {
        /// User pattern to match (login name or user ID)
        #[arg(short, long, required = true)]
        user: String,
    },

    /// Suspend a user (use -y before 'users' to skip confirmation)
    Suspend {
        /// User pattern to match (login name or user ID)
        #[arg(short, long, required = true)]
        user: String,
    },

    /// Restore a suspended user (use -y before 'users' to skip confirmation)
    Restore {
        /// User pattern to match (login name or user ID)
        #[arg(short, long, required = true)]
        user: String,
    },

    /// Delete a user from the tailnet (use -y before 'users' to skip confirmation)
    Delete {
        /// User pattern to match (login name or user ID)
        #[arg(short, long, required = true)]
        user: String,
    },
}

#[derive(Subcommand)]
pub enum ContactCommands {
    /// List tailnet contacts
    List {
        /// Output as JSON instead of a table
        #[arg(long)]
        json: bool,
    },
}
