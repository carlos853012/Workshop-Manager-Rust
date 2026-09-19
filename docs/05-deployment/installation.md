# Installation

Step-by-step guide to build and install WorkshopManager.

## Building from Source

### 1. Clone the Repository

```powershell
git clone https://github.com/your-org/workshop-manager.git
cd workshop-manager
```

### 2. Build the Release Binaries

```powershell
cargo build --workspace --release
```

This produces two binaries in `target/release/`:

| Binary | Purpose |
|--------|---------|
| `workshop-server.exe` | Backend server (Axum + embedded PostgreSQL) |
| `workshop-viewer.exe` | Desktop UI (Dioxus) |

> First build takes 5-10 minutes due to LTO optimizations. Subsequent builds are fast.

### 3. Copy Configuration Files

```powershell
New-Item -ItemType Directory -Force -Path "target\release\config"
Copy-Item "config\server.toml" -Destination "target\release\config\"
Copy-Item "config\viewer.toml" -Destination "target\release\config\"
```

## Running for the First Time

### 1. Start the Server

```powershell
cd target\release
.\workshop-server.exe
```

On first run, the server will:
1. Generate TLS certificates (self-signed)
2. Generate JWT secret and encryption key
3. Start embedded PostgreSQL
4. Create the `workshop_manager` database
5. Run all migrations
6. Create the initial API key (if not configured)
7. Start listening on `https://127.0.0.1:8443`

> Keep the server terminal open. The server logs to stdout.

### 2. Start the Viewer

In a second terminal:

```powershell
cd target\release
.\workshop-viewer.exe
```

The desktop app will open and connect to the server.

## Platform-Specific Instructions

### Windows

**Prerequisites:**
- Windows 10/11
- WebView2 Runtime (pre-installed on most systems)
- Visual Studio Build Tools (for building)

**Build:**
```powershell
cargo build --workspace --release
```

**Run server as background service (optional):**
Use [NSSM](https://nssm.cc/) or Windows Task Scheduler to run `workshop-server.exe` at startup.

### Linux

**Prerequisites (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libsoup-3.0-dev \
    libjavascriptcoregtk-4.1-dev \
    libssl-dev \
    build-essential \
    pkg-config
```

**Build:**
```bash
cargo build --workspace --release
```

**Run:**
```bash
cd target/release
./workshop-server &
./workshop-viewer
```

**Create .deb package (optional):**
```bash
cargo deb -p workshop-server
```

### Installing the .deb Package (Recommended for Linux)

The easiest way to deploy on Ubuntu/Debian:

```bash
# Download the .deb from GitHub Releases
# Or build locally:
cargo deb -p workshop-server

# Install
sudo dpkg -i workshop-server_*.deb

# If there are missing dependencies
sudo apt-get install -f

# Verify installation
dpkg -L workshop-server
systemctl is-enabled workshop-server
```

The .deb package:
- Installs the binary to `/usr/bin/workshop-server`
- Installs the systemd service file to `/etc/systemd/system/workshop-server.service`
- Creates a `workshop-server` system user
- Creates the data directory at `/var/lib/workshop-server/`
- Enables and starts the service automatically

**Open firewall port:**
```bash
sudo ufw allow 8443/tcp
```

**Get the API key (required for viewer pairing):**
```bash
sudo cat /var/lib/workshop-server/.local/share/WorkshopManager/config/server.toml | grep api_key
```

## First-Time Setup Wizard

When the viewer connects to a fresh server, the setup wizard launches automatically:

### Step 1: Workshop Profile

| Field | Description | Example |
|-------|-------------|---------|
| Workshop Name | Your business name | "Taller Moto Express" |
| Address | Street address | "Av. Providencia 1234" |
| City | City | "Santiago" |

### Step 2: Admin Account

| Field | Description | Example |
|-------|-------------|---------|
| Full Name | Admin display name | "Carlos González" |
| Email | Login email | "admin@taller.cl" |
| Password | Minimum 8 characters | ******** |

### Step 3: First Login

Use the admin credentials created in Step 2 to log in.

## Verifying Installation

### Server Health Check

**Windows:**
```powershell
curl https://127.0.0.1:8443/health -k
# Expected: "OK"
```

**Linux:**
```bash
curl -k https://localhost:8443/health
# Expected: "OK"
```

### Check Database

The server logs will show:
```
INFO WorkshopManager Server...
INFO Config loaded
INFO Secrets initialized
INFO Crypto initialized
INFO PostgreSQL embedded started
INFO Database pool and migrations ready
INFO Server listening on https://127.0.0.1:8443
```

**Linux — view logs:**
```bash
sudo journalctl -u workshop-server -n 20 --no-pager
```

## Post-Installation

- See [configuration.md](configuration.md) to customize settings
- See [backup-recovery.md](backup-recovery.md) to configure backup strategy
- See [troubleshooting.md](troubleshooting.md) if you encounter issues
