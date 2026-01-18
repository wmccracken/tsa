# tsa - Tailscale CLI Tool

A command-line tool written in `Rust` for managing device tags and signing locked-out nodes on your Tailscale network.

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

## Usage

```bash
# Check version
tsa --version
# or
tsa -V

# List all devices (with automatic paging)
tsa devices list

# List all devices without paging
tsa devices list --no-paging

# List only specific columns
tsa devices list --columns hostname,status,locked

# List locked-out devices (tailnet lock)
tsa devices list --locked

# Update tags (replaces existing tags)
tsa devices update-tags -d <device-pattern> -t <tags>

# Add tags with interactive device selection
tsa devices add-tags -t <tags>

# Remove tags
tsa devices remove-tags -d <device-pattern> -t <tags>

# Sign locked-out devices (tailnet lock)
tsa devices sign -d <device-pattern>

# Sign with interactive device selection
tsa devices sign

# List users in the tailnet
tsa users list

# List tailnet contacts
tsa contacts list

# Approve a user
tsa users approve -u user@example.com

# Suspend a user
tsa users suspend -u user@example.com

# Restore a suspended user
tsa users restore -u user@example.com

# Delete a user
tsa users delete -u user@example.com

# Display detailed information about a device
tsa devices info <device-pattern>

# Display device info as JSON
tsa devices info <device-pattern> --json

# Rename a device
tsa devices rename <device-pattern> <new-name>

# Rename without confirmation
tsa -y devices rename myserver web-server-01

# Delete devices (with interactive selection)
tsa devices delete

# Delete specific devices by pattern
tsa devices delete -d <device-pattern>
```

### Options

| Flag | Description |
|------|-------------|
| `-a, --api-key` | Tailscale API key |
| `-n, --tailnet` | Tailnet name (default: `-`) |
| `-y, --yes` | Skip confirmation prompts (global flag, place before subcommand) |

**Note about `-y` flag:** The `-y/--yes` flag is a global option and must be placed **before** the subcommand:
```bash
# Correct usage
tsa -y devices rename myserver newserver
tsa -y devices delete -d oldserver

# Incorrect usage (will not work)
tsa devices rename -y myserver newserver
tsa devices delete -d oldserver -y
```

### Device Matching

Devices can be specified by name, hostname, or ID. The matching priority is:

1. Exact match on device ID
2. Exact match on name
3. Exact match on hostname
4. Case-insensitive exact match
5. Partial match (contains) - returns **all matching devices**

When multiple devices match a pattern, you'll be shown the list and asked to confirm before proceeding.

**Note:** Device patterns can contain hyphens. If a pattern starts with a hyphen (e.g., `-gpu-1`), use the `-d` flag with the value directly:
```bash
tsa devices list -d -gpu-1
tsa devices info -gpu-server
```

### Interactive Device Selection

When you run certain commands without specifying device patterns (or omit the `-d` flag), you'll be presented with an interactive device selector, similar to tools like `yay`:

```bash
# Interactive selection for signing
tsa devices sign

# Interactive selection for tag operations
tsa devices add-tags -t tag:prod
tsa devices update-tags -t tag:server
tsa devices remove-tags -t tag:deprecated
```

The interactive selector displays a numbered list of devices and accepts:
- **Single numbers**: `1 3 5` - selects devices 1, 3, and 5
- **Ranges**: `1-5` - selects devices 1 through 5
- **Combinations**: `1 3-7 10` - selects device 1, devices 3-7, and device 10
- **All devices**: `all` or `*` - selects all devices
- **Cancel**: empty input or `none` - cancels the operation

### List Command

The `list` command automatically pages output to fit your terminal size, similar to `less` or `more`:

- If the output fits on one screen, it displays directly
- If the output is longer, it shows one page at a time with interactive controls:
  - Press `Space` or `Enter` to see the next page
  - Press `q` or `Esc` to quit
- Use `--no-paging` to disable paging and show all results at once
- Use `--columns` to select which columns to display

Available columns:
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
- Use `--json` to output data as JSON instead of a table (useful for scripting and piping to tools like `jq`)

## Device Management

### Device Information

Display detailed information about a specific device:

```bash
# Show device details
tsa devices info myserver

# Output as JSON for scripting
tsa devices info myserver --json

# Find device by various patterns
tsa devices info web-01          # by name
tsa devices info hostname.domain # by hostname
tsa devices info 12345          # by device ID
```

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

### Rename Device

Rename a device in your tailnet:

```bash
# Rename a device
tsa devices rename myserver new-server-name

# Rename without confirmation
tsa -y devices rename old-name new-name
```

### Delete Devices

Delete devices from your tailnet:

```bash
# Delete with interactive selection
tsa devices delete

# Delete specific devices by pattern
tsa devices delete -d myserver

# Filter by pattern, then select interactively
tsa devices delete -d server
# Shows only devices matching "server" for selection

# Delete without confirmation (use with caution!)
tsa -y devices delete -d old-server
```

The delete command:
- Shows a warning that the action cannot be undone
- Displays the devices that will be deleted
- Requires confirmation unless `-y` flag is used
- Supports both direct pattern matching and interactive selection

### Examples

```bash
# List all devices (with automatic paging)
tsa devices list

# List without paging
tsa devices list --no-paging

# Show only specific columns
tsa devices list --columns hostname,name,status

# Show device IDs and tags
tsa devices list --columns id,hostname,tags

# Show lock-related information
tsa devices list --columns hostname,locked,tailnet_lock_error

# Show all device details
tsa devices list --columns id,hostname,name,owner,os,status,tags,last_seen

# Combine column selection with locked devices
tsa devices list --locked --columns hostname,status,locked

# Output as JSON (useful for scripting)
tsa devices list --json

# Output locked devices as JSON
tsa devices list --locked --json

# Pipe JSON output to jq for filtering
tsa devices list --json | jq '.[] | select(.os == "linux")'

# Get just hostnames of online devices using jq
tsa devices list --json | jq -r '.[] | select(.lastSeen == "") | .hostname'

# Set tags on a specific device
tsa devices update-tags -d myserver -t tag:prod,tag:web

# Add a tag to all devices matching "server"
tsa devices add-tags -d server -t tag:monitored

# Remove a tag from multiple devices (skip confirmation)
tsa -y remove-tags -d server -t tag:deprecated

# Use multiple patterns
tsa devices add-tags -d web,api,db -t tag:production

# Tags can be specified with or without the "tag:" prefix
tsa devices add-tags -d myserver -t prod,web
# equivalent to:
tsa devices add-tags -d myserver -t tag:prod,tag:web

# Interactive device selection examples
# Add tags interactively (displays numbered list)
tsa devices add-tags -t tag:monitored
# Then enter: 1 3 5 (to select devices 1, 3, and 5)

# Sign devices interactively
tsa devices sign
# Then enter: 1-5 10 (to select devices 1 through 5, and device 10)

# Update tags for all devices interactively
tsa devices update-tags -t tag:production
# Then enter: all (to select all devices)

# Display detailed information about a device
tsa devices info myserver
tsa devices info web-01
tsa devices info 12345  # by device ID

# Get device info as JSON for scripting
tsa devices info myserver --json | jq '.tags'

# Rename a device
tsa devices rename old-name new-name
tsa devices rename myserver web-server-01

# Rename without confirmation
tsa -y devices rename myserver production-web-01

# Delete devices interactively
tsa devices delete
# Then select devices by number: 1 3 5 or 1-5

# Delete specific devices by pattern
tsa devices delete -d myserver
tsa devices delete -d "old-*"

# Delete with pattern filter, then interactive selection
tsa devices delete -d server
# This shows only devices matching "server" for selection
```

## User Management

You can manage users in your tailnet using the `tsa` CLI.

### List Users

```bash
# List all users in the tailnet
tsa users list

# Output as JSON
tsa users list --json
```

The user list displays:
- Login name (email)
- Display name
- Role (owner, admin, member, etc.)
- Status (active, suspended, etc.)
- Device count
- Currently active status

### User Operations

```bash
# Approve a pending user
tsa users approve -u user@example.com

# Suspend a user (prevents tailnet access)
tsa users suspend -u user@example.com

# Restore a suspended user
tsa users restore -u user@example.com

# Delete a user from the tailnet (permanent action)
tsa users delete -u user@example.com

# Skip confirmation prompts
tsa -y delete-user -u user@example.com
```

### Tailnet Contacts

View or list the tailnet's contact information:

```bash
# List all contacts (account, support, security)
tsa contacts list

# Output as JSON
tsa contacts list --json
```

### User Matching

When specifying users with the `-u` flag, you can match by:
1. Exact user ID
2. Exact login name (email)
3. Case-insensitive login name
4. Partial match on login name or display name

If multiple users match, you'll see a table of matches and be asked to provide a more specific pattern.

## Tailnet Lock Signing

If you have [Tailnet Lock](https://tailscale.com/kb/1226/tailnet-lock) enabled, you can use `tsa` to sign locked-out devices. This requires:

1. The machine running `tsa` must be a **signing node** (have a trusted Tailnet Lock key)
2. The `tailscale` CLI must be installed and accessible

### Sign Commands

```bash
# Interactive device selection (displays numbered list)
tsa devices sign
# Then enter device numbers: 1 3 5, or 1-5, or all

# Sign specific devices by pattern
tsa sign -d myserver

# Sign all devices matching a pattern
tsa sign -d server

# Sign without confirmation
tsa -y sign -d server
```

The `sign` command will:
1. Fetch devices from the API with their `nodeKey` and `tailnetLockKey`
2. Show the devices that will be signed
3. Ask for confirmation
4. Execute `tailscale lock sign <nodeKey> <tailnetLockKey>` for each device

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
