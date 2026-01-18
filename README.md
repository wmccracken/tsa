# tsa - Tailscale CLI Tool

A command-line tool written in Rust for managing your Tailscale network.

## Notice

This application is currently under development and should be considered alpha release quality. I'm using it daily for Tailscale admin tasks but am also planning to make frequent updates.

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/tsa`.

Packages for popular distributions will be coming soon.

## Configuration

The tool requires a Tailscale API key. You can provide it via:

- Environment variable: `TAILSCALE_API_KEY`
- Command-line flag: `-a` or `--api-key`

Generate an API key at: https://login.tailscale.com/admin/settings/keys

Optionally, set your tailnet name:

- Environment variable: `TAILSCALE_TAILNET`
- Command-line flag: `-n` or `--tailnet`
- Default: `-` (uses the default tailnet for your API key)

## Global Options

| Flag | Description |
|------|-------------|
| `-a, --api-key` | Tailscale API key |
| `-n, --tailnet` | Tailnet name (default: `-`) |
| `-y, --yes` | Skip confirmation prompts |
| `-V, --version` | Display version information |

**Note:** The `-y/--yes` flag is a global option and must be placed **before** the subcommand:

```bash
# Correct usage
tsa -y devices rename myserver newserver
tsa -y devices delete -d oldserver

# Incorrect usage (will not work)
tsa devices rename -y myserver newserver
```

## Device Matching

Device commands accept patterns that match by name, hostname, or ID. The matching priority is:

1. Exact match on device ID
2. Exact match on name
3. Exact match on hostname
4. Case-insensitive exact match
5. Partial match (contains) - returns **all matching devices**

When multiple devices match a pattern, you'll be shown the list and asked to confirm before proceeding.

**Note:** If a pattern starts with a hyphen (e.g., `-gpu-1`), use the `-d` flag:

```bash
tsa devices info -d -gpu-server
```

## Interactive Selection

When you run certain commands without specifying device patterns (or omit the `-d` flag), you'll be presented with an interactive device selector. The selector accepts:

- **Single numbers**: `1 3 5` - selects devices 1, 3, and 5
- **Ranges**: `1-5` - selects devices 1 through 5
- **Combinations**: `1 3-7 10` - selects device 1, devices 3-7, and device 10
- **All devices**: `all` or `*` - selects all devices
- **Cancel**: empty input or `none` - cancels the operation

---

## Subcommand: `devices list`

List all devices in the tailnet.

```bash
tsa devices list
tsa devices list --no-paging
tsa devices list --columns hostname,status,locked
tsa devices list --locked
tsa devices list --json
```

### Options

| Flag | Description |
|------|-------------|
| `--no-paging` | Disable automatic paging |
| `--columns <cols>` | Select which columns to display |
| `--locked` | Show only locked-out devices |
| `--json` | Output as JSON |

### Paging

Output automatically pages to fit your terminal size:

- If output fits on one screen, it displays directly
- If longer, shows one page at a time:
  - Press `Space` or `Enter` for next page
  - Press `q` or `Esc` to quit

### Available Columns

- `id` - Device ID
- `hostname` - Device hostname
- `name` - Device name
- `owner` (or `user`) - Device owner email
- `os` - Operating system
- `status` - Online/offline status
- `locked` - Tailnet lock status
- `tags` - Device tags
- `last_seen` - Last seen timestamp
- `node_key` - Node key
- `tailnet_lock_key` (or `lock_key`) - Tailnet lock key
- `tailnet_lock_error` (or `lock_error`) - Tailnet lock error message
- `blocks_incoming_connections` (or `blocks_incoming`) - Whether device blocks incoming connections

### Examples

```bash
# Show only specific columns
tsa devices list --columns hostname,name,status

# Show lock-related information
tsa devices list --columns hostname,locked,tailnet_lock_error

# Pipe JSON output to jq for filtering
tsa devices list --json | jq '.[] | select(.os == "linux")'
```

---

# Device Management

## Device Information
Display detailed information about a specific device.

```bash
tsa devices info <device-pattern>
tsa devices info myserver --json
```

### Options

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

The `info` command displays:
- Device name and hostname
- Owner (user email)
- Operating system
- Online/offline status
- Tailnet lock status (if applicable)
- Tags
- Last seen timestamp
- Node keys and lock keys (if present)
- Any lock errors

---

## Device Renaming

Rename a device in your tailnet.

```bash
tsa devices rename <device-pattern> <new-name>
tsa -y devices rename myserver web-server-01
```

---

## Device Deletion

Delete devices from your tailnet.

```bash
# Interactive selection
tsa devices delete

# Delete specific devices by pattern
tsa devices delete -d myserver

# Filter by pattern, then select interactively
tsa devices delete -d server

# Skip confirmation
tsa -y devices delete -d old-server
```

### Options

| Flag | Description |
|------|-------------|
| `-d <pattern>` | Device pattern to match |

The delete command:
- Shows a warning that the action cannot be undone
- Displays the devices that will be deleted
- Requires confirmation unless `-y` flag is used

---

## Update Device Tags

Replace all tags on a device with the specified tags.

```bash
tsa devices update-tags -d <device-pattern> -t <tags>
tsa devices update-tags -t tag:server  # interactive selection
```

### Options

| Flag | Description |
|------|-------------|
| `-d <pattern>` | Device pattern to match |
| `-t <tags>` | Comma-separated list of tags |

Tags can be specified with or without the `tag:` prefix:

```bash
tsa devices update-tags -d myserver -t prod,web
# equivalent to:
tsa devices update-tags -d myserver -t tag:prod,tag:web
```

---

## Add Device Tags

Add tags to a device without removing existing tags.

```bash
tsa devices add-tags -d <device-pattern> -t <tags>
tsa devices add-tags -t tag:monitored  # interactive selection
```

### Options

| Flag | Description |
|------|-------------|
| `-d <pattern>` | Device pattern to match (optional for interactive selection) |
| `-t <tags>` | Comma-separated list of tags to add |

---

## Remove Device Tags

Remove specific tags from a device.

```bash
tsa devices remove-tags -d <device-pattern> -t <tags>
tsa -y devices remove-tags -d server -t tag:deprecated
```

### Options

| Flag | Description |
|------|-------------|
| `-d <pattern>` | Device pattern to match |
| `-t <tags>` | Comma-separated list of tags to remove |

---

## Signing Devices

Sign locked-out devices for [Tailnet Lock](https://tailscale.com/kb/1226/tailnet-lock).

This requires:
1. The machine running `tsa` must be a **signing node** (have a trusted Tailnet Lock key)
2. The `tailscale` CLI must be installed and accessible

```bash
# Interactive selection
tsa devices sign

# Sign specific devices
tsa devices sign -d <device-pattern>

# Skip confirmation
tsa -y devices sign -d server
```

### Options

| Flag | Description |
|------|-------------|
| `-d <pattern>` | Device pattern to match (optional for interactive selection) |

The `sign` command will:
1. Fetch devices from the API with their `nodeKey` and `tailnetLockKey`
2. Show the devices that will be signed
3. Ask for confirmation
4. Execute `tailscale lock sign <nodeKey> <tailnetLockKey>` for each device

---

# User Management

## Listing Users

List all users in the tailnet.

```bash
tsa users list
tsa users list --json
```

### Options

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

The user list displays:
- Login name (email)
- Display name
- Role (owner, admin, member, etc.)
- Status (active, suspended, etc.)
- Device count
- Currently active status

---

## Approving Users

Approve a pending user.

```bash
tsa users approve -u user@example.com
```

---

## Suspending Users

Suspend a user, preventing tailnet access.

```bash
tsa users suspend -u user@example.com
```

---

## Restoring Users

Restore a suspended user.

```bash
tsa users restore -u user@example.com
```

---

## Deleting Users

Delete a user from the tailnet (permanent action).

```bash
tsa users delete -u user@example.com
tsa -y users delete -u user@example.com
```

### User Matching

When specifying users with the `-u` flag, you can match by:
1. Exact user ID
2. Exact login name (email)
3. Case-insensitive login name
4. Partial match on login name or display name

If multiple users match, you'll see a table of matches and be asked to provide a more specific pattern.

---

# Contacts

## List Contacts

List the tailnet's contact information.

```bash
tsa contacts list
tsa contacts list --json
```

### Options

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

Displays account, support, and security contacts.

---

## Notes

- Tags must be defined in your [tailnet policy file (ACL)](https://login.tailscale.com/admin/acls) before they can be applied to devices.
- Updating tags on a device does not change the device's key expiry unless you re-authenticate.

## Versioning

This project follows [Semantic Versioning](https://semver.org/). To check the current version:

```bash
tsa --version
```

The version number is defined in [Cargo.toml](Cargo.toml) and automatically included in the binary.

## License

MIT
