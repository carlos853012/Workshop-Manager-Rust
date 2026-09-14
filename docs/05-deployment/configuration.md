# Configuration

Reference for all configuration files and settings.

## Configuration Files

| File | Location | Purpose |
|------|----------|---------|
| `server.toml` | `config/server.toml` | Server settings |
| `viewer.toml` | `config/viewer.toml` | Viewer/client settings |

Both files are TOML format and are auto-generated on first run if missing.

## server.toml

```toml
[server]
host = "127.0.0.1"
port = 8443
api_key = "dev-key-change-in-production"
require_device_key = false
max_viewers = 2

[tax]
iva_rate = 0.19

[icon]
bg = "#F59E0B"
fg = "#FFFFFF"
```

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `host` | string | `127.0.0.1` | Bind address. Use `0.0.0.0` for LAN access. |
| `port` | integer | `8443` | HTTPS listen port. |
| `api_key` | string | *(auto-generated)* | Shared secret for API authentication. Auto-generates a random 64-char hex key if left empty or set to the default. |
| `require_device_key` | boolean | `false` | When `true`, clients must present a valid device key in addition to the API key. |
| `max_viewers` | integer | `2` | Maximum concurrent viewer connections. Overridden by license tier. |

### [tax] Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `iva_rate` | float | `0.19` | IVA tax rate as decimal (0.19 = 19%). |

### [icon] Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bg` | string | `#F59E0B` | Background color of the application icon (hex). |
| `fg` | string | `#FFFFFF` | Foreground (wrench) color of the application icon (hex). |

### Environment Variable Override

The API key can be overridden via environment variable:

```powershell
$env:WORKSHOP_MANAGER_API_KEY = "your-production-api-key"
```

This takes precedence over the value in `server.toml`.

## viewer.toml

```toml
[server]
base_url = "https://127.0.0.1:8443"
api_key = "dev-key-change-in-production"
device_key = ""
```

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `base_url` | string | `https://127.0.0.1:8443` | Server URL to connect to. |
| `api_key` | string | `dev-key-change-in-production` | Must match the server's API key. |
| `device_key` | string | *(empty)* | Device key for device-bound authentication. Leave empty if `require_device_key` is `false`. |

### TLS Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `tls_accept_invalid_certs` | boolean | `true` | Accept self-signed certificates. Set to `false` when using a CA-signed certificate. |

## Secrets Management

Secrets are stored in the data directory:

| Platform | Path |
|----------|------|
| Windows | `%LOCALAPPDATA%\WorkshopManager\data\` |
| Linux | `~/.local/share/WorkshopManager/data/` |

### Files

| File | Size | Purpose |
|------|------|---------|
| `.jwt_secret` | 64 chars (hex) | HMAC-SHA256 key for JWT token signing |
| `.crypto_key` | 32 bytes (binary) | AES-256-GCM key for encrypting credentials and backups |
| `server.crt` | PEM | TLS certificate (auto-generated or custom) |
| `server.key` | PEM | TLS private key (auto-generated or custom) |

> On Windows, these files are marked as hidden (`attrib +H`).
> On Unix, permissions are set to `0600` (owner read/write only).

### Regenerating Secrets

Delete the secret files and restart the server:

```powershell
Remove-Item "$env:LOCALAPPDATA\WorkshopManager\data\.jwt_secret"
Remove-Item "$env:LOCALAPPDATA\WorkshopManager\data\.crypto_key"
```

> **Warning:** This invalidates all existing JWT tokens and makes encrypted data unreadable.

## TLS Certificates

### Auto-Generated (Default)

On first run, the server generates a self-signed certificate valid for:
- `localhost`
- `127.0.0.1`
- CN: `WorkshopManager Server`

Certificate files are stored in the data directory:
- `server.crt`
- `server.key`

### Custom Certificates

To use your own certificate:

1. Replace `server.crt` in the data directory
2. Replace `server.key` in the data directory
3. Restart the server

For production, use a certificate from a trusted CA or an internal PKI.

## Database Location

The embedded PostgreSQL stores data at:

| Platform | Path |
|----------|------|
| Windows | `%LOCALAPPDATA%\WorkshopManager\data\` |
| Linux | `~/.local/share/WorkshopManager/data/` |

The database is named `workshop_manager`. PostgreSQL binaries and data files are stored within this directory.

## Backup Configuration

Backups are configured in code with these defaults:

| Setting | Value | Source |
|---------|-------|--------|
| Backup interval | 24 hours | `main.rs:210` |
| Retention count | 7 backups | `main.rs:211` |
| Backup location | `$DATA_DIR/backups/` | `main.rs:77` |
| Format | `pg_dump` + gzip + AES-256-GCM | `backup.rs` |

See [backup-recovery.md](backup-recovery.md) for details.
