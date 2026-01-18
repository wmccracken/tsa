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

#[derive(Debug, Serialize)]
pub struct RenameDeviceRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UsersResponse {
    pub users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub login_name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub profile_pic_url: String,
    pub role: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub device_count: u32,
    #[serde(default)]
    pub last_seen: String,
    #[serde(default, rename = "currentlyConnected")]
    pub currently_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactsResponse {
    pub account: Contact,
    pub support: Contact,
    pub security: Contact,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub email: String,
    #[serde(default)]
    pub fallback_email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_is_online_empty_last_seen() {
        let device = Device {
            id: "test".to_string(),
            name: "test".to_string(),
            hostname: "test".to_string(),
            tags: vec![],
            os: "linux".to_string(),
            user: "test@example.com".to_string(),
            last_seen: String::new(),
            node_key: String::new(),
            tailnet_lock_error: String::new(),
            tailnet_lock_key: String::new(),
            blocks_incoming_connections: false,
        };
        assert!(device.is_online());
    }

    #[test]
    fn test_device_is_online_recent_timestamp() {
        let now = chrono::Utc::now();
        let recent = now - chrono::Duration::minutes(2);
        let device = Device {
            id: "test".to_string(),
            name: "test".to_string(),
            hostname: "test".to_string(),
            tags: vec![],
            os: "linux".to_string(),
            user: "test@example.com".to_string(),
            last_seen: recent.to_rfc3339(),
            node_key: String::new(),
            tailnet_lock_error: String::new(),
            tailnet_lock_key: String::new(),
            blocks_incoming_connections: false,
        };
        assert!(device.is_online());
    }

    #[test]
    fn test_device_is_offline_old_timestamp() {
        let now = chrono::Utc::now();
        let old = now - chrono::Duration::minutes(10);
        let device = Device {
            id: "test".to_string(),
            name: "test".to_string(),
            hostname: "test".to_string(),
            tags: vec![],
            os: "linux".to_string(),
            user: "test@example.com".to_string(),
            last_seen: old.to_rfc3339(),
            node_key: String::new(),
            tailnet_lock_error: String::new(),
            tailnet_lock_key: String::new(),
            blocks_incoming_connections: false,
        };
        assert!(!device.is_online());
    }

    #[test]
    fn test_device_is_locked_out() {
        let device = Device {
            id: "test".to_string(),
            name: "test".to_string(),
            hostname: "test".to_string(),
            tags: vec![],
            os: "linux".to_string(),
            user: "test@example.com".to_string(),
            last_seen: String::new(),
            node_key: "nodekey:abc123".to_string(),
            tailnet_lock_error: "signature verification failed".to_string(),
            tailnet_lock_key: "tlpub:def456".to_string(),
            blocks_incoming_connections: false,
        };
        assert!(device.is_locked_out());
    }

    #[test]
    fn test_device_not_locked_out() {
        let device = Device {
            id: "test".to_string(),
            name: "test".to_string(),
            hostname: "test".to_string(),
            tags: vec![],
            os: "linux".to_string(),
            user: "test@example.com".to_string(),
            last_seen: String::new(),
            node_key: "nodekey:abc123".to_string(),
            tailnet_lock_error: String::new(),
            tailnet_lock_key: "tlpub:def456".to_string(),
            blocks_incoming_connections: false,
        };
        assert!(!device.is_locked_out());
    }

    #[test]
    fn test_device_has_lock_keys() {
        let device = Device {
            id: "test".to_string(),
            name: "test".to_string(),
            hostname: "test".to_string(),
            tags: vec![],
            os: "linux".to_string(),
            user: "test@example.com".to_string(),
            last_seen: String::new(),
            node_key: "nodekey:abc123".to_string(),
            tailnet_lock_error: String::new(),
            tailnet_lock_key: "tlpub:def456".to_string(),
            blocks_incoming_connections: false,
        };
        assert!(device.has_lock_keys());
    }

    #[test]
    fn test_device_owner_with_user() {
        let device = Device {
            id: "test".to_string(),
            name: "test".to_string(),
            hostname: "test".to_string(),
            tags: vec![],
            os: "linux".to_string(),
            user: "test@example.com".to_string(),
            last_seen: String::new(),
            node_key: String::new(),
            tailnet_lock_error: String::new(),
            tailnet_lock_key: String::new(),
            blocks_incoming_connections: false,
        };
        assert_eq!(device.owner(), "test@example.com");
    }

    #[test]
    fn test_device_owner_without_user() {
        let device = Device {
            id: "test".to_string(),
            name: "test".to_string(),
            hostname: "test".to_string(),
            tags: vec![],
            os: "linux".to_string(),
            user: String::new(),
            last_seen: String::new(),
            node_key: String::new(),
            tailnet_lock_error: String::new(),
            tailnet_lock_key: String::new(),
            blocks_incoming_connections: false,
        };
        assert_eq!(device.owner(), "-");
    }

    #[test]
    fn test_device_deserialization() {
        let json = r#"{
            "id": "123",
            "name": "test-device",
            "hostname": "test-host",
            "os": "linux",
            "tags": ["tag:server", "tag:prod"]
        }"#;
        let device: Device = serde_json::from_str(json).unwrap();
        assert_eq!(device.id, "123");
        assert_eq!(device.name, "test-device");
        assert_eq!(device.hostname, "test-host");
        assert_eq!(device.os, "linux");
        assert_eq!(device.tags, vec!["tag:server", "tag:prod"]);
    }

    #[test]
    fn test_user_deserialization_with_currently_connected() {
        let json = r#"{
            "id": "u123",
            "loginName": "test@example.com",
            "displayName": "Test User",
            "role": "member",
            "status": "active",
            "deviceCount": 2,
            "currentlyConnected": true
        }"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, "u123");
        assert_eq!(user.login_name, "test@example.com");
        assert_eq!(user.display_name, "Test User");
        assert_eq!(user.role, "member");
        assert_eq!(user.status, "active");
        assert_eq!(user.device_count, 2);
        assert!(user.currently_active);
    }

    #[test]
    fn test_user_deserialization_without_currently_connected() {
        let json = r#"{
            "id": "u456",
            "loginName": "inactive@example.com",
            "role": "member"
        }"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, "u456");
        assert_eq!(user.login_name, "inactive@example.com");
        assert!(!user.currently_active);
    }

    #[test]
    fn test_update_tags_request_serialization() {
        let request = UpdateTagsRequest {
            tags: vec!["tag:server".to_string(), "tag:prod".to_string()],
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("tag:server"));
        assert!(json.contains("tag:prod"));
    }
}
