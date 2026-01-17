use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct DevicesResponse {
    pub devices: Vec<Device>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub name: String,
    pub hostname: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub os: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub last_seen: String,
    #[serde(default)]
    pub node_key: String,
    #[serde(default)]
    pub tailnet_lock_error: String,
    #[serde(default)]
    pub tailnet_lock_key: String,
    #[serde(default)]
    pub blocks_incoming_connections: bool,
}

impl Device {
    /// Check if device is online based on lastSeen timestamp.
    /// The API returns lastSeen only when disconnected, so empty/recent = online.
    pub fn is_online(&self) -> bool {
        if self.last_seen.is_empty() {
            // No lastSeen means currently connected
            return true;
        }

        // Parse the timestamp and check if it's recent (within 5 minutes)
        if let Ok(last_seen) = chrono::DateTime::parse_from_rfc3339(&self.last_seen) {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(last_seen);
            duration.num_minutes() < 5
        } else {
            false
        }
    }

    pub fn is_locked_out(&self) -> bool {
        !self.tailnet_lock_key.is_empty() && !self.tailnet_lock_error.is_empty()
    }

    pub fn has_lock_keys(&self) -> bool {
        !self.node_key.is_empty() && !self.tailnet_lock_key.is_empty()
    }

    /// Get a display-friendly owner name from the user email
    pub fn owner(&self) -> &str {
        if self.user.is_empty() {
            "-"
        } else {
            &self.user
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UpdateTagsRequest {
    pub tags: Vec<String>,
}
