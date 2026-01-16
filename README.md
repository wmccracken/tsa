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
# List all devices
tsa list

# List locked-out devices (tailnet lock)
tsa list --locked

# Update tags (replaces existing tags)
tsa update-tags -d <device-pattern> -t <tags>

# Add tags (keeps existing tags)
tsa add-tags -d <device-pattern> -t <tags>

# Remove tags
tsa remove-tags -d <device-pattern> -t <tags>

# Sign locked-out devices (tailnet lock)
tsa sign -d <device-pattern>
```

### Options

| Flag | Description |
|------|-------------|
| `-a, --api-key` | Tailscale API key |
| `-n, --tailnet` | Tailnet name (default: `-`) |
| `-y, --yes` | Skip confirmation prompts |

### Device Matching

Devices can be specified by name, hostname, or ID. The matching priority is:

1. Exact match on device ID
2. Exact match on name
3. Exact match on hostname
4. Case-insensitive exact match
5. Partial match (contains) - returns **all matching devices**

When multiple devices match a pattern, you'll be shown the list and asked to confirm before proceeding.

### Examples

```bash
# List all devices
tsa list

# Set tags on a specific device
tsa update-tags -d myserver -t tag:prod,tag:web

# Add a tag to all devices matching "server"
tsa add-tags -d server -t tag:monitored

# Remove a tag from multiple devices (skip confirmation)
tsa -y remove-tags -d server -t tag:deprecated

# Use multiple patterns
tsa add-tags -d web,api,db -t tag:production

# Tags can be specified with or without the "tag:" prefix
tsa add-tags -d myserver -t prod,web
# equivalent to:
tsa add-tags -d myserver -t tag:prod,tag:web
```

## Tailnet Lock Signing

If you have [Tailnet Lock](https://tailscale.com/kb/1226/tailnet-lock) enabled, you can use `tsa` to sign locked-out devices. This requires:

1. The machine running `tsa` must be a **signing node** (have a trusted Tailnet Lock key)
2. The `tailscale` CLI must be installed and accessible

### Sign Commands

```bash
# List all devices with their lock keys (to see what can be signed)
tsa sign

# Sign specific devices
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

## License

MIT
