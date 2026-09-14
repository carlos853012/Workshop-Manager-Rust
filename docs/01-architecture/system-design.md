# System Design — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Architects, senior developers

---

## 1. C4 Model — Level 1: System Context

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        WORKSHOP MANAGER SYSTEM                          │
│                                                                         │
│  Desktop application for motorcycle workshop management.                │
│  Handles inventory, sales, repairs, suppliers, and reporting.           │
│  Runs entirely on the host machine with embedded PostgreSQL.            │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │                       │
                    ▼                       ▼
    ┌───────────────────────┐   ┌───────────────────────┐
    │     Workshop Owner    │   │   Chilean Tax System   │
    │     (End User)        │   │   (SII / IVA 19%)     │
    │                       │   │                       │
    │  - Operates POS       │   │  - IVA calculation    │
    │  - Manages inventory  │   │  - CLP currency       │
    │  - Tracks repairs     │   │  - Patente validation │
    │  - Views reports      │   │                       │
    └───────────────────────┘   └───────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │                       │
                    ▼                       ▼
    ┌───────────────────────┐   ┌───────────────────────┐
    │    Backup Storage     │   │    Hardware Layer      │
    │    (Local Disk)       │   │    (CPU/MB/Disk)      │
    │                       │   │                       │
    │  - Encrypted backups  │   │  - Hardware hash      │
    │  - 7-day retention    │   │    for license binding │
    │  - Auto-pruned        │   │                       │
    └───────────────────────┘   └───────────────────────┘
```

### Actors

| Actor | Description | Interaction |
|-------|-------------|-------------|
| Workshop Owner | Primary user, manages all operations | Desktop UI via workshop-viewer |
| Workshop Mechanic | Creates repairs, views inventory | Desktop UI (limited role) |
| Workshop Seller | Processes sales, looks up products | Desktop UI (limited role) |
| System Admin | Manages users, device keys | Desktop UI (admin role only) |

---

## 2. C4 Model — Level 2: Container Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        WORKSHOP MANAGER SYSTEM                          │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                                                                  │   │
│  │   CONTAINER 1: workshop-viewer                                  │   │
│  │   Technology: Dioxus 0.6 Desktop (Rust)                         │   │
│  │                                                                  │   │
│  │   - 13 page components (Router)                                  │   │
│  │   - Atomic Design UI (atoms/molecules/organisms)                 │   │
│  │   - API client (reqwest with TLS)                                │   │
│  │   - Theme system (light/dark via TOML tokens)                    │   │
│  │   - Auth state management (JWT token, user session)              │   │
│  │                                                                  │   │
│  │   Port: N/A (desktop app)                                        │   │
│  └──────────────────────────┬───────────────────────────────────────┘   │
│                              │                                           │
│                         HTTPS │ (TLS 1.3, port 8443)                    │
│                         JSON  │ (REST API)                              │
│                              │                                           │
│  ┌──────────────────────────▼───────────────────────────────────────┐   │
│  │                                                                  │   │
│  │   CONTAINER 2: workshop-server                                  │   │
│  │   Technology: Axum 0.7 (Rust)                                    │   │
│  │                                                                  │   │
│  │   - REST API (public, protected, admin routes)                   │   │
│  │   - JWT authentication + Argon2id password hashing               │   │
│  │   - AES-256-GCM encryption for data at rest                     │   │
│  │   - Audit trail logging                                          │   │
│  │   - Automated encrypted backups                                  │   │
│  │   - Rate limiting (login endpoint)                               │   │
│  │   - Device key binding                                           │   │
│  │                                                                  │   │
│  │   Port: 8443 (HTTPS)                                             │   │
│  └──────────────────────────┬───────────────────────────────────────┘   │
│                              │                                           │
│                         SQL   │ (sqlx 0.7, connection pool)             │
│                              │                                           │
│  ┌──────────────────────────▼───────────────────────────────────────┐   │
│  │                                                                  │   │
│  │   CONTAINER 3: PostgreSQL (Embedded)                             │   │
│  │   Technology: postgresql_embedded 0.20                           │   │
│  │                                                                  │   │
│  │   - 11 tables (workshops, users, products, sales, sale_items,   │   │
│  │     repairs, repair_updates, repair_parts, suppliers,            │   │
│  │     audit_log, device_keys)                                      │   │
│  │   - 4 custom enums (payment_method, repair_status, priority,    │   │
│  │     user_role)                                                   │   │
│  │   - Row-level locking for stock deduction                        │   │
│  │   - JSONB for audit trail values                                 │   │
│  │   - Parameterized queries (SQLx compile-time checked)            │   │
│  │                                                                  │   │
│  │   Database: workshop_manager                                     │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │   SHARED LIBRARY: workshop-common                               │   │
│  │   Used by: workshop-server, workshop-viewer                    │   │
│  │                                                                  │   │
│  │   - Domain types (Product, Sale, Repair, Supplier, User)        │   │
│  │   - DTOs (request/response structs)                              │   │
│  │   - Business logic (IVA, patente validation, money formatting)   │   │
│  │   - License system (Ed25519 signatures, hardware binding)       │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. C4 Model — Level 3: Component Diagram (Server)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      workshop-server Components                         │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     HTTP LAYER                                  │   │
│  │                                                                 │   │
│  │  ┌─────────────┐ ┌──────────────┐ ┌──────────────────────────┐ │   │
│  │  │ CorsLayer   │ │Security      │ │ TraceLayer               │ │   │
│  │  │ (localhost  │ │ Headers      │ │ (request/response        │ │   │
│  │  │  origins)   │ │ HSTS,        │ │  logging)                │ │   │
│  │  │             │ │ X-Frame,     │ │                          │ │   │
│  │  │             │ │ X-Content    │ │                          │ │   │
│  │  └─────────────┘ └──────────────┘ └──────────────────────────┘ │   │
│  │  ┌─────────────────────────────────────────────────────────────┐│   │
│  │  │ RequestBodyLimitLayer (10MB)                                ││   │
│  │  └─────────────────────────────────────────────────────────────┘│   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              │                                          │
│                              ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     MIDDLEWARE STACK                             │   │
│  │                                                                 │   │
│  │  Layer 1: api_key_middleware                                    │   │
│  │    └─ Validates X-WorkshopManager-Key header                   │   │
│  │                                                                 │   │
│  │  Layer 2: require_device_key                                    │   │
│  │    └─ Validates X-WorkshopManager-Device-Key header            │   │
│  │    └─ Checks device_keys table (hash + active status)          │   │
│  │    └─ Updates last_seen_at timestamp                           │   │
│  │                                                                 │   │
│  │  Layer 3: authenticate_middleware                               │   │
│  │    └─ Extracts Bearer token from Authorization header          │   │
│  │    └─ Validates JWT (HS256, expiration)                        │   │
│  │    └─ Injects AuthenticatedUser into request extensions        │   │
│  │                                                                 │   │
│  │  Layer 4: require_admin_middleware                              │   │
│  │    └─ Checks user.role == Admin                                │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              │                                          │
│                              ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     ROUTE HANDLERS                               │   │
│  │                                                                 │   │
│  │  PUBLIC:              PROTECTED:            ADMIN:              │   │
│  │  POST /api/auth/login GET /api/products     GET /api/users      │   │
│  │  POST /api/auth/reg   GET /api/sales        POST /api/users     │   │
│  │                       POST /api/sales       PUT /api/users/:id  │   │
│  │                       GET /api/repairs      DELETE /api/users/:id│   │
│  │                       POST /api/repairs     GET /api/device-keys│   │
│  │                       GET /api/suppliers    POST /api/device-keys│   │
│  │                       GET /api/analytics    POST /device-keys/:id/revoke│
│  │                       GET /api/reports      POST /device-keys/:id/unbind│
│  │                       GET /health                              │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              │                                          │
│                              ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     SERVICE LAYER                                │   │
│  │                                                                 │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐  │   │
│  │  │  Auth    │ │  Crypto  │ │  Audit   │ │  Rate Limiter    │  │   │
│  │  │ Service  │ │ Service  │ │ Service  │ │  (Login only)    │  │   │
│  │  │          │ │          │ │          │ │  5 attempts/5min  │  │   │
│  │  │ Argon2id │ │ AES-256- │ │ log_     │ │                  │  │   │
│  │  │ JWT HS256│ │ GCM      │ │ change() │ │  RwLock<HashMap> │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘  │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐  │   │
│  │  │  Backup  │ │  Device  │ │  Config  │ │  Secrets         │  │   │
│  │  │ Service  │ │ Key Svc  │ │ Service  │ │  Service         │  │   │
│  │  │          │ │          │ │          │ │                  │  │   │
│  │  │ pg_dump  │ │ SHA-256  │ │ TOML     │ │ load/generate    │  │   │
│  │  │ + gzip   │ │ hashed   │ │ loader   │ │ JWT secret       │  │   │
│  │  │ + encrypt│ │ keys     │ │          │ │ crypto key       │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              │                                          │
│                              ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     DATA LAYER                                   │   │
│  │                                                                 │   │
│  │  ┌──────────────────────────────────────────────────────────┐  │   │
│  │  │ AppState (shared via Axum FromRef)                       │  │   │
│  │  │                                                          │  │   │
│  │  │  Secrets:                                                │  │   │
│  │  │    jwt_secret: String                                    │  │   │
│  │  │    crypto_key: Vec<u8> (32 bytes)                        │  │   │
│  │  │                                                          │  │   │
│  │  │  Config:                                                 │  │   │
│  │  │    host: String (default: "127.0.0.1")                   │  │   │
│  │  │    port: u16 (default: 8443)                             │  │   │
│  │  │    api_key: String                                       │  │   │
│  │  │    require_device_key: bool                              │  │   │
│  │  │                                                          │  │   │
│  │  │  Pool: PgPool (sqlx connection pool)                     │  │   │
│  │  │  RateLimiter: Arc<RateLimiter>                           │  │   │
│  │  └──────────────────────────────────────────────────────────┘  │   │
│  │                                                                 │   │
│  │  ┌──────────────────────────────────────────────────────────┐  │   │
│  │  │ PostgreSQL (workshop_manager)                            │  │   │
│  │  │                                                          │  │   │
│  │  │  workshops ──┐                                          │  │   │
│  │  │  users ──────┤── workshop_id (FK)                       │  │   │
│  │  │  products ───┤── supplier_id (FK)                       │  │   │
│  │  │  sales ──────┤── workshop_id (FK)                       │  │   │
│  │  │  sale_items ─┤── sale_id (FK), product_id (FK)          │  │   │
│  │  │  repairs ────┤── technician_id → users (FK)             │  │   │
│  │  │  repair_updates─┤── repair_id (FK), created_by → users  │  │   │
│  │  │  repair_parts──┤── repair_id (FK), product_id (FK)      │  │   │
│  │  │  suppliers ──┘                                          │  │   │
│  │  │  audit_log ──── user_id → users (FK)                    │  │   │
│  │  │  device_keys ── (no FK, key_hash indexed)               │  │   │
│  │  └──────────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Data Flow Diagrams

### 4.1 Login Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  Viewer  │     │ API Key  │     │ Device   │     │  Auth    │     │   DB     │
│  (UI)    │     │ Middleware│    │ Key MW   │     │ Middleware│     │          │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │                │
     │ POST /api/auth/login            │                │                │
     │ {email, password}               │                │                │
     │───────────────►│                │                │                │
     │                │                │                │                │
     │                │ Validate       │                │                │
     │                │ X-WorkshopManager-Key            │                │
     │                │───────────────►│                │                │
     │                │                │                │                │
     │                │                │ Skip (public   │                │
     │                │                │ route)         │                │
     │                │                │───────────────►│                │
     │                │                │                │                │
     │                │                │                │ Rate limit    │
     │                │                │                │ check         │
     │                │                │                │───────────────►│
     │                │                │                │                │
     │                │                │                │ Query user     │
     │                │                │                │ by email       │
     │                │                │                │───────────────►│
     │                │                │                │                │
     │                │                │                │◄───────────────│
     │                │                │                │ User row       │
     │                │                │                │                │
     │                │                │                │ Verify         │
     │                │                │                │ Argon2id hash  │
     │                │                │                │                │
     │                │                │                │ Create JWT     │
     │                │                │                │ (8h expiry)    │
     │                │                │                │                │
     │                │                │                │ Record attempt │
     │                │                │                │───────────────►│
     │                │                │                │                │
     │◄───────────────│───────────────│───────────────│                │
     │ {token, user,  │                │                │                │
     │  workshop}     │                │                │                │
     │                │                │                │                │
```

### 4.2 POS Sale Transaction Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  Viewer  │     │  Server  │     │  Audit   │     │   DB     │
│  (POS)   │     │  Routes  │     │  Log     │     │          │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ POST /api/sales                │                │
     │ {items, payment, discount}     │                │
     │───────────────►│                │                │
     │                │                │                │
     │                │ Validate role  │                │
     │                │ (Admin|Seller) │                │
     │                │                │                │
     │                │ Validate input │                │
     │                │ (email, phone, │                │
     │                │  quantities,   │                │
     │                │  discounts)    │                │
     │                │                │                │
     │                │ BEGIN TX       │                │
     │                │───────────────►│                │
     │                │                │                │
     │                │ For each item (sorted by product_id):   │
     │                │                │                │
     │                │ SELECT ... FOR UPDATE           │
     │                │ (lock product row)              │
     │                │───────────────►│                │
     │                │                │                │
     │                │ Check stock >= quantity         │
     │                │                │                │
     │                │ UPDATE products SET stock -= qty│
     │                │───────────────►│                │
     │                │                │                │
     │                │ INSERT INTO sale_items          │
     │                │───────────────►│                │
     │                │                │                │
     │                │ Calculate totals:               │
     │                │ - subtotal (sum of items)       │
     │                │ - discount_amount               │
     │                │ - extract_iva(total) →          │
     │                │   (taxable_amount, tax_amount)  │
     │                │                │                │
     │                │ INSERT INTO sales               │
     │                │───────────────►│                │
     │                │                │                │
     │                │ COMMIT TX      │                │
     │                │───────────────►│                │
     │                │                │                │
     │                │ Audit (best-effort)             │
     │                │───────────────►│                │
     │                │                │ INSERT audit_log│
     │                │                │───────────────►│
     │                │                │                │
     │◄───────────────│                │                │
     │ {sale, items}  │                │                │
     │                │                │                │
```

### 4.3 Repair Lifecycle Flow

```
                    ┌──────────────┐
                    │    Create    │
                    │   Repair     │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │    PENDING   │◄─────────────────────┐
                    │  (initial)   │                      │
                    └──────┬───────┘                      │
                           │                              │
                     Start work                           │
                           │                              │
                           ▼                              │
                    ┌──────────────┐                      │
                    │ IN_PROGRESS  │                      │
                    │ (diagnosis,  │                      │
                    │  parts, cost)│                      │
                    └──────┬───────┘                      │
                           │                              │
              ┌────────────┼────────────┐                 │
              │            │            │                 │
              ▼            ▼            │                 │
       ┌────────────┐ ┌────────────┐   │                 │
       │ COMPLETED  │ │ CANCELLED  │   │                 │
       │ (final_cost│ │            │   │                 │
       │  set)      │ │            │   │                 │
       └────────────┘ └────────────┘   │                 │
              │            │           │                 │
              │            └───────────┼─────────────────┘
              │                        │
              ▼                        │
       ┌────────────┐                  │
       │  DELETED   │                  │
       │ (soft del) │                  │
       └────────────┘                  │
                                       │
                    ┌──────────────────┘
                    │
              Can reopen?
              (Cancelled → Pending)
```

**Repair state transitions:**

| From | To | Trigger |
|------|----|---------|
| (new) | Pending | `POST /api/repairs` |
| Pending | InProgress | `PUT /api/repairs/:id` with `status: "in_progress"` |
| InProgress | Completed | `PUT /api/repairs/:id` with `status: "completed"` |
| InProgress | Cancelled | `PUT /api/repairs/:id` with `status: "cancelled"` |
| Pending | Cancelled | `PUT /api/repairs/:id` with `status: "cancelled"` |
| Any | Deleted | `DELETE /api/repairs/:id` (soft delete) |
| Cancelled | Pending | `PUT /api/repairs/:id` with `status: "pending"` (reopen) |

---

## 5. Integration Patterns

### 5.1 Client-Server Communication

| Pattern | Implementation |
|---------|---------------|
| Protocol | HTTPS (TLS 1.3) over localhost |
| Serialization | JSON (application/json) |
| Authentication | Bearer JWT token in Authorization header |
| API Key | `X-WorkshopManager-Key` header (shared secret) |
| Device Key | `X-WorkshopManager-Device-Key` header (optional) |
| Error Format | `{"success": false, "error": "message"}` |
| Success Format | `{"success": true, "data": {...}}` |
| Pagination | Query params: `?page=1&per_page=20` |
| Pagination Response | `{"items": [...], "total": N, "page": P, "per_page": PP}` |

### 5.2 Middleware Chain Execution Order

```
Request → CorsLayer → Security Headers → RequestBodyLimit
       → TraceLayer → api_key_middleware
       → device_key_middleware → authenticate_middleware
       → require_admin_middleware (admin routes only)
       → Route Handler → Response
```

### 5.3 Error Propagation Pattern

```
Route Handler
    │
    ├─ Validation error → AppError::Validation → 422
    ├─ Not found → AppError::NotFound → 404
    ├─ Conflict → AppError::Conflict → 409
    ├─ DB error → AppError::Internal → 500 (sanitized)
    ├─ Auth error → StatusCode::UNAUTHORIZED → 401
    ├─ Forbidden → StatusCode::FORBIDDEN → 403
    └─ Rate limited → AppError::TooManyRequests → 429
```

---

## 6. Concurrency Model

### 6.1 Tokio Async Runtime

The server uses `#[tokio::main]` with the full feature set, providing:

| Feature | Usage |
|---------|-------|
| Multi-threaded scheduler | Request handling across CPU cores |
| `tokio::spawn` | Backup scheduler runs as background task |
| `tokio::signal::ctrl_c` | Graceful shutdown detection |
| `tokio::select!` | Windows tray + ctrl_c shutdown race |
| `tokio::time::sleep` | Backup interval (24 hours) |
| `tokio::sync::oneshot` | Tray shutdown signal |

### 6.2 Shared State

```
AppState (Clone, shared via Axum)
    │
    ├─ Secrets (Clone) ──────── Immutable after init
    ├─ ServerConfig (Clone) ─── Immutable after init
    ├─ PgPool (Clone) ───────── Connection pool (thread-safe)
    └─ Arc<RateLimiter> ─────── RwLock<HashMap> (interior mutability)
```

### 6.3 Database Concurrency

| Pattern | Usage | Rationale |
|---------|-------|-----------|
| Connection pooling | `PgPool` with default settings | Handles concurrent requests |
| `SELECT ... FOR UPDATE` | Product stock check in POS | Prevents overselling |
| Sorted lock ordering | Items sorted by `product_id` before locking | Prevents deadlocks |
| Transaction isolation | `BEGIN`/`COMMIT`/`ROLLBACK` | Atomic sale creation |
| Best-effort audit | `if let Err(e) = audit::log_change()` | Audit failures don't block operations |

### 6.4 Rate Limiter Concurrency

```rust
// Thread-safe via RwLock (multiple readers, single writer)
struct RateLimiter {
    attempts: RwLock<HashMap<String, (u32, Instant)>>,
    max_attempts: u32,       // 5
    window: Duration,        // 300 seconds (5 minutes)
}
```

- `check()` acquires read lock → checks count and window
- `record_attempt()` acquires write lock → increments count
- Window resets automatically after 300 seconds

---

## 7. State Management

### 7.1 Server-Side State (AppState)

The `AppState` is created once at startup and cloned into every request via Axum's `FromRef` trait:

```rust
pub struct AppState {
    pub secrets: Secrets,           // JWT secret, crypto key
    pub config: ServerConfig,       // Host, port, API key, device key flag
    pub pool: PgPool,              // Database connection pool
    pub login_rate_limiter: Arc<RateLimiter>,  // Login rate limiting
}
```

**Extraction in handlers:**

| Extractor | Usage |
|-----------|-------|
| `State(state)` | Access to full AppState |
| `Extension(user)` | `AuthenticatedUser` (inserted by middleware) |
| `Path(id)` | URL path parameters |
| `Query(params)` | Query string parameters |
| `Json(body)` | Request body deserialization |

### 7.2 Client-Side State (workshop-viewer)

| Provider | State | Purpose |
|----------|-------|---------|
| `AuthProvider` | `token: Option<String>`, `user: Option<User>` | JWT session management |
| `TabsProvider` | `active_tab: Signal<String>` | UI tab navigation |
| `ThemeProvider` | `is_dark: bool` | Light/dark mode toggle |
| Router | `Route` enum | Client-side page routing |

### 7.3 State Lifecycle

```
Startup:
  1. crypto::init() → CIPHER (OnceLock)
  2. secrets::init_secrets() → Secrets struct
  3. config::load_config() → ServerConfig struct
  4. db_manager::start() → database_url
  5. create_pool() → PgPool
  6. run_migrations() → schema ready
  7. AppState::new() → shared state
  8. Backup scheduler → tokio::spawn (background)

Runtime:
  - AppState cloned into every request
  - PgPool shared across all handlers
  - RateLimiter shared via Arc

Shutdown:
  1. Signal received (ctrl_c or tray)
  2. handle.graceful_shutdown(10s timeout)
  3. db_manager::stop() → PostgreSQL stops
  4. Process exits
```

---

## 8. Cross-References

| Document | Description |
|----------|-------------|
| [Architecture Overview](./overview.md) | High-level system purpose and component descriptions |
| [Data Flow](./data-flow.md) | Detailed request lifecycle and transaction flows |
| [Technology Stack](./technology-stack.md) | Dependencies, library rationale, version matrix |
