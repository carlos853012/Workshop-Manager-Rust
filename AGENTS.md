# AGENTS.md — WorkshopManager

## Project status
**Phase 3 of 8 complete** (`PLAN_DESARROLLO.md`). Server bootstrap done: DB, migrations, JWT auth, admin middleware, TLS, audit, backup. Version `0.1.0`. Domain: motorcycle workshop (products, sales, repairs, suppliers).

## Workspace structure
3 crates in a Cargo workspace:
- `crates/inventory-common` — shared types (Product, Sale, SaleItem, Repair, Supplier, User, AuditLog), enums, DTOs, licensing (Ed25519 hardware-bound), money logic, patent validation
- `crates/inventory-server` — Axum 0.7 backend on `:8443`, full DB (PostgreSQL embedded + migrations), TLS, JWT auth, audit, backup, rate limiter, device keys, barcode generation
- `crates/inventory-viewer` — Dioxus 0.6 Desktop, 13 pages, routing, Atomic Design components (atoms/molecules/organisms), theme system, API client

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

## Domain types (inventory-common)
Structs: `Product`, `Sale`, `SaleItem`, `Repair`, `RepairUpdate`, `Supplier`, `User`, `AuditLog`
Enums: `PaymentMethod` (Cash/Card/Transfer), `RepairStatus` (Pending/InProgress/Completed/Cancelled), `Priority` (High/Medium/Low), `UserRole` (Admin/Mechanic/Seller)
DTOs: `LoginRequest`, `CreateProductRequest`, `CreateSaleRequest`, `CreateRepairRequest`, `ApiResponse<T>`, `PaginatedResponse<T>`, `LoginResponse`, `DashboardResponse`
License: `Feature` enum (15 features across 4 tiers), `License` struct with Ed25519 signature verification

## Server architecture
- Entrypoint: `crates/inventory-server/src/main.rs`
- Config: `config/server.toml` (TOML, host/port/api_key/require_device_key)
- Secrets: generated/persisted at `dirs::data_local_dir()/WorkshopManager/data/`
- Crypto: AES-256-GCM via `crypto::init()` with `OnceLock` (no `unsafe`)
- Auth: JWT HS256 + Argon2id; middleware `require_auth` y `require_admin`
- DB: PostgreSQL embebido via `postgresql_embedded` (`bundled` feature), pool en `AppState`
- Database name: `workshop_manager`
- Migrations: SQLx migrations in `crates/inventory-server/migrations/` (8 files)
- TLS: certificados autofirmados generados con `rcgen`, servidos por `axum-server` (`tls-rustls`)
- Audit: `audit::log_change()` inserta en `audit_log` con redacción de credenciales
- Backup: scheduler automático cada 24h vía `pg_dump` + gzip, retención de 7 días
- Rate limiter: connected and active on login
- Device keys: full CRUD + UI page
- Routes: `/api/auth/*` público (login/register) y protegido (status); resto de `/api/*` protegido; `/api/users/*` admin
- Health: `GET /health` returns `"OK"`

## Viewer architecture
- Entrypoint: `crates/inventory-viewer/src/main.rs`
- CSS inlined at compile time via `include_str!("../index.css")` — **rebuild to see CSS changes**
- Routing: `dioxus-router` with pages for Home, Login, Setup, Products, Sales, POS, Repairs, Suppliers, Reports, Users, DeviceKeys
- Components: Atomic Design — atoms (Badge, Button, Icon, Input, Spinner), molecules (Card, ConfirmModal, FormGroup, Modal, Tooltip), organisms (ConnectionSettings, DataTable, Header)
- Theme system: light/dark tokens via TOML files in `assets/`
- API client: `api.rs` with typed methods for all server endpoints
- 13 pages total in `pages/`

## Frontend Architecture Rules
- **CSS tokens:** Use CSS variables from `tokens-*.toml`. Never hardcode colors, sizes, or spacing.
- **Atomic Design:** atoms/ → molecules/ → organisms/. Extract ALL modals as organisms.
- **Table pattern:** Always `div.data-table-wrapper > table.data-table` for scrollable tables.
- **Modal pattern:** Modal molecule + footer with Ghost cancel + Primary action.
- **Error pattern:** All API errors must surface to user via alert. Never silently discard Results.
- **i18n:** Display translations go in `i18n.rs`. Enum Display traits stay in English (technical values for JWT/DB/serde).

## Known issues
- **Server unused deps:** `tracing-appender`, `obfstr` in Cargo.toml but never imported
- **`.clinerules`/`.cursorrules`/`.geminirules`:** links and OT/SCADA references fixed
- **`scripts/bump.ps1`:** works correctly (updates workspace version, builds, commits, tags)

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

## Known security improvements (backlog)
- **Secrets storage**: `.crypto_key`, `.jwt_secret`, `.postgres_password` stored as raw files in `$LOCALAPPDATA/WorkshopManager/data/`. Anyone with filesystem access can decrypt. Mitigation: encrypt `.crypto_key` with Windows DPAPI / macOS Keychain, or use OS-level secret store.
- **DB password**: encrypted with AES-256-GCM but key is co-located. Should migrate to env var or OS keychain.
- **JWT secret**: stored in plaintext on disk. Should be volatile (regenerated on startup) or OS-protected.

## Testing
23 inline unit tests across 9 files. No integration tests yet.
```powershell
cargo test --workspace
```

## Security Checklist (before production)
- [ ] Argon2id params hardened (64MB+)
- [ ] API key auto-generated (not default)
- [ ] Body size limit configured (10MB)
- [ ] CORS restricted to needed methods/headers
- [ ] Security headers set (HSTS, X-Content-Type-Options, X-Frame-Options)
- [ ] Rate limiting on all write endpoints
- [ ] device_keys scoped to workshop_id
- [ ] All secrets use DPAPI/Keychain (not plain files)
- [ ] Token revocation mechanism in place
- [ ] Email validation with regex
