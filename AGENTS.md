# AGENTS.md — WorkshopManager

## Project status
**Version 0.1.0** — Phases 1-4 of audit complete. All critical/high items resolved. 83 tests passing, clippy clean. Domain: motorcycle workshop (products, sales, repairs, suppliers).

## Workspace structure
3 crates in a Cargo workspace:
- `crates/inventory-common` — shared types (Product, Sale, SaleItem, Repair, Supplier, User, AuditLog, Workshop), enums (PaymentMethod, RepairStatus, Priority, UserRole), DTOs, licensing (Ed25519 hardware-bound), money logic (IVA configurable), patent validation
- `crates/inventory-server` — Axum 0.7 backend on `:8443`, full DB (PostgreSQL embedded + 10 migrations), TLS, JWT auth, audit, backup, rate limiter, device keys, barcode generation
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
cargo test --workspace                     # tests (83 total)
```

## Version bump
```powershell
.\scripts\bump.ps1 0.2.0   # updates [workspace.package] version, builds, commits, tags
git push && git push --tags
```

## Domain types (inventory-common)
Structs: `Product`, `Sale`, `SaleItem`, `Repair`, `RepairUpdate`, `RepairPart`, `Supplier`, `User`, `Workshop`, `AuditLog`
Enums: `PaymentMethod` (Cash/Card/Transfer), `RepairStatus` (Pending/InProgress/Completed/Cancelled/Deleted), `Priority` (High/Medium/Low), `UserRole` (Admin/Mechanic/Seller)
DTOs: `LoginRequest`, `CreateProductRequest`, `CreateSaleRequest`, `CreateRepairRequest`, `CreateSupplierRequest`, `AddRepairPartRequest`, `ApiResponse<T>`, `PaginatedResponse<T>`, `LoginResponse`, `DashboardResponse`
License: `Feature` enum (16 features across 4 tiers), `License` struct with Ed25519 signature verification

## Server architecture
- Entrypoint: `crates/inventory-server/src/main.rs`
- Config: `config/server.toml` (TOML, host/port/api_key/require_device_key/tax.iva_rate)
- Secrets: generated/persisted at `dirs::data_local_dir()/WorkshopManager/data/` (.jwt_secret, .crypto_key, .postgres_password)
- Crypto: AES-256-GCM via `crypto::init()` with `OnceLock` (no `unsafe`)
- Auth: JWT HS256 (8h) + Argon2id (64MB, 3 iter, 4 parallelism); middleware `api_key` → `device_key` → `authenticate` → `require_admin`
- DB: PostgreSQL embebido via `postgresql_embedded` (`bundled` feature), pool en `AppState`
- Database name: `workshop_manager`
- Migrations: SQLx migrations in `crates/inventory-server/migrations/` (10 files)
- TLS: certificados autofirmados generados con `rcgen`, servidos por `axum-server` (`tls-rustls`)
- Security headers: HSTS, X-Content-Type-Options: nosniff, X-Frame-Options: DENY
- CORS: localhost only, GET/POST/PUT/DELETE, Authorization + Content-Type headers
- Body limit: 10MB via `RequestBodyLimitLayer`
- IVA: configurable via `[tax] iva_rate` in server.toml (default 0.19)
- Audit: `audit::log_change()` inserta en `audit_log` con redacción de credenciales
- Backup: scheduler automático cada 24h vía `pg_dump` + gzip, retención de 7 días
- Rate limiter: 5 attempts / 300s per email on login
- Device keys: full CRUD + UI page (admin only)
- Routes: `/api/auth/*` público (login/register) y protegido (status); `/api/*` protegido; `/api/users/*` y `/api/device-keys/*` admin
- Health: `GET /health` returns `"OK"`

## Viewer architecture
- Entrypoint: `crates/inventory-viewer/src/main.rs`
- CSS inlined at compile time via `include_str!("../index.css")` — **rebuild to see CSS changes**
- Routing: `dioxus-router` with 13 pages (Root, Login, Setup, Dashboard, Products, POS, Sales, Repairs, Suppliers, Reports, ServiceCertificate, Users, DeviceKeys)
- Components: Atomic Design — atoms (Button, Icon, Input, Spinner), molecules (Card, ConfirmModal, Modal), organisms (ServerSettings, DataTable, Header, UserFormModal, SaleDetailModal, RepairDetailModal, StockEntryModal, PartsTab, PartsTabCert)
- Theme system: light/dark tokens via TOML files in `assets/`
- API client: `api.rs` with typed methods for all server endpoints
- Card component: supports `header_action` prop for action buttons next to title
- ConfirmModal: supports `variant` prop (default Danger)

## Frontend Architecture Rules
- **CSS tokens:** Use CSS variables from `tokens-*.toml`. Never hardcode colors, sizes, or spacing.
- **Atomic Design:** atoms/ → molecules/ → organisms/. Extract ALL modals as organisms.
- **Table pattern:** Always `div.data-table-wrapper > table.data-table` for scrollable tables.
- **Modal pattern:** Modal molecule + footer with Ghost cancel + Primary action.
- **Error pattern:** All API errors must surface to user via alert. Never silently discard Results.
- **i18n:** Display translations go in `i18n.rs`. Enum Display traits stay in English (technical values for JWT/DB/serde).

## Constitutional rules (from CONSTITUCION.md)
All code MUST comply. Key rules:
- **No `unsafe`**. No `unwrap()`, `expect()`, or `panic!()` — use `Result<T, E>`
- **Parameterized SQL only** — never concatenate strings
- **Encrypt credentials** — use `crypto::encrypt_opt`/`decrypt_opt`
- **Validate all inputs** before DB insertion
- **Minimal changes** — localized, reversible, with risk analysis

## Mandatory workflow (before any code change)
1. Explain understanding of the task
2. Identify affected files and dependencies
3. Analyze risks and side effects
4. Propose step-by-step implementation plan
5. Apply changes only after plan is aligned
6. **Do NOT commit without explicit user permission**

## Release profile
Aggressive size optimization in workspace `Cargo.toml`: `strip = true`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `opt-level = "z"`

## Key dependencies
- Server: axum 0.7, tokio (full), sqlx 0.7 (postgres), argon2 0.5, aes-gcm 0.10, tower-http 0.6 (cors, limit, trace, set-header)
- Viewer: dioxus 0.6 (desktop)
- Common: serde, chrono, sqlx, uuid, sha2, rust_decimal, resvg
- Workspace: tray-icon 0.19 (all features disabled to avoid GTK/Linux init)

## Documentation
Full professional documentation in `docs/` (53 files):
- `docs/README.md` — documentation portal
- `docs/01-architecture/` — overview, system design (C4), data flow, tech stack
- `docs/02-api-reference/` — 9 endpoint docs (auth, products, sales, repairs, suppliers, reports, users, device-keys, common)
- `docs/03-database/` — schema, migrations, indexes, ER diagram
- `docs/04-security/` — overview, auth, authorization, encryption, audit, hardening
- `docs/05-deployment/` — prerequisites, installation, configuration, backup/recovery, troubleshooting
- `docs/06-user-guide/` — 9 module guides
- `docs/07-developer-guide/` — setup, project structure, conventions, testing, component library, contributing
- `docs/08-compliance/` — OWASP Top 10, data protection, coding standards
- `docs/09-operations/` — monitoring, incident response, maintenance
- `CHANGELOG.md`, `GLOSSARY.md`, `ACRONYMS.md`

## Audit status
See `AUDITORIA.md` for full tracking. Summary:
- **Critical (9/9):** All resolved
- **High (12/12):** All resolved
- **Medium:** 12/16 resolved (M-2, M-4, M-16 pending)
- **Low:** 5/10 resolved (B-1 to B-4, B-5, B-7 to B-10 pending)
- **Dependencies:** D-1 pending (sqlx upgrade), D-2/D-3 not applicable

## Known security improvements (backlog)
- **Secrets storage**: `.crypto_key`, `.jwt_secret`, `.postgres_password` stored as raw files. Mitigation: DPAPI/Keychain.
- **JWT refresh**: No refresh token mechanism (8h fixed TTL).
- **Sale cancellation**: No endpoint for annulling/returning sales.
- **Audit read**: No endpoint for reading audit logs.
