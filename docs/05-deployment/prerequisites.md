# Prerequisites

System requirements for building and running WorkshopManager.

## Hardware Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 2 GB | 4+ GB |
| Disk | 500 MB free | 2 GB+ free |

> The embedded PostgreSQL database grows with usage. Plan for ~100 MB per year of active operation.

## Software Requirements

### Required

| Component | Version | Notes |
|-----------|---------|-------|
| Rust (stable) | 1.75+ | Install via [rustup](https://rustup.rs) |
| Cargo | Bundled with Rust | |

### Windows

| Component | Version | Notes |
|-----------|---------|-------|
| WebView2 Runtime | Latest | Pre-installed on Windows 10/11. [Download](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
| Visual Studio C++ Build Tools | 2019+ | Required for native dependencies |

### Linux

| Component | Version | Notes |
|-----------|---------|-------|
| libwebkit2gtk-4.1 | 4.1+ | `sudo apt install libwebkit2gtk-4.1-dev` |
| libgtk-3-dev | 3.24+ | `sudo apt install libgtk-3-dev` |
| libsoup-3.0-dev | 3.0+ | `sudo apt install libsoup-3.0-dev` |
| libjavascriptcoregtk-4.1-dev | 4.1+ | `sudo apt install libjavascriptcoregtk-4.1-dev` |

### Packaging Tools (Optional)

| Tool | Purpose | Install |
|------|---------|---------|
| [WiX Toolset](https://wixtoolset.org/) | MSI installer (Windows) | `winget install WixToolset` |
| [cargo-deb](https://github.com/kornelski/cargo-deb) | .deb package (Linux) | `cargo install cargo-deb` |

## Network Requirements

| Port | Protocol | Binding | Notes |
|------|----------|---------|-------|
| 8443 | HTTPS | `127.0.0.1` (default) | Main server port |

> By default the server binds to localhost only. For LAN access, change `host` in `config/server.toml` to `0.0.0.0` and configure firewall rules.

No external internet access is required at runtime. The application is fully self-contained.

## Database Requirements

**None.** WorkshopManager uses an embedded PostgreSQL instance that:

- Starts automatically with the server
- Creates the `workshop_manager` database on first run
- Runs migrations automatically
- Requires no manual installation or configuration

Database files are stored at:
- **Windows:** `%LOCALAPPDATA%\WorkshopManager\data\`
- **Linux:** `~/.local/share/WorkshopManager/data/`

## TLS Certificates

TLS certificates are **auto-generated** on first run using `rcgen` (self-signed, for `localhost` and `127.0.0.1`). No manual certificate setup is needed.

To use custom certificates, replace the files at:
- **Certificate:** `$DATA_DIR/server.crt`
- **Private key:** `$DATA_DIR/server.key`

## Quick Verification

After installing Rust, verify your environment:

```powershell
rustc --version    # Should show 1.75+
cargo --version    # Should show cargo 1.75+
```

On Linux, verify webview dependencies:

```bash
pkg-config --modversion webkit2gtk-4.1   # Should show 4.1+
```
