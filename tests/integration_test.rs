use tsa::models::{Device, User};
use tsa::utils::{
    find_devices_by_pattern, normalize_tag, resolve_device_patterns,
};

fn create_test_device(id: &str, name: &str, hostname: &str, tags: Vec<String>) -> Device {
    Device {
        id: id.to_string(),
        name: name.to_string(),
        hostname: hostname.to_string(),
        tags,
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
fn test_end_to_end_device_filtering() {
    let devices = vec![
        create_test_device("1", "web-server-1", "web1.example.com", vec!["tag:web".to_string(), "tag:prod".to_string()]),
        create_test_device("2", "web-server-2", "web2.example.com", vec!["tag:web".to_string(), "tag:staging".to_string()]),
        create_test_device("3", "db-server-1", "db1.example.com", vec!["tag:database".to_string(), "tag:prod".to_string()]),
        create_test_device("4", "api-server-1", "api1.example.com", vec!["tag:api".to_string(), "tag:prod".to_string()]),
    ];

    // Test finding all web servers
    let web_servers = find_devices_by_pattern("web", &devices);
    assert_eq!(web_servers.len(), 2);
    assert!(web_servers.iter().all(|d| d.name.contains("web")));

    // Test finding specific server by ID
    let specific = find_devices_by_pattern("1", &devices);
    assert_eq!(specific.len(), 1);
    assert_eq!(specific[0].id, "1");

    // Test finding by hostname
    let by_hostname = find_devices_by_pattern("db1.example.com", &devices);
    assert_eq!(by_hostname.len(), 1);
    assert_eq!(by_hostname[0].hostname, "db1.example.com");
}

#[test]
fn test_tag_normalization_workflow() {
    let raw_tags = vec!["server", "tag:prod", "web"];
    let normalized: Vec<String> = raw_tags.iter()
        .map(|t| normalize_tag(t))
        .collect();

    assert_eq!(normalized, vec![
        "tag:server",
        "tag:prod",
        "tag:web"
    ]);
}

#[test]
fn test_multiple_pattern_resolution() {
    let devices = vec![
        create_test_device("1", "server1", "host1", vec![]),
        create_test_device("2", "server2", "host2", vec![]),
        create_test_device("3", "server3", "host3", vec![]),
        create_test_device("4", "database1", "dbhost", vec![]),
    ];

    let patterns = vec![
        "server1".to_string(),
        "host2".to_string(),
        "database".to_string(),
    ];

    let result = resolve_device_patterns(&patterns, &devices);

    // Should match server1, server2 (via host2), and database1
    assert_eq!(result.len(), 3);
    assert!(result.iter().any(|d| d.id == "1"));
    assert!(result.iter().any(|d| d.id == "2"));
    assert!(result.iter().any(|d| d.id == "4"));
}

#[test]
fn test_device_status_methods() {
    let mut device = create_test_device("1", "test", "host", vec![]);

    // Test online detection (empty last_seen)
    assert!(device.is_online());

    // Test locked out detection
    assert!(!device.is_locked_out());

    device.tailnet_lock_key = "tlpub:abc".to_string();
    device.tailnet_lock_error = "signature failed".to_string();
    assert!(device.is_locked_out());

    // Test lock keys
    assert!(!device.has_lock_keys()); // no node_key yet

    device.node_key = "nodekey:xyz".to_string();
    assert!(device.has_lock_keys());
}

#[test]
fn test_user_deserialization_edge_cases() {
    // Test minimal user
    let json = r#"{
        "id": "u1",
        "loginName": "test@example.com",
        "role": "member"
    }"#;
    let user: User = serde_json::from_str(json).unwrap();
    assert_eq!(user.id, "u1");
    assert_eq!(user.login_name, "test@example.com");
    assert_eq!(user.display_name, "");
    assert!(!user.currently_active);
    assert_eq!(user.device_count, 0);

    // Test full user with currentlyConnected
    let json_full = r#"{
        "id": "u2",
        "loginName": "active@example.com",
        "displayName": "Active User",
        "profilePicUrl": "https://example.com/pic.jpg",
        "role": "admin",
        "status": "active",
        "deviceCount": 5,
        "lastSeen": "2024-01-01T00:00:00Z",
        "currentlyConnected": true
    }"#;
    let user_full: User = serde_json::from_str(json_full).unwrap();
    assert_eq!(user_full.id, "u2");
    assert_eq!(user_full.login_name, "active@example.com");
    assert_eq!(user_full.display_name, "Active User");
    assert_eq!(user_full.role, "admin");
    assert_eq!(user_full.status, "active");
    assert_eq!(user_full.device_count, 5);
    assert!(user_full.currently_active);
}

#[test]
fn test_device_owner_display() {
    let mut device = create_test_device("1", "test", "host", vec![]);

    // With user
    assert_eq!(device.owner(), "test@example.com");

    // Without user
    device.user = String::new();
    assert_eq!(device.owner(), "-");
}
