# Development Environment Setup

This guide covers setting up a complete development environment for WorkshopManager.

---

## Prerequisites

### Required

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | stable (1.75+) | Compiler and toolchain |
| **Cargo** | Bundled with Rust | Build system and package manager |
| **Git** | 2.30+ | Version control |

### Recommended

| Tool | Purpose |
|------|---------|
| **rust-analyzer** | IDE support (VS Code, Neovim, etc.) |
| **Visual Studio Code** | Recommended IDE |
| **pgAdmin** or **DBeaver** | Database inspection |
| **Postman** or **Insomnia** | API testing |

### Install Rust (Windows)

```powershell
# Download and install rustup-init.exe from https://rustup.rs
# Or via winget:
winget install Rustlang.Rustup

# Verify installation
rustc --version
cargo --version
```

---

## Clone and Build

```powershell
# Clone the repository
git clone https://github.com/your-org/workshop-manager.git
cd workshop-manager

# Build the entire workspace
cargo build --workspace

# Build in release mode (optimized)
cargo build --workspace --release
```

The first build may take several minutes as dependencies are compiled.

---

## Running in Development Mode

### Server

```powershell
# Start the backend server (runs on :8443)
cargo run -p inventory-server
```

The server automatically:
- Starts an embedded PostgreSQL instance
- Runs all pending migrations
- Generates TLS certificates if missing
- Creates encryption keys if missing

### Viewer (Desktop App)

```powershell
# Start the Dioxus desktop app
cargo run -p inventory-viewer
```

### Both Simultaneously

Open two terminals:

```powershell
# Terminal 1 - Server
cargo run -p inventory-server

# Terminal 2 - Viewer
cargo run -p inventory-viewer
```

---

## Database Setup

The database is **fully auto-managed**. No manual PostgreSQL installation required.

- **Engine**: Embedded PostgreSQL via `postgresql_embedded`
- **Database**: `workshop_manager`
- **Location**: `{data_dir}/WorkshopManager/data/`
- **Migrations**: Applied automatically on server start

Data directory locations:
- **Windows**: `%LOCALAPPDATA%/WorkshopManager/data/`
- **macOS**: `~/Library/Application Support/WorkshopManager/data/`
- **Linux**: `~/.local/share/WorkshopManager/data/`

---

## Useful Commands

### Build Commands

```powershell
cargo build --workspace                    # Build all crates
cargo build -p inventory-server            # Build server only
cargo build -p inventory-viewer            # Build viewer only
cargo check -p inventory-viewer            # Fast type-check (no codegen)
```

### Quality Commands

```powershell
cargo clippy --workspace -- -D warnings    # Lint (warnings = errors)
cargo fmt --all --check                    # Check formatting
cargo fmt --all                            # Auto-format code
```

### Test Commands

```powershell
cargo test --workspace                     # Run all tests
cargo test -p inventory-server             # Server tests only
cargo test -p inventory-common             # Common tests only
```

### Version Management

```powershell
.\scripts\bump.ps1 0.2.0   # Bump version, build, commit, tag
```

---

## IDE Configuration

### VS Code (rust-analyzer)

Install the `rust-analyzer` extension. Recommended `settings.json`:

```json
{
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.check.allTargets": true,
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.linkedProjects": [
        "Cargo.toml"
    ]
}
```

### rust-analyzer Features

- **Inlay hints**: Type inference display
- **Go to definition**: Navigate across crates
- **Inline hints**: See macro expansions
- **Code actions**: Auto-import, fill implementations

---

## Debugging Tips

### Server Debugging

```powershell
# Run with debug logging
RUST_LOG=debug cargo run -p inventory-server

# Run with SQL query logging
RUST_LOG=sqlx=debug cargo run -p inventory-server
```

### Viewer Debugging

The Dioxus viewer opens a webview. Use browser DevTools:
- Right-click → Inspect Element (if enabled in build)
- Check console for JavaScript errors
- Network tab for API calls

### Common Issues

| Issue | Solution |
|-------|----------|
| Port 8443 in use | Change port in `config/server.toml` |
| Build fails on TLS | Ensure `ring` and `rcgen` compile (needs C toolchain on Windows) |
| DB lock error | Kill any existing server process |
| CSS not updating | Rebuild viewer (`cargo build -p inventory-viewer`) |

---

## First Run Checklist

1. ✅ `cargo build --workspace` completes without errors
2. ✅ Server starts and shows "Listening on 0.0.0.0:8443"
3. ✅ Viewer opens and shows login/setup screen
4. ✅ Create first workshop via Setup wizard
5. ✅ Login with created credentials
