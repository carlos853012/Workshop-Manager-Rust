# AGENTS.md — WorkshopManager

## Project status
Scaffold at **Phase 1-2 of 8** (`PLAN_DESARROLLO.md`). Most modules are stubs. Version `0.1.0`. Domain: motorcycle workshop (products, sales, repairs, suppliers).

## Workspace structure
3 crates in a Cargo workspace:
- `crates/inventory-common` — shared types (Product, Sale, SaleItem, Repair, Supplier, User, AuditLog), enums, DTOs, licensing (Ed25519 hardware-bound)
- `crates/inventory-server` — Axum 0.7 backend on `:8443`, stub routes, no DB, no TLS
- `crates/inventory-viewer` — Dioxus 0.6 Desktop, single static page, no routing yet

**Important:** Package names are `inventory-common`, `inventory-server`, `inventory-viewer` — NOT `common`, `server`, `viewer`.

## Build commands
```powershell
cargo build --workspace                    # build all
cargo build -p inventory-server            # server only
cargo build -p inventory-viewer            # viewer only
cargo check -p inventory-viewer            # fast verify
cargo clippy --workspace -- -D warnings    # linter
cargo fmt --all --check                    # format check
cargo test --workspace                     # tests
```

## Version bump
```powershell
.\scripts\bump.ps1 0.2.0   # updates [workspace.package] version, builds, commits, tags
git push && git push --tags
```

## CI bug
`.github/workflows/ci.yml` uses `-p common`, `-p server`, `-p viewer` — these are wrong. Correct names are `-p inventory-common`, `-p inventory-server`, `-p inventory-viewer`. Clippy and build steps will fail until fixed.

## Domain types (inventory-common)
Structs: `Product`, `Sale`, `SaleItem`, `Repair`, `RepairUpdate`, `Supplier`, `User`, `AuditLog`
Enums: `PaymentMethod` (Cash/Card/Transfer), `RepairStatus` (Pending/InProgress/Completed/Cancelled), `Priority` (High/Medium/Low), `UserRole` (Admin/Mechanic/Seller)
DTOs: `LoginRequest`, `CreateProductRequest`, `CreateSaleRequest`, `CreateRepairRequest`, `ApiResponse<T>`, `PaginatedResponse<T>`, `LoginResponse`, `DashboardResponse`
License: `Feature` enum (15 features across 4 tiers), `License` struct with Ed25519 signature verification

## Server architecture
- Entrypoint: `crates/inventory-server/src/main.rs`
- Config: `config/server.toml` (TOML, loaded but `host`/`port` not used — binding hardcoded to `0.0.0.0:8443`)
- Secrets: generated/persisted at `dirs::data_local_dir()/WorkshopManager/data/`
- Crypto: AES-256-GCM via `crypto::init()` with static cipher
- Routes: stub JSON responses under `/api` (auth, products, sales, repairs, suppliers, analytics, users)
- Health: `GET /health` returns `"OK"`
- No database connection, no migrations, no TLS, no audit logging yet

## Viewer architecture
- Entrypoint: `crates/inventory-viewer/src/main.rs`
- CSS inlined at compile time via `include_str!("../index.css")` — **rebuild to see CSS changes**
- Atomic Design scaffold: `components/{atoms,molecules,organisms}/` (all empty)
- No routing, no pages, no icons yet

## Known issues
- **Server unused deps:** `jsonwebtoken`, `tracing-appender`, `obfstr` in Cargo.toml but never imported
- **`static mut` in crypto.rs:** uses `unsafe` blocks (violates constitutional rule)
- **`unwrap()` in rate_limiter.rs:** lines 24, 41 (violates constitutional rule)
- **`.clinerules`/`.cursorrules`/`.geminirules`:** broken link to `file:///C:/Users/carlos/Desktop/Equipos-Rust/CONSTITUCION.md` — actual path is `CONSTITUCION.md` in workspace root
- **CONSTITUCION.md:** still contains OT/SCADA rules (Section 16) from Equipos-Rust — should be adapted to motorcycle workshop domain
- **`scripts/bump.ps1`:** works correctly (updates workspace version, builds, commits, tags)
- **`.gitignore`:** references `crates/server/data/` and `crates/viewer/data/` — actual paths are `crates/inventory-server/` and `crates/inventory-viewer/`

## Constitutional rules (from CONSTITUCION.md)
All code MUST comply. Key rules:
- **No `unsafe`**. No `unwrap()`, `expect()`, or `panic!()` — use `Result<T, E>`
- **Parameterized SQL only** — never concatenate strings
- **Encrypt credentials** — use `crypto::encrypt_opt`/`decrypt_opt`
- **Validate all inputs** before DB insertion
- **Minimal changes** — localized, reversible, with risk analysis

## Development plan
See `PLAN_DESARROLLO.md` for the 8-phase plan. See `GUIA_PROYECTO_NUEVO.md` for step-by-step creation guide.

## Mandatory workflow (before any code change)
1. Explain understanding of the task
2. Identify affected files and dependencies
3. Analyze risks and side effects
4. Propose step-by-step implementation plan
5. Apply changes only after plan is aligned

## Release profile
Aggressive size optimization in workspace `Cargo.toml`: `strip = true`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `opt-level = "z"`

## Key dependencies
- Server: axum 0.7, tokio (full), sqlx 0.7 (postgres), argon2 0.5, aes-gcm 0.10, tower-http 0.6
- Viewer: dioxus 0.6 (desktop)
- Common: serde, chrono, sqlx, uuid, ed25519-dalek, sha2
- Workspace: tray-icon 0.14 (all features disabled to avoid GTK/Linux init)

## Testing
11 inline unit tests across 5 files. No integration tests. No test framework config.
```powershell
cargo test --workspace
```
