# Technology Stack — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-14
**Audience:** Developers, DevOps, technical stakeholders

---

## 1. Dependency Inventory

### 1.1 workshop-common Dependencies

| Crate | Version | Purpose | License | Notes |
|-------|---------|---------|---------|-------|
| `serde` | 1.0 | Serialization/deserialization framework | MIT/Apache-2.0 | `derive` feature |
| `serde_json` | 1.0 | JSON serialization | MIT/Apache-2.0 | Used for DTOs and audit trail |
| `chrono` | 0.4 | Date/time handling | MIT/Apache-2.0 | `serde` feature for JSON |
| `sqlx` | 0.7 | SQLx types and traits | MIT/Apache-2.0 | `postgres`, `chrono`, `uuid`, `rust_decimal` features |
| `rust_decimal` | 1.0 | Arbitrary-precision decimals | MIT | `serde-with-str` for JSON |
| `uuid` | 1.0 | UUID generation and parsing | MIT/Apache-2.0 | `v4` (random), `serde` feature |
| `thiserror` | 1.0 | Derive macro for Error trait | MIT/Apache-2.0 | Type-safe error definitions |
| `ed25519-dalek` | 2.1 | Ed25519 digital signatures | Apache-2.0 | License system signing |
| `sha2` | 0.10 | SHA-256 hashing | MIT/Apache-2.0 | Hardware hash, device keys |
| `resvg` | 0.48 | SVG rendering | MIT | Icon generation (default features disabled) |

### 1.2 workshop-server Dependencies

| Crate | Version | Purpose | License | Notes |
|-------|---------|---------|---------|-------|
| `workshop-common` | path | Shared types and logic | Proprietary | Local workspace crate |
| `axum` | 0.7 | HTTP framework | MIT | Tower-based, async |
| `tokio` | 1.x | Async runtime | MIT | `full` feature set |
| `sqlx` | 0.7 | Async PostgreSQL driver | MIT/Apache-2.0 | `runtime-tokio`, `postgres`, `chrono`, `uuid`, `migrate` |
| `postgresql_embedded` | 0.20 | Embedded PostgreSQL server | MIT | `bundled`, `rustls` features |
| `flate2` | 1.0 | Gzip compression | MIT/Apache-2.0 | Backup compression |
| `rcgen` | 0.13 | TLS certificate generation | MIT | Self-signed certificates |
| `axum-server` | 0.7 | HTTPS server | MIT | `tls-rustls` feature |
| `rustls` | 0.23 | TLS implementation | MIT/Apache-2.0 | `ring` crypto provider |
| `rustls-pemfile` | 2.0 | PEM file parsing | MIT/Apache-2.0 | Certificate loading |
| `rust_decimal` | 1.0 | Decimal arithmetic | MIT | Financial calculations |
| `tracing` | 0.1 | Structured logging | MIT | Application logging |
| `tracing-subscriber` | 0.3 | Log formatting | MIT | Console output |
| `argon2` | 0.5 | Password hashing | MIT/Apache-2.0 | Argon2id algorithm |
| `jsonwebtoken` | 9 | JWT creation/validation | MIT | HS256 support |
| `aes-gcm` | 0.10 | AES-256-GCM encryption | MIT/Apache-2.0 | Data at rest encryption |
| `tower-http` | 0.6 | HTTP middleware | MIT | `cors`, `limit`, `trace`, `set-header` |
| `serde` | 1.0 | Serialization | MIT/Apache-2.0 | `derive` feature |
| `serde_json` | 1.0 | JSON handling | MIT/Apache-2.0 | Request/response bodies |
| `toml` | 0.8 | TOML configuration | MIT | Config file parsing |
| `dirs` | 5.0 | OS-specific directories | MIT | Data directory resolution |
| `uuid` | 1.0 | UUID generation | MIT/Apache-2.0 | Entity identifiers |
| `chrono` | 0.4 | Date/time | MIT/Apache-2.0 | Timestamps |
| `rand` | 0.8 | Random number generation | MIT/Apache-2.0 | Secret generation |
| `sha2` | 0.10 | SHA-256 hashing | MIT/Apache-2.0 | Device key hashing |
| `base64` | 0.22 | Base64 encoding | MIT/Apache-2.0 | Encrypted data encoding |
| `anyhow` | 1.0 | Error handling | MIT | Application-level errors |
| `regex` | 1 | Regular expressions | MIT/Apache-2.0 | Input validation |
| `thiserror` | 1.0 | Error derive | MIT/Apache-2.0 | Typed errors |
| `genpdf` | 0.2 | PDF generation | MIT | Service certificates |

**Windows-only:**

| Crate | Version | Purpose | License | Notes |
|-------|---------|---------|---------|-------|
| `tao` | 0.30 | Window management | Apache-2.0 | Desktop window for tray |
| `tray-icon` | 0.19 | System tray icon | MIT | Windows system tray |
| `winreg` | 0.52 | Windows registry | MIT | Registry access |
| `clipboard-win` | 5.4 | Clipboard access | MIT | Copy/paste support |

### 1.3 workshop-viewer Dependencies

| Crate | Version | Purpose | License | Notes |
|-------|---------|---------|---------|-------|
| `workshop-common` | path | Shared types and logic | Proprietary | Local workspace crate |
| `dioxus` | 0.6 | Desktop UI framework | MIT/Apache-2.0 | `desktop` feature |
| `dioxus-desktop` | 0.6 | Desktop runtime | MIT/Apache-2.0 | WebView-based rendering |
| `dioxus-router` | 0.6 | Client-side routing | MIT/Apache-2.0 | Page navigation |
| `reqwest` | 0.12 | HTTP client | MIT/Apache-2.0 | `json`, `rustls-tls` features |
| `serde` | 1.0 | Serialization | MIT/Apache-2.0 | `derive` feature |
| `serde_json` | 1.0 | JSON handling | MIT/Apache-2.0 | API responses |
| `tokio` | 1.x | Async runtime | MIT | `rt-multi-thread`, `time` |
| `chrono` | 0.4 | Date/time | MIT/Apache-2.0 | Timestamps |
| `uuid` | 1.x | UUID handling | MIT/Apache-2.0 | `serde` feature |
| `toml` | 0.8 | TOML config | MIT | Viewer configuration |
| `rust_decimal` | 1.35 | Decimal handling | MIT | Price display |

### 1.4 Workspace-Level Dependencies

| Crate | Version | Purpose | License | Notes |
|-------|---------|---------|---------|-------|
| `tray-icon` | 0.19 | System tray (workspace override) | MIT | All features disabled to avoid GTK init |

### 1.5 license-tool Dependencies

| Crate | Version | Purpose | License | Notes |
|-------|---------|---------|---------|-------|
| `workshop-common` | path | Shared types and licensing logic | Proprietary | Local workspace crate |
| `clap` | 4 | CLI argument parsing | MIT | `derive` feature |
| `serde_json` | 1.0 | JSON serialization | MIT/Apache-2.0 | License data |
| `chrono` | 0.4 | Date/time | MIT/Apache-2.0 | License timestamps |

---

## 2. Rust Ecosystem Choices

### 2.1 Why Axum (not Actix-web, Rocket, Warp)?

| Criterion | Axum 0.7 | Actix-web 4 | Rocket 0.5 | Warp 0.3 |
|-----------|----------|-------------|------------|----------|
| Architecture | Tower-native | Actor-based | Custom | Filter-based |
| Learning curve | Low | Medium | Low | High |
| Middleware | Tower layers | Custom + Tower | Fairings | Custom |
| State extraction | `FromRef` trait | App data | Managed state | Custom |
| Async model | Tokio | Tokio | Tokio | Tokio |
| Type safety | Excellent | Good | Good | Good |
| Ecosystem | Growing fast | Mature | Moderate | Small |
| Performance | Excellent | Excellent | Good | Excellent |
| Compile times | Fast | Medium | Medium | Fast |

**Decision rationale:**
- Axum's Tower-native design integrates seamlessly with the Tower middleware ecosystem
- `FromRef<AppState>` provides clean, type-safe state extraction
- Minimal boilerplate for route definitions
- Strong community support and active development
- No macro-heavy syntax (unlike Rocket)

### 2.2 Why Dioxus (not Tauri, Qt, GTK)?

| Criterion | Dioxus 0.6 | Tauri 1.x | Qt (qmetaobject) | GTK (gtk-rs) |
|-----------|-----------|-----------|-------------------|--------------|
| Language | Rust | Rust + JS/TS | Rust + QML | Rust |
| Rendering | System WebView | System WebView | Native widgets | Native widgets |
| Learning curve | Low (RSX ≈ JSX) | Medium | High | High |
| UI language | RSX (Rust) | HTML/CSS/JS | QML | XML + Rust |
| Bundle size | Small | Small | Large | Large |
| Cross-platform | Yes | Yes | Yes | Linux/Windows |
| Community | Growing | Large | Moderate | Moderate |
| Rust-native types | Yes | Partial | No | Yes |
| Desktop features | Good | Good | Excellent | Good |

**Decision rationale:**
- RSX syntax is familiar to web developers (similar to JSX)
- Full Rust type safety with shared `workshop-common` types
- System WebView avoids bundling Chromium (unlike Electron)
- Simpler architecture than Tauri for desktop-only apps
- Active development with frequent releases

### 2.3 Why SQLx (not Diesel, SeaORM, rusqlite)?

| Criterion | SQLx 0.7 | Diesel 2.x | SeaORM 1.x | rusqlite |
|-----------|---------|------------|------------|----------|
| Database | PostgreSQL (async) | PostgreSQL (sync/async) | PostgreSQL (async) | SQLite (sync) |
| Type safety | Compile-time checked | Compile-time checked | Runtime checked | Runtime checked |
| Async | Native Tokio | Tokio (diesel-async) | Tokio | No |
| Migrations | Built-in | Built-in | Built-in | Manual |
| Query style | Raw SQL (checked) | DSL | DSL + Raw SQL | Raw SQL |
| Learning curve | Low | Medium | Medium | Low |
| Embedded PG | Yes | Yes | Yes | No (SQLite only) |
| Features | Minimal | Full ORM | Full ORM | Minimal |

**Decision rationale:**
- SQLx's compile-time query checking catches SQL errors at build time
- Raw SQL provides full control over queries (important for `SELECT ... FOR UPDATE`)
- Native Tokio async integration
- Built-in migration system with compile-time embedding
- Minimal abstraction over PostgreSQL features (enums, JSONB, INET)

---

## 3. Database Choice Rationale

### 3.1 PostgreSQL Embedded vs. Alternatives

| Criterion | PostgreSQL (embedded) | SQLite | MySQL (embedded) |
|-----------|----------------------|--------|-------------------|
| ACID compliance | Full | Full (with WAL) | Full |
| Row-level locking | `SELECT ... FOR UPDATE` | Limited (database-level) | `SELECT ... FOR UPDATE` |
| Enum types | Native `CREATE TYPE` | CHECK constraints | Native `ENUM` |
| JSONB support | Native | Limited (JSON) | Native |
| INET type | Native | No (TEXT) | No |
| UUID type | Native | No (TEXT) | No |
| Concurrency | Excellent (MVCC) | Single-writer | Good |
| Embedded binary | ~50MB | ~1MB | ~30MB |
| Memory usage | ~64MB baseline | ~1MB | ~30MB |
| pg_dump | Yes | `.backup` command | `mysqldump` |
| Tooling | Excellent (psql, pgAdmin) | Good (DB Browser) | Good (Workbench) |

**Decision rationale:**
- Full PostgreSQL feature set (enums, JSONB, INET, row locking) is critical for the domain
- `SELECT ... FOR UPDATE` is essential for atomic stock deduction
- Native enum types enforce data integrity at the database level
- JSONB for audit trail provides flexible schema for old/new values
- `postgresql_embedded` bundles the server, requiring no external installation
- `pg_dump` provides reliable, well-tested backup capability

### 3.2 Database Schema Summary

| Table | Purpose | Key Relationships |
|-------|---------|-------------------|
| `workshops` | Workshop tenant isolation | Parent of all domain tables |
| `users` | User accounts with RBAC | `workshop_id` FK |
| `products` | Product catalog | `supplier_id` FK, `workshop_id` filter |
| `sales` | Sales transactions | `workshop_id` FK |
| `sale_items` | Individual sale line items | `sale_id` FK, `product_id` FK |
| `repairs` | Repair orders | `technician_id` FK → users, `workshop_id` FK |
| `repair_updates` | Repair status history | `repair_id` FK, `created_by` FK → users |
| `repair_parts` | Parts used in repairs | `repair_id` FK, `product_id` FK |
| `suppliers` | Supplier directory | `workshop_id` FK |
| `audit_log` | Audit trail | `user_id` FK → users |
| `device_keys` | Device binding keys | No FK (hash-based lookup) |

---

## 4. Security Libraries

### 4.1 argon2 0.5 — Password Hashing

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Algorithm | Argon2id | OWASP-recommended hybrid (data-independent + data-dependent) |
| Memory | 64 MB (65536 KB) | Exceeds OWASP minimum (19 MB) |
| Iterations | 3 | OWASP minimum recommendation |
| Parallelism | 4 | Matches typical CPU core count |
| Salt | Random 16 bytes via `OsRng` | Cryptographically secure |
| Output | Variable-length hash string | Includes algorithm, params, salt, and hash |

```rust
// Implementation in auth.rs
fn argon2_params() -> Argon2<'static> {
    let params = Params::new(65536, 3, 4, None).expect("valid argon2 params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}
```

### 4.2 aes-gcm 0.10 — Symmetric Encryption

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Algorithm | AES-256-GCM | NIST-approved, authenticated encryption |
| Key size | 256 bits (32 bytes) | Maximum AES security level |
| Nonce size | 96 bits (12 bytes) | Standard for AES-GCM |
| Authentication | GCM (Galois/Counter Mode) | Provides confidentiality + integrity |
| Key storage | Random file at `$LOCALAPPDATA/WorkshopManager/data/.crypto_key` | Generated on first run |

```
Encryption format: base64(nonce || ciphertext || auth_tag)
Example: encrypt("my_password") → "aB3dEfGhIjKlMnOpQrStUvWxYz1234567890..."
```

### 4.3 jsonwebtoken 9 — JWT Tokens

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Algorithm | HS256 (HMAC-SHA256) | Symmetric, simple, sufficient for single-server |
| Secret | 32 random bytes (64 hex chars) | Generated via `OsRng` |
| Expiry | 8 hours | Balances security and usability |
| Claims | `sub`, `email`, `role`, `workshop_id`, `exp` | Minimal required claims |
| Validation | Default (checks `exp`) | Automatic expiration check |

```rust
pub struct Claims {
    pub sub: String,        // User UUID
    pub email: String,      // User email
    pub role: String,       // "admin" | "mechanic" | "seller"
    pub workshop_id: String, // Workshop UUID
    pub exp: usize,         // Expiration timestamp
}
```

### 4.4 rcgen 0.13 — TLS Certificate Generation

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Algorithm | ECDSA P-256 | Modern, efficient, widely supported |
| SAN | `localhost`, `127.0.0.1` | Self-signed for local use |
| Validity | 1 year | Long-lived for desktop app |
| Storage | PEM files in data directory | Standard format |
| Server | `axum-server` with `tls-rustls` | Rust-native TLS |

### 4.5 ed25519-dalek 2.1 — License Signatures

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Algorithm | Ed25519 | Fast, small keys (32 bytes), EdDSA |
| Key size | 32 bytes (private), 32 bytes (public) | Compact |
| Use case | License key signing | Verify license authenticity |
| Hardware binding | SHA-256 hash of CPU+MB+Disk | Optional machine binding |

---

## 5. Build and Deployment Tools

### 5.1 Build Configuration

```toml
# Workspace Cargo.toml
[profile.release]
strip = true        # Remove debug symbols
lto = true          # Link-Time Optimization
codegen-units = 1   # Maximum optimization
panic = "abort"     # Smaller binary, no unwinding
opt-level = "z"     # Optimize for size
```

### 5.2 Build Commands

```powershell
# Development build
cargo build --workspace

# Release build (optimized)
cargo build --workspace --release

# Server only
cargo build -p workshop-server --release

# Viewer only
cargo build -p workshop-viewer --release

# Fast type check
cargo check -p workshop-viewer

# Linter
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --all --check

# Run tests
cargo test --workspace
```

### 5.3 Version Bumping

```powershell
# Automated version bump
.\scripts\bump.ps1 0.2.0

# What bump.ps1 does:
# 1. Updates [workspace.package] version in Cargo.toml
# 2. Runs cargo build --release
# 3. Commits changes with version tag
# 4. Creates git tag v0.2.0
```

### 5.4 CI/CD Pipeline (Recommended)

```yaml
# .github/workflows/ci.yml (recommended)
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace
      - run: cargo build -p workshop-server --release
      - run: cargo build -p workshop-viewer --release
```

### 5.5 Release Profile

| Setting | Value | Impact |
|---------|-------|--------|
| `strip = true` | Remove debug symbols | ~30% size reduction |
| `lto = true` | Link-Time Optimization | ~10-15% size/perf improvement |
| `codegen-units = 1` | Single codegen unit | Better optimization, slower build |
| `panic = "abort"` | No unwinding | ~5% size reduction |
| `opt-level = "z"` | Optimize for size | Smaller binary |

---

## 6. Version Compatibility Matrix

### 6.1 Rust Toolchain

| Component | Version | Notes |
|-----------|---------|-------|
| Rust | 1.75+ (stable) | Edition 2021 |
| Cargo | Bundled with Rust | Workspace resolver v2 |
| rustup | Latest | For toolchain management |

### 6.2 Key Crate Versions

| Crate | Version | Compatibility Notes |
|-------|---------|-------------------|
| axum | 0.7 | Requires tokio 1.x, hyper 1.x |
| tokio | 1.x | Full feature set for server |
| sqlx | 0.7 | Compile-time checked queries |
| dioxus | 0.6 | Desktop feature, RSX syntax |
| serde | 1.0 | Derive macros |
| postgresql_embedded | 0.20 | Bundled PostgreSQL binary |

### 6.3 Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows x64 | Primary | System tray, clipboard, registry |
| macOS ARM64 | Secondary | WebView rendering |
| Linux x64 | Secondary | WebView rendering, no tray |

### 6.4 Database Compatibility

| Database | Version | Status |
|----------|---------|--------|
| PostgreSQL (embedded) | 16.x | Primary, bundled |
| PostgreSQL (external) | 14+ | Supported via config |

### 6.5 Browser/WebView Compatibility

| WebView | Platform | Status |
|---------|----------|--------|
| WebView2 (Edge) | Windows | Primary |
| WKWebView | macOS | Secondary |
| WebKitGTK | Linux | Secondary |

---

## 7. License Summary

| License | Count | Notable Crates |
|---------|-------|---------------|
| MIT | 30+ | axum, tokio, serde, argon2, aes-gcm |
| MIT/Apache-2.0 | 20+ | sqlx, uuid, chrono, jsonwebtoken |
| Apache-2.0 | 5 | ed25519-dalek, tao, reqwest |
| Proprietary | 1 | workshop-common, workshop-server, workshop-viewer |

All dependencies use permissive licenses (MIT, Apache-2.0). No copyleft (GPL/AGPL) dependencies are used, preserving the proprietary license for the application.

---

## 8. Cross-References

| Document | Description |
|----------|-------------|
| [Architecture Overview](./overview.md) | High-level system purpose and components |
| [System Design](./system-design.md) | C4 diagrams, integration patterns, concurrency model |
| [Data Flow](./data-flow.md) | Request lifecycle, authentication flow, POS transaction flow |
| `Cargo.toml` | Workspace configuration and dependency declarations |
| `crates/*/Cargo.toml` | Individual crate dependency declarations |
