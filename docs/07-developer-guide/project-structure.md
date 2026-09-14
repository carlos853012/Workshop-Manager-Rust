# Project Structure

This document describes the codebase layout, module organization, and crate responsibilities.

---

## Workspace Layout

```
workshop-manager/
├── Cargo.toml                    # Workspace root (members, shared config)
├── Cargo.lock                    # Dependency lock file
├── config/
│   ├── server.toml               # Server configuration
│   └── viewer.toml               # Viewer configuration
├── crates/
│   ├── workshop-common/         # Shared types, DTOs, enums, licensing
│   ├── workshop-server/         # Axum backend, DB, auth, TLS
│   ├── workshop-viewer/         # Dioxus Desktop frontend
│   └── license-tool/            # CLI for license generation
├── docs/                         # Documentation
├── scripts/
│   ├── bump.ps1                  # Version bump automation
│   └── restore.ps1               # Backup restore script
├── tools/                        # Dev utilities
├── AGENTS.md                     # AI assistant instructions
├── CONSTITUCION.md               # Coding rules and principles
├── PLAN_DESARROLLO.md            # Development roadmap
└── README.md                     # Project overview
```

---

## Crate Responsibilities

### workshop-common

**Shared foundation** used by both server and viewer.

```
crates/workshop-common/src/
├── lib.rs          # Module declarations, re-exports
├── dto.rs          # Request/Response DTOs (358 lines)
├── features.rs     # Feature flags and licensing tiers (17 features, 5 tiers)
├── hardware.rs     # Hardware fingerprint extraction (CPU + MB + Disk)
├── icon_data.rs    # Icon SVG data
├── license.rs      # Ed25519 signing/verification, license creation
├── money.rs        # Monetary calculations (Decimal-based)
└── patente.rs      # Vehicle patent/plate validation
```

**Key Types:**
- Domain structs: `Product`, `Sale`, `SaleItem`, `Repair`, `Supplier`, `User`, `AuditLog`
- Enums: `PaymentMethod`, `RepairStatus`, `Priority`, `UserRole`
- DTOs: `LoginRequest`, `CreateProductRequest`, `CreateSaleRequest`, `ApiResponse<T>`, `PaginatedResponse<T>`
- License: `Feature` enum (17 features), `LicenseTier` enum (5 tiers), `License` struct with Ed25519 signature verification

### workshop-server

**Backend API server** with embedded database.

```
crates/workshop-server/src/
├── main.rs         # Entrypoint, server bootstrap
├── state.rs        # AppState (DB pool, config, secrets)
├── config.rs       # TOML configuration loading
├── auth.rs         # JWT generation, Argon2id hashing
├── middleware.rs   # require_auth, require_admin
├── error.rs        # AppError enum, IntoResponse
├── crypto.rs       # AES-256-GCM encryption/decryption
├── secrets.rs      # Secret generation and persistence
├── tls.rs          # Self-signed certificate generation
├── audit.rs        # Audit logging
├── backup.rs       # Automated backup scheduler
├── rate_limiter.rs # Request rate limiting
├── device_key.rs   # Device key management
├── barcode.rs      # Barcode generation
├── certificate.rs # Service certificate generation
├── schema.rs       # DB schema types
├── db_manager.rs   # Database lifecycle management
├── license.rs      # License loading, validation, online activation
├── tray.rs         # System tray icon (API key copy, license status)
└── routes/
    ├── mod.rs      # Route registration
    ├── auth.rs     # POST /api/auth/login, /register
    ├── products.rs # CRUD /api/products
    ├── sales.rs    # CRUD /api/sales
    ├── repairs.rs  # CRUD /api/repairs
    ├── suppliers.rs# CRUD /api/suppliers
    ├── users.rs    # CRUD /api/users (admin)
    ├── device_keys.rs # CRUD /api/device-keys
    ├── audit.rs     # GET /api/audit (admin)
    ├── analytics.rs   # Dashboard analytics
    ├── reports.rs     # Report generation
    └── pagination.rs  # Shared pagination logic
```

### workshop-viewer

**Dioxus Desktop frontend** with Atomic Design components.

```
crates/workshop-viewer/src/
├── main.rs         # Entrypoint, Dioxus app launch
├── routes.rs       # Route enum (dioxus-router)
├── app_state.rs    # Global app state
├── api.rs          # API client (all server endpoints)
├── config.rs       # Viewer configuration
├── i18n.rs         # UI translations (Spanish)
├── icons.rs        # Icon imports
├── icons/          # SVG icon data files
├── layout.rs       # Page layout wrapper
├── theme/
│   ├── mod.rs      # Theme loading
│   └── tokens.rs   # CSS token parsing
├── components/
│   ├── mod.rs      # Module declarations
│   ├── atoms/      # Basic UI primitives
│   ├── molecules/  # Composed components
│   └── organisms/  # Complex UI sections
└── pages/          # Route page components
```

---

## Module Organization

### Server Routes Pattern

Each route module follows this structure:

```rust
// routes/products.rs
pub async fn list_products(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 1. Parse query params / body
    // 2. Validate input
    // 3. Execute parameterized SQL
    // 4. Map to response
    // 5. Return Result<Json<T>, AppError>
}
```

### Component Pattern (Viewer)

Each component file exports a single function component:

```rust
#[component]
pub fn MyComponent(props: MyComponentProps) -> Element {
    rsx! {
        // JSX-like markup
    }
}
```

---

## Dependency Graph

```
workshop-viewer ──depends on──► workshop-common
       │
       └── (HTTP calls to server at runtime)

workshop-server ──depends on──► workshop-common
       │
       └── (Direct DB access, no viewer dependency)
```

**Rule**: `workshop-common` never depends on server or viewer. It is a pure data/types crate.

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace members, shared dependencies, release profile |
| `config/server.toml` | Host, port, API key, TLS, device key settings |
| `CONSTITUCION.md` | Coding rules, security requirements, workflow |
| `AGENTS.md` | AI assistant context and commands |
| `PLAN_DESARROLLO.md` | 8-phase development roadmap |
| `scripts/bump.ps1` | Automated version bumping |

---

## Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Crate names | `inventory-{role}` | `workshop-server` |
| Module files | `snake_case.rs` | `device_key.rs` |
| Structs | `PascalCase` | `CreateProductRequest` |
| Functions | `snake_case` | `require_auth()` |
| Constants | `SCREAMING_SNAKE` | `MAX_PAGE_SIZE` |
| CSS classes | `kebab-case` | `data-table-wrapper` |
| Routes | `/kebab-case` | `/service-certificate` |
