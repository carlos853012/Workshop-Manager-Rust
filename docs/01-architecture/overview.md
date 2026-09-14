# Architecture Overview — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-14
**Audience:** Architects, developers, technical stakeholders

---

## 1. System Purpose and Scope

WorkshopManager is a **desktop-native, integrated management system** designed for motorcycle workshops in the Chilean market. It provides end-to-end coverage of core business operations: inventory management, point-of-sale (POS), repair order tracking, supplier management, analytics, and reporting — all within a single, self-contained application requiring no external infrastructure beyond the host machine.

### Key Differentiators

| Aspect | WorkshopManager | Typical SaaS Alternatives |
|--------|-----------------|---------------------------|
| Deployment | Self-contained binary, zero-config | Cloud-hosted, requires internet |
| Database | Embedded PostgreSQL (bundled) | External database server |
| Networking | HTTPS with self-signed TLS 1.3 | Public HTTPS endpoint |
| Data ownership | 100% local, no third-party access | Third-party cloud provider |
| Offline operation | Full functionality offline | Requires constant connectivity |
| Cost model | One-time license (DLC tiers) | Recurring subscription |
| Market focus | Chilean tax (IVA 19%), CLP, patentes | Generic international |

### Scope Boundaries

**In scope:**
- Product catalog and inventory (CRUD, stock tracking, barcode generation)
- Point of Sale with atomic stock deduction and IVA calculation
- Repair order lifecycle (create → in-progress → completed/cancelled)
- Supplier directory
- Analytics dashboard and KPIs
- Client reports, service certificates (PDF generation)
- User management with RBAC (Admin, Mechanic, Seller)
- Encrypted audit trail
- Automatic encrypted backups
- Device key-based machine binding
- License system with hardware-bound Ed25519 signatures

**Out of scope:**
- Multi-user concurrent editing from multiple machines (single-workshop model)
- Web/mobile clients (desktop only)
- Multi-currency (CLP only)
- Electronic invoicing (DTE/SII integration)
- Warehouse management across physical locations

---

## 2. Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           HOST MACHINE (Windows/macOS/Linux)                │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                    workshop-viewer (Dioxus 0.6 Desktop)            │   │
│  │                                                                      │   │
│  │  ┌─────────┐ ┌──────────┐ ┌─────────┐ ┌──────────┐ ┌───────────┐  │   │
│  │  │  Pages   │ │Components│ │  Theme  │ │   API    │ │  Router   │  │   │
│  │  │ (13)    │ │ (Atomic) │ │ (TOML)  │ │  Client  │ │(dioxus-   │  │   │
│  │  │         │ │ atoms/   │ │ light/  │ │ (reqwest)│ │ router)   │  │   │
│  │  │         │ │ molecules│ │ dark    │ │          │ │           │  │   │
│  │  │         │ │ organisms│ │         │ │          │ │           │  │   │
│  │  └─────────┘ └──────────┘ └─────────┘ └────┬─────┘ └───────────┘  │   │
│  │                                              │                      │   │
│  └──────────────────────────────────────────────┼──────────────────────┘   │
│                                                 │                           │
│                                            HTTPS │ (TLS 1.3)               │
│                                            :8443 │                          │
│                                                 │                           │
│  ┌──────────────────────────────────────────────┼──────────────────────┐   │
│  │                workshop-server (Axum 0.7)    │                      │   │
│  │                                              ▼                      │   │
│  │  ┌────────────────────────────────────────────────────────────────┐ │   │
│  │  │                      MIDDLEWARE STACK                          │ │   │
│  │  │  CorsLayer → HSTS → X-Frame-Options → RequestBodyLimit (10MB) │ │   │
│  │  │  → TraceLayer → api_key → device_key → JWT auth → admin check │ │   │
│  │  └────────────────────────────────────────────────────────────────┘ │   │
│  │                                                                    │   │
│  │  ┌────────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │   │
│  │  │   Routes    │ │  Auth    │ │  Crypto  │ │  Audit   │           │   │
│  │  │ public/     │ │ JWT HS256│ │ AES-256- │ │ log_change│           │   │
│  │  │ protected/  │ │ Argon2id │ │ GCM      │ │ redact   │           │   │
│  │  │ admin/      │ │          │ │          │ │          │           │   │
│  │  └────────────┘ └──────────┘ └──────────┘ └──────────┘           │   │
│  │  ┌────────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │   │
│  │  │  Rate      │ │  Backup  │ │  Device  │ │  Config  │           │   │
│  │  │  Limiter   │ │ pg_dump  │ │  Keys    │ │ server.  │           │   │
│  │  │  (login)   │ │ + gzip   │ │ SHA-256  │ │ toml     │           │   │
│  │  │            │ │ + encrypt│ │          │ │          │           │   │
│  │  └────────────┘ └──────────┘ └──────────┘ └──────────┘           │   │
│  │                                                                    │   │
│  │  ┌────────────────────────────────────────────────────────────┐    │   │
│  │  │           AppState (shared via Axum FromRef)               │    │   │
│  │  │  Secrets (jwt_secret, crypto_key)                         │    │   │
│  │  │  ServerConfig (host, port, api_key, require_device_key)   │    │   │
│  │  │  PgPool (sqlx connection pool)                            │    │   │
│  │  │  RateLimiter (Arc<RwLock<HashMap>>)                       │    │   │
│  │  └────────────────────────────────────────────────────────────┘    │   │
│  └──────────────────────────────┬─────────────────────────────────────┘   │
│                                 │                                          │
│                                 ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                 PostgreSQL (embedded via postgresql_embedded)         │  │
│  │                                                                      │  │
│  │  Tables: workshops, users, products, sales, sale_items,             │  │
│  │          repairs, repair_updates, repair_parts, suppliers,          │  │
│  │          audit_log, device_keys                                      │  │
│  │                                                                      │  │
│  │  Enums: payment_method, repair_status, priority, user_role          │  │
│  │                                                                      │  │
│  │  Data isolation: ALL queries filtered by workshop_id                │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  DATA DIRECTORY: $LOCALAPPDATA/WorkshopManager/data/                │  │
│  │                                                                      │  │
│  │  .crypto_key          AES-256-GCM encryption key (32 bytes)        │  │
│  │  .jwt_secret          JWT signing secret (64 hex chars)             │  │
│  │  backups/             Encrypted .sql.gz.enc files (7-day retention) │  │
│  │  *.pem                Self-signed TLS certificates                  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Component Descriptions

### 3.1 workshop-common (Shared Types Library)

**Purpose:** Single source of truth for domain types, DTOs, enums, and business logic shared between server and viewer.

| Module | Responsibility |
|--------|---------------|
| `lib.rs` | Core domain structs (`Product`, `Sale`, `Repair`, `Supplier`, `User`, `Workshop`, `AuditLog`) and enums (`PaymentMethod`, `RepairStatus`, `Priority`, `UserRole`) |
| `dto.rs` | Request/response DTOs (`LoginRequest`, `CreateSaleRequest`, `ApiResponse<T>`, `PaginatedResponse<T>`, `DashboardResponse`, `ServiceCertificate`) |
| `money.rs` | Chilean CLP monetary logic: IVA 19% calculation, price extraction (base + IVA), CLP rounding to nearest $10, CLP formatting with thousand separators |
| `patente.rs` | Chilean license plate (PPU) validation: Antigua (LLnnnn), Nueva (LLLLnn), Moto (LLLnn), Policia (Lnnnn) |
| `license.rs` | License system: Ed25519 signature creation/verification, hardware hash validation, license file I/O |
| `features.rs` | Feature flag system with 5 license tiers: Trial, Base, Reports, Advanced, API (17 features) |
| `hardware.rs` | Hardware fingerprint extraction (CPU + Motherboard + Disk → SHA-256) |
| `icon_data.rs` | Procedurally generated application icon (wrench) |

### 3.2 workshop-server (Backend)

**Purpose:** HTTPS API server providing all business logic, data persistence, and security services.

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Application bootstrap: crypto init → secrets → embedded PostgreSQL → migrations → backup scheduler → TLS server → graceful shutdown |
| `state.rs` | `AppState` definition with `FromRef` impls for Axum extractors (`Secrets`, `ServerConfig`, `PgPool`, `RateLimiter`) |
| `auth.rs` | JWT token creation/validation (HS256), Argon2id password hashing (64MB, 3 iterations, 4 parallelism) |
| `middleware.rs` | Three-layer middleware: API key check → device key check → JWT authentication → admin role enforcement |
| `crypto.rs` | AES-256-GCM encryption/decryption with random nonces, `OnceLock` cipher initialization |
| `error.rs` | `AppError` enum mapping domain errors to HTTP status codes, DB error sanitization |
| `audit.rs` | Audit trail logging with sensitive field redaction (`password_hash`, `crypto_key`, `jwt_secret`) |
| `backup.rs` | Automated backup: `pg_dump` → gzip compression → AES-256-GCM encryption, 7-day retention |
| `rate_limiter.rs` | In-memory rate limiter using `RwLock<HashMap>` with sliding window |
| `device_key.rs` | Device binding: SHA-256 hashed keys, middleware validation, CRUD operations |
| `schema.rs` | SQLx migrations runner with barcode_prefix column ensure |
| `secrets.rs` | Secret initialization: load or generate JWT secret and crypto key with OS permission hardening |
| `config.rs` | TOML configuration loader from `config/server.toml` |
| `routes/` | Axum route handlers organized by domain (auth, products, sales, repairs, suppliers, analytics, reports, users, device_keys) |
| `certificate.rs` | PDF service certificate generation |
| `barcode.rs` | EAN-13 barcode image generation |
| `tls.rs` | Self-signed TLS certificate generation via `rcgen`, rustls configuration |
| `db_manager.rs` | Embedded PostgreSQL lifecycle management via `postgresql_embedded` |

### 3.3 workshop-viewer (Desktop Client)

**Purpose:** Native desktop UI built with Dioxus 0.6, providing the user interface for all operations.

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Application entry point: window configuration (1500×900), icon generation, provider stack |
| `api.rs` | Typed HTTP client wrapping `reqwest` with automatic API key, device key, and JWT header injection |
| `routes.rs` | Client-side routing with 13 pages via `dioxus-router` |
| `app_state.rs` | Global state providers: `AuthProvider` (JWT token, user session), `TabsProvider` |
| `theme.rs` | Light/dark theme system with CSS variable tokens from TOML files |
| `i18n.rs` | UI string translations (Spanish/English) |
| `icons.rs` | SVG icon definitions for the UI |
| `config.rs` | Client configuration loader from `config/viewer.toml` |
| `pages/` | Page components: Root, Setup, Login, Dashboard, Products, POS, Sales, Repairs, Suppliers, Reports, ServiceCertificate, Users, DeviceKeys |
| `components/` | Atomic Design components: atoms (Badge, Button, Icon, Input, Spinner), molecules (Card, ConfirmModal, FormGroup, Modal, Tooltip), organisms (ConnectionSettings, DataTable, Header) |

---

## 4. Key Design Decisions and Rationale

### D1: Embedded PostgreSQL over SQLite

**Decision:** Bundle PostgreSQL via `postgresql_embedded` instead of using SQLite.

**Rationale:**
- Full ACID compliance with row-level locking (`SELECT ... FOR UPDATE`)
- Native JSONB support for audit trail (`old_values`, `new_values`)
- `INET` type for IP addresses in audit log
- Enum types (`payment_method`, `repair_status`, `priority`, `user_role`) enforced at DB level
- Better concurrency model for future multi-viewer support
- Familiar tooling for debugging (`psql`, `pg_dump`)

**Trade-off:** Larger binary size (~50MB PostgreSQL binaries), higher memory usage (~64MB baseline), but acceptable for desktop deployment.

### D2: Desktop-First with Embedded Server

**Decision:** Run the API server on localhost as part of the application, not as a separate deployment.

**Rationale:**
- Zero infrastructure requirements for small workshops
- No network exposure (server binds to `127.0.0.1`)
- TLS is still valuable for encrypting data at rest on disk
- API key mechanism ensures only the authorized viewer can connect
- Enables future multi-viewer support (multiple desktop instances)

### D3: AES-256-GCM Encryption for Data at Rest

**Decision:** Encrypt database credentials and backups using AES-256-GCM.

**Rationale:**
- Provides authenticated encryption (confidentiality + integrity)
- Random 12-byte nonces prevent nonce reuse attacks
- Base64-encoded output for safe storage/transmission
- Used for: PostgreSQL password, backup files, sensitive config values

### D4: Argon2id for Password Hashing

**Decision:** Use Argon2id with 64MB memory, 3 iterations, 4 parallelism.

**Rationale:**
- OWASP-recommended algorithm for password hashing
- Memory-hard design resists GPU/ASIC attacks
- Parameters exceed OWASP minimum recommendations
- Salt generated via `OsRng` (CSPRNG)

### D5: License System with Hardware Binding

**Decision:** Implement Ed25519-signed licenses with hardware fingerprinting and tiered feature gating.

**Rationale:**
- 5 license tiers (Trial → Base → Reports → Advanced → API) with 17 features
- Hardware binding (CPU + MB + Disk SHA-256) prevents license sharing
- Ed25519 signatures are fast, small keys (32 bytes), and cryptographically secure
- Max viewers per tier controls concurrent connections
- Max transfers limits migration abuse
- Online validation on first activation prevents multi-PC use
- Trial mode enables immediate use without activation
- `license-tool` CLI for vendor license generation

**Trade-off:** Requires vendor involvement for license generation; offline piracy detection is limited.

### D6: Atomic Design for UI Components

**Decision:** Organize viewer components following Atomic Design methodology.

**Rationale:**
- Clear hierarchy: atoms → molecules → organisms → pages
- Promotes component reuse and consistency
- Theme tokens (CSS variables) ensure visual consistency
- Separation of concerns: presentation vs. logic

---

## 5. Technology Choices with Justification

### 5.1 Language: Rust

| Criterion | Assessment |
|-----------|-----------|
| Performance | Zero-cost abstractions, no GC pauses, ideal for real-time POS |
| Memory Safety | Ownership system prevents data races, null pointers, buffer overflows |
| Ecosystem | Mature crates for HTTP (axum), DB (sqlx), crypto (aes-gcm, argon2), desktop (dioxus) |
| Deployment | Single binary distribution, no runtime dependencies |
| Compile-time Safety | SQLx compile-time query checking, exhaustive pattern matching |

### 5.2 Backend: Axum 0.7

| Criterion | Assessment |
|-----------|-----------|
| Ergonomics | Tower middleware integration, extractor pattern reduces boilerplate |
| Performance | Built on tokio + hyper, competitive with actix-web |
| Type Safety | `FromRef<AppState>` enables clean state extraction |
| Ecosystem | Active development, strong community, compatible with tower ecosystem |

### 5.3 Frontend: Dioxus 0.6 Desktop

| Criterion | Assessment |
|-----------|-----------|
| Native Performance | Renders via system WebView (no Electron bloat) |
| Rust-native | Shared types with server via `workshop-common` |
| Desktop-first | Full OS integration (tray icon, window management) |
| Component Model | RSX syntax similar to JSX, familiar to web developers |

### 5.4 Database: PostgreSQL (Embedded)

| Criterion | Assessment |
|-----------|-----------|
| ACID | Full transaction support with row-level locking |
| Types | Native enums, JSONB, UUID, INET, NUMERIC(19,4) |
| Migrations | SQLx migrate with compile-time embedding |
| Tooling | `pg_dump` for backup, `psql` for debugging |
| Embedded | `postgresql_embedded` crate bundles server + data directory |

### 5.5 Security Libraries

| Library | Purpose | Choice Rationale |
|---------|---------|-----------------|
| `argon2 0.5` | Password hashing | OWASP-recommended, memory-hard |
| `aes-gcm 0.10` | Symmetric encryption | Authenticated encryption, NIST-approved |
| `jsonwebtoken 9` | JWT creation/validation | De facto standard, HS256 support |
| `rcgen 0.13` | TLS certificate generation | Pure Rust, no OpenSSL dependency |
| `ed25519-dalek 2.1` | License signatures | Fast, small keys, EdDSA standard |
| `sha2 0.10` | Hashing | SHA-256 for hardware fingerprinting, device keys |
| `rand 0.8` | CSPRNG | `OsRng` for cryptographic randomness |

---

## 6. Quality Attributes

### 6.1 Security

| Control | Implementation | Status |
|---------|---------------|--------|
| Transport encryption | TLS 1.3 via rustls + self-signed certs | Implemented |
| Password hashing | Argon2id (64MB, 3 iter, 4 parallelism) | Implemented |
| Token authentication | JWT HS256, 8-hour expiry | Implemented |
| Data at rest | AES-256-GCM for credentials and backups | Implemented |
| API key | Shared secret between viewer and server | Implemented |
| Device binding | SHA-256 hashed device keys, middleware check | Implemented |
| Input validation | Request-level validation before DB insert | Implemented |
| SQL injection prevention | Parameterized queries only (sqlx) | Implemented |
| Error sanitization | DB errors mapped to generic messages | Implemented |
| Audit trail | All mutations logged with old/new values | Implemented |
| Rate limiting | Login endpoint: 5 attempts per 5 minutes | Implemented |
| Security headers | HSTS, X-Content-Type-Options, X-Frame-Options | Implemented |
| CORS restriction | Whitelist of localhost origins | Implemented |
| Body size limit | 10MB maximum request body | Implemented |

### 6.2 Performance

| Metric | Target | Current |
|--------|--------|---------|
| POS transaction latency | < 200ms | ~50ms (local PostgreSQL) |
| Product lookup (barcode) | < 100ms | ~20ms (indexed query) |
| Dashboard load | < 500ms | ~150ms (aggregate queries) |
| Startup time | < 5s | ~3s (including PostgreSQL init) |
| Memory footprint | < 200MB | ~120MB (server + PostgreSQL) |
| Binary size | < 100MB | ~80MB (with release optimizations) |

### 6.3 Scalability

| Dimension | Current | Limit |
|-----------|---------|-------|
| Concurrent viewers | Per license tier | Trial: 1, Base: 2, Reports: 5, Advanced: 10, API: 999 |
| Product catalog | Unlimited | Limited by disk space |
| Transaction history | Unlimited | Limited by disk space |
| Multi-workshop | Single workshop | Data isolation ready for multi-tenant |
| Database size | PostgreSQL limits | ~32TB practical limit |

### 6.4 Maintainability

| Aspect | Approach |
|--------|----------|
| Code organization | 4-crate workspace (common, server, viewer, license-tool) |
| Type safety | Shared domain types across server and viewer |
| Testing | 92 unit tests across 4 crates |
| Linting | `cargo clippy --workspace -- -D warnings` |
| Formatting | `cargo fmt --all --check` |
| Versioning | Semantic versioning with `scripts/bump.ps1` |
| Documentation | Architecture docs, AGENTS.md, CONSTITUCION.md |

### 6.5 Reliability

| Aspect | Approach |
|--------|----------|
| Data integrity | Database transactions with `BEGIN`/`COMMIT`/`ROLLBACK` |
| Stock deduction | `SELECT ... FOR UPDATE` with sorted lock ordering to prevent deadlocks |
| Backup | Automatic daily encrypted backups, 7-day retention |
| Graceful shutdown | 10-second timeout, PostgreSQL stop on exit |
| Error handling | `Result<T, E>` throughout, no `unwrap()` or `panic!()` in production code |
| Audit trail | Best-effort logging (audit failures don't block operations) |

---

## 7. Constraints and Assumptions

### 7.1 Constraints

| ID | Constraint | Impact |
|----|-----------|--------|
| C1 | Desktop-only deployment (no web/mobile) | Limits remote access scenarios |
| C2 | Single-workshop per installation | No cross-workshop analytics |
| C3 | Chilean market only (CLP, IVA 19%, patentes chilenas) | No internationalization of business logic |
| C4 | No internet required at runtime | Limits online features (e.g., DTE integration) |
| C5 | Windows primary target | macOS/Linux support secondary |
| C6 | Proprietary license | No open-source distribution |
| C7 | No `unsafe` code (CONSTITUCION.md) | May limit some low-level optimizations |

### 7.2 Assumptions

| ID | Assumption | Risk |
|----|-----------|------|
| A1 | Single user operates the application at a time | Concurrent access not tested |
| A2 | Host machine has sufficient disk space (>1GB) | PostgreSQL + backups need space |
| A3 | Host machine has sufficient RAM (>2GB) | PostgreSQL + server need memory |
| A4 | Workshop owner has basic technical literacy | Setup wizard guides initial config |
| A5 | Chilean tax law remains at IVA 19% | Rate is configurable in `money.rs` |
| A6 | Self-signed TLS is acceptable | No CA certificate needed |

---

## 8. Cross-References

| Document | Description |
|----------|-------------|
| [System Design](./system-design.md) | C4 diagrams, data flows, integration patterns |
| [Data Flow](./data-flow.md) | Request lifecycle, authentication flow, POS transaction flow |
| [Technology Stack](./technology-stack.md) | Dependencies table, library rationale, version matrix |
| `AGENTS.md` | Project status, build commands, known issues |
| `CONSTITUCION.md` | Constitutional rules (no unsafe, parameterized SQL, input validation) |
| `PLAN_DESARROLLO.md` | 8-phase development plan |
| `GUIA_PROYECTO_NUEVO.md` | Step-by-step project creation guide |
