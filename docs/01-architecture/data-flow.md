# Data Flow — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, QA engineers

---

## 1. Request Lifecycle (Client to DB and Back)

### 1.1 Complete Request Pipeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        REQUEST LIFECYCLE                                 │
│                                                                         │
│  ┌───────────┐                                                          │
│  │ 1. Client │  Viewer sends HTTPS request                             │
│  │    Send   │  POST /api/sales {items, payment_method, ...}           │
│  └─────┬─────┘                                                          │
│        │                                                                │
│        ▼                                                                │
│  ┌───────────────┐                                                      │
│  │ 2. TLS        │  rustls terminates TLS 1.3                          │
│  │    Terminate  │  Self-signed certificate verified                   │
│  └─────┬─────────┘                                                      │
│        │                                                                │
│        ▼                                                                │
│  ┌───────────────┐                                                      │
│  │ 3. Tower      │  CorsLayer → Security Headers →                     │
│  │    Layers     │  RequestBodyLimit (10MB) → TraceLayer               │
│  └─────┬─────────┘                                                      │
│        │                                                                │
│        ▼                                                                │
│  ┌───────────────┐                                                      │
│  │ 4. Route      │  Router matches POST /api/sales                     │
│  │    Matching   │  Selects correct handler function                   │
│  └─────┬─────────┘                                                      │
│        │                                                                │
│        ▼                                                                │
│  ┌───────────────┐                                                      │
│  │ 5. Middleware  │  api_key → device_key → authenticate                │
│  │    Stack      │  (Each layer passes or rejects)                     │
│  └─────┬─────────┘                                                      │
│        │                                                                │
│        ▼                                                                │
│  ┌───────────────┐                                                      │
│  │ 6. Extractors │  State(state), Extension(user), Json(req)           │
│  │    (Axum)     │  Deserializes request body into CreateSaleRequest    │
│  └─────┬─────────┘                                                      │
│        │                                                                │
│        ▼                                                                │
│  ┌───────────────┐                                                      │
│  │ 7. Business   │  Role check, input validation,                      │
│  │    Logic      │  IVA calculation, stock verification                │
│  └─────┬─────────┘                                                      │
│        │                                                                │
│        ▼                                                                │
│  ┌───────────────┐                                                      │
│  │ 8. Database   │  BEGIN → SELECT FOR UPDATE → UPDATE → INSERT        │
│  │    Tx         │  → COMMIT (or ROLLBACK on error)                    │
│  └─────┬─────────┘                                                      │
│        │                                                                │
│        ▼                                                                │
│  ┌───────────────┐                                                      │
│  │ 9. Audit      │  Best-effort audit log insertion                    │
│  │    Log        │  (failures don't block response)                    │
│  └─────┬─────────┘                                                      │
│        │                                                                │
│        ▼                                                                │
│  ┌───────────────┐                                                      │
│  │ 10. Response  │  Json(ApiResponse::success(sale_detail))            │
│  │     Serialize │  HTTP 200 with JSON body                            │
│  └─────┬─────────┘                                                      │
│        │                                                                │
│        ▼                                                                │
│  ┌───────────────┐                                                      │
│  │ 11. Client    │  Viewer deserializes ApiResponse<SaleDetail>        │
│  │     Receive   │  Updates UI state                                   │
│  └───────────────┘                                                      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Request-Response Transformation

| Stage | Input | Output |
|-------|-------|--------|
| Client | Rust struct `CreateSaleRequest` | JSON bytes |
| TLS | Encrypted bytes | Decrypted bytes |
| Tower Layers | Raw HTTP request | Augmented request (CORS headers added) |
| Route Matching | HTTP method + path | Handler function reference |
| Middleware | Request + state | Request + `AuthenticatedUser` extension |
| Extractors | Request body bytes | Typed Rust structs |
| Business Logic | Validated input | Domain objects |
| Database | SQL queries | Row data |
| Audit | JSON values | Audit log row |
| Response | `ApiResponse<SaleDetail>` | JSON bytes |
| Client | JSON bytes | `Result<SaleDetail, ApiError>` |

---

## 2. Authentication Flow

### 2.1 Login Flow (Detailed)

```
┌──────────┐              ┌──────────┐              ┌──────────┐
│  Viewer  │              │  Server  │              │    DB    │
└────┬─────┘              └────┬─────┘              └────┬─────┘
     │                         │                         │
     │ 1. POST /api/auth/login │                         │
     │ {email, password}       │                         │
     │────────────────────────►│                         │
     │                         │                         │
     │  2. api_key_middleware   │                         │
     │  ───────────────────────┤                         │
     │  Validate header:       │                         │
     │  X-WorkshopManager-Key  │                         │
     │  == config.api_key      │                         │
     │  ✓ Pass                 │                         │
     │                         │                         │
     │  3. device_key_middleware│                         │
     │  ───────────────────────┤                         │
     │  (Skipped for public    │                         │
     │   routes)               │                         │
     │                         │                         │
     │  4. Rate limit check    │                         │
     │  ───────────────────────┤                         │
     │  limiter.check("email") │                         │
     │  ✓ Under limit          │                         │
     │                         │                         │
     │  5. Query user by email │                         │
     │  ───────────────────────┼────────────────────────►│
     │                         │  SELECT * FROM users    │
     │                         │  WHERE email = $1       │
     │                         │◄────────────────────────│
     │                         │  User row or None       │
     │                         │                         │
     │  6. Verify password     │                         │
     │  ───────────────────────┤                         │
     │  argon2.verify_password │                         │
     │  (hash, password)       │                         │
     │  ✓ Match                │                         │
     │                         │                         │
     │  7. Create JWT          │                         │
     │  ───────────────────────┤                         │
     │  Claims:                │                         │
     │    sub: user_id         │                         │
     │    email: user.email    │                         │
     │    role: user.role      │                         │
     │    workshop_id: UUID    │                         │
     │    exp: now + 8 hours   │                         │
     │  Sign with jwt_secret   │                         │
     │                         │                         │
     │  8. Record attempt      │                         │
     │  ───────────────────────┤                         │
     │  limiter.record("email")│                         │
     │                         │                         │
     │  9. Response 200        │                         │
     │◄────────────────────────│                         │
     │ {token, user, workshop} │                         │
     │                         │                         │
     │  10. Store token        │                         │
     │  ───────────────────────┤                         │
     │  auth_state.token =     │                         │
     │  Some(jwt)              │                         │
     │                         │                         │
```

### 2.2 Subsequent Authenticated Request

```
┌──────────┐              ┌──────────┐              ┌──────────┐
│  Viewer  │              │  Server  │              │    DB    │
└────┬─────┘              └────┬─────┘              └────┬─────┘
     │                         │                         │
     │  GET /api/products      │                         │
     │  Headers:               │                         │
     │   Authorization:        │                         │
     │     Bearer <jwt_token>  │                         │
     │   X-WorkshopManager-Key:│                         │
     │     <api_key>           │                         │
     │   X-WorkshopManager-    │                         │
     │     Device-Key: <key>   │                         │
     │────────────────────────►│                         │
     │                         │                         │
     │  1. api_key_middleware   │                         │
     │  ✓ Pass                 │                         │
     │                         │                         │
     │  2. device_key_middleware│                         │
     │  ───────────────────────┤                         │
     │  hash(key) → key_hash   │                         │
     │  SELECT active FROM     │                         │
     │  device_keys WHERE      │                         │
     │  key_hash = $1          │                         │
     │  ───────────────────────┼────────────────────────►│
     │                         │◄────────────────────────│
     │  ✓ active = true        │                         │
     │                         │                         │
     │  3. authenticate_mw     │                         │
     │  ───────────────────────┤                         │
     │  Extract "Bearer <jwt>" │                         │
     │  jwt::decode(token,      │                         │
     │    secret)              │                         │
     │  ✓ Valid, not expired   │                         │
     │  Insert AuthenticatedUser│                         │
     │  {id, email, role,      │                         │
     │   workshop_id}          │                         │
     │                         │                         │
     │  4. Route handler       │                         │
     │  Extension(user):       │                         │
     │    user.workshop_id     │                         │
     │  SELECT ... WHERE       │                         │
     │  workshop_id = $1       │                         │
     │  ───────────────────────┼────────────────────────►│
     │                         │◄────────────────────────│
     │                         │  Filtered results       │
     │                         │                         │
     │  5. Response 200        │                         │
     │◄────────────────────────│                         │
     │ {success, data: [...]}  │                         │
     │                         │                         │
```

### 2.3 Authentication Failure Scenarios

| Scenario | Middleware | HTTP Status | Response |
|----------|-----------|-------------|----------|
| Missing API key | `api_key_middleware` | 401 | `{"success": false, "error": "Unauthorized"}` |
| Invalid API key | `api_key_middleware` | 401 | `{"success": false, "error": "Unauthorized"}` |
| Missing device key (when required) | `require_device_key` | 401 | `{"success": false, "error": "Unauthorized"}` |
| Invalid device key | `require_device_key` | 401 | `{"success": false, "error": "Unauthorized"}` |
| Missing Authorization header | `authenticate_middleware` | 401 | `{"success": false, "error": "Unauthorized"}` |
| Missing "Bearer " prefix | `authenticate_middleware` | 401 | `{"success": false, "error": "Unauthorized"}` |
| Expired JWT | `authenticate_middleware` | 401 | `{"success": false, "error": "Unauthorized"}` |
| Invalid JWT signature | `authenticate_middleware` | 401 | `{"success": false, "error": "Unauthorized"}` |
| Invalid role in claims | `authenticate_middleware` | 401 | `{"success": false, "error": "Unauthorized"}` |
| Non-admin on admin route | `require_admin_middleware` | 403 | `{"success": false, "error": "Forbidden"}` |
| Rate limit exceeded | Route handler | 429 | `{"success": false, "error": "Too many requests"}` |

---

## 3. POS Transaction Flow (Atomic Stock Deduction)

### 3.1 Complete POS Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     POS TRANSACTION FLOW                                 │
│                                                                         │
│  STEP 1: PRODUCT LOOKUP (Barcode Scanner)                               │
│  ─────────────────────────────────────────                              │
│  Viewer → GET /api/products/lookup?barcode=7891234567890               │
│  Server → SELECT id, name, price, stock FROM products                  │
│           WHERE barcode = $1 AND status = 'active'                     │
│           AND workshop_id = $2                                         │
│  Returns → PosProductResponse {product_id, name, price, stock, ...}   │
│                                                                         │
│  STEP 2: ADD TO CART (Client-side)                                      │
│  ────────────────────────────────                                       │
│  Viewer maintains local cart state:                                    │
│  Vec<{product_id, name, quantity, unit_price, discount}>               │
│                                                                         │
│  STEP 3: PROCESS SALE                                                  │
│  ──────────────────                                                     │
│                                                                         │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     │
│  │  Viewer  │     │  Server  │     │   Lock   │     │   DB     │     │
│  │  (POS)   │     │  (Axum)  │     │  Manager │     │ (PG)     │     │
│  └────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘     │
│       │                │                │                │            │
│       │ POST /api/sales│                │                │            │
│       │ {items, pay,   │                │                │            │
│       │  discount}     │                │                │            │
│       │───────────────►│                │                │            │
│       │                │                │                │            │
│       │                │ BEGIN TX       │                │            │
│       │                │───────────────►│───────────────►│            │
│       │                │                │                │            │
│       │                │ Sort items by product_id        │            │
│       │                │ (deadlock prevention)           │            │
│       │                │                │                │            │
│       │                │ For item[0]:   │                │            │
│       │                │ SELECT ... FOR UPDATE          │            │
│       │                │───────────────►│───────────────►│            │
│       │                │                │◄───────────────│            │
│       │                │ (name, price,  │ LOCK row       │            │
│       │                │  stock)        │                │            │
│       │                │                │                │            │
│       │                │ Check: stock[0] >= qty[0]?     │            │
│       │                │ ✓ OK           │                │            │
│       │                │                │                │            │
│       │                │ UPDATE products                │            │
│       │                │ SET stock = stock - qty[0]      │            │
│       │                │ WHERE id = product_id[0]       │            │
│       │                │───────────────►│───────────────►│            │
│       │                │                │                │            │
│       │                │ INSERT sale_items[0]            │            │
│       │                │───────────────►│───────────────►│            │
│       │                │                │                │            │
│       │                │ (repeat for item[1..n])         │            │
│       │                │                │                │            │
│       │                │ Calculate:     │                │            │
│       │                │ subtotal = Σ(item.unit_price × │            │
│       │                │              item.quantity)     │            │
│       │                │ discount_total =                │            │
│       │                │   subtotal × discount / 100    │            │
│       │                │ total = subtotal - discount     │            │
│       │                │ (taxable, tax) =                │            │
│       │                │   extract_iva(total)            │            │
│       │                │                │                │            │
│       │                │ INSERT sales   │                │            │
│       │                │───────────────►│───────────────►│            │
│       │                │                │                │            │
│       │                │ COMMIT TX      │                │            │
│       │                │───────────────►│───────────────►│            │
│       │                │                │                │            │
│       │                │ Audit log (best-effort)         │            │
│       │                │ INSERT audit_log                │            │
│       │                │───────────────►│───────────────►│            │
│       │                │                │                │            │
│       │◄───────────────│                │                │            │
│       │ {sale, items}  │                │                │            │
│       │                │                │                │            │
```

### 3.2 IVA Calculation (Chilean Tax)

Chilean prices include IVA (19%). The system extracts base and tax amounts:

```
Given: total = $11.900 CLP

Step 1: Calculate one_plus_iva = 1 + 0.19 = 1.19
Step 2: base = round_to_ten(total / 1.19)
        base = round_to_ten(10000.00) = $10.000
Step 3: iva = total - base
        iva = $11.900 - $10.000 = $1.900

Result: (taxable_amount: $10.000, tax_amount: $1.900)
```

### 3.3 CLP Rounding Rules

Chilean law requires rounding to the nearest $10:

| Input | Output | Rule |
|-------|--------|------|
| $15.678 | $15.680 | Round up (remainder 8 ≥ 5) |
| $15.673 | $15.670 | Round down (remainder 3 < 5) |
| $15.675 | $15.680 | Round up (remainder 5, round away from zero) |
| $999 | $1.000 | Round up |
| $15 | $20 | Round up |
| -$15.678 | -$15.680 | Round away from zero |

### 3.4 Deadlock Prevention

```rust
// Items are sorted by product_id before locking
let mut sorted_items = req.items.clone();
sorted_items.sort_by_key(|a| a.product_id);

// Each product is locked with SELECT ... FOR UPDATE
// in consistent order, preventing deadlocks
for item_req in &sorted_items {
    sqlx::query_as(
        "SELECT name, price, stock FROM products
         WHERE id = $1 AND status = 'active' AND workshop_id = $2
         FOR UPDATE"
    )
    .bind(item_req.product_id)
    .bind(workshop_id)
    .fetch_optional(&mut **tx)
    .await?;
}
```

---

## 4. Repair Lifecycle Flow

### 4.1 Repair Creation Flow

```
┌──────────┐              ┌──────────┐              ┌──────────┐
│  Viewer  │              │  Server  │              │    DB    │
└────┬─────┘              └────┬─────┘              └────┬─────┘
     │                         │                         │
     │ POST /api/repairs       │                         │
     │ {customer_name,         │                         │
     │  vehicle, license_plate,│                         │
     │  description, priority, │                         │
     │  estimated_cost,        │                         │
     │  estimated_delivery}    │                         │
     │────────────────────────►│                         │
     │                         │                         │
     │  Validate license plate │                         │
     │  (if provided):         │                         │
     │  - normalize (remove    │                         │
     │    spaces, hyphens)     │                         │
     │  - detect type (Antigua,│                         │
     │    Nueva, Moto, Policia)│                         │
     │  ✓ Valid format         │                         │
     │                         │                         │
     │  INSERT INTO repairs    │                         │
     │  (id, workshop_id,      │                         │
     │   customer_*, vehicle,  │                         │
     │   license_plate,        │                         │
     │   description, priority,│                         │
     │   status='pending',     │                         │
     │   estimated_cost,       │                         │
     │   estimated_delivery)   │                         │
     │────────────────────────►│────────────────────────►│
     │                         │                         │
     │  Audit log              │                         │
     │────────────────────────►│────────────────────────►│
     │                         │                         │
     │◄────────────────────────│                         │
     │ {repair}                │                         │
     │                         │                         │
```

### 4.2 Repair Update Flow

```
┌──────────┐              ┌──────────┐              ┌──────────┐
│  Viewer  │              │  Server  │              │    DB    │
└────┬─────┘              └────┬─────┘              └────┬─────┘
     │                         │                         │
     │ PUT /api/repairs/:id    │                         │
     │ {status: "in_progress", │                         │
     │  diagnosis: "Engine     │                         │
     │  oil leak",             │                         │
     │  estimated_cost: 85000} │                         │
     │────────────────────────►│                         │
     │                         │                         │
     │  BEGIN TX               │                         │
     │────────────────────────►│────────────────────────►│
     │                         │                         │
     │  UPDATE repairs         │                         │
     │  SET status = $2,       │                         │
     │      diagnosis = $3,    │                         │
     │      estimated_cost=$4, │                         │
     │      updated_at = NOW() │                         │
     │  WHERE id = $1          │                         │
     │  AND workshop_id = $5   │                         │
     │────────────────────────►│────────────────────────►│
     │                         │                         │
     │  INSERT INTO            │                         │
     │  repair_updates         │                         │
     │  (id, repair_id,        │                         │
     │   status, description,  │                         │
     │   created_by, created_at)│                        │
     │────────────────────────►│────────────────────────►│
     │                         │                         │
     │  COMMIT TX              │                         │
     │────────────────────────►│────────────────────────►│
     │                         │                         │
     │  Audit log              │                         │
     │────────────────────────►│────────────────────────►│
     │                         │                         │
     │◄────────────────────────│                         │
     │ {repair, updates}       │                         │
     │                         │                         │
```

### 4.3 Repair Parts Management

```
Add Part:                Remove Part:
POST /api/repairs/:id/parts   DELETE /api/repairs/:id/parts/:part_id
                               │
┌──────────────┐          ┌──────────────┐
│ 1. Validate  │          │ 1. Verify    │
│    name, qty │          │    part      │
│    unit_cost │          │    exists    │
└──────┬───────┘          └──────┬───────┘
       │                         │
       ▼                         ▼
┌──────────────┐          ┌──────────────┐
│ 2. INSERT    │          │ 2. DELETE    │
│    repair_   │          │    FROM     │
│    parts     │          │    repair_  │
│    (linked   │          │    parts    │
│    to repair)│          │    WHERE id │
└──────┬───────┘          └──────┬───────┘
       │                         │
       ▼                         ▼
┌──────────────┐          ┌──────────────┐
│ 3. Audit log │          │ 3. Audit log │
└──────────────┘          └──────────────┘
```

---

## 5. Backup Flow

### 5.1 Automated Backup Process

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        BACKUP FLOW                                       │
│                                                                         │
│  BOOT (immediate):                                                      │
│  ─────────────────                                                      │
│  1. backup_scheduler() spawned as tokio task                            │
│  2. create_backup() called immediately                                  │
│  3. prune_old_backups() called immediately                              │
│                                                                         │
│  EVERY 24 HOURS:                                                        │
│  ────────────────                                                       │
│                                                                         │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐           │
│  │   Backup     │     │   Compress   │     │   Encrypt    │           │
│  │   Scheduler  │     │   (gzip)     │     │   (AES-256)  │           │
│  └──────┬───────┘     └──────┬───────┘     └──────┬───────┘           │
│         │                    │                    │                     │
│         │ pg_dump            │                    │                     │
│         │ --dbname           │                    │                     │
│         │ <database_url>     │                    │                     │
│         │ --clean            │                    │                     │
│         │ --if-exists        │                    │                     │
│         │                    │                    │                     │
│         │ stdout (SQL)       │ gzip bytes         │ base64 string      │
│         │───────────────────►│───────────────────►│                    │
│         │                    │                    │                    │
│         │                    │                    │ Write to:          │
│         │                    │                    │ backups/           │
│         │                    │                    │ workshop_manager_  │
│         │                    │                    │ backup_YYYYMMDD_   │
│         │                    │                    │ HHMMSS.sql.gz.enc  │
│         │                    │                    │                    │
│         │                    │                    │───────────────────►│
│         │                    │                    │                    │
│                                                                         │
│  RETENTION POLICY:                                                      │
│  ─────────────────                                                      │
│  1. Read all .enc files in backups/                                    │
│  2. Sort by modification time (oldest first)                           │
│  3. If count > 7: delete oldest files until count = 7                  │
│                                                                         │
│  SHUTDOWN:                                                              │
│  ─────────                                                              │
│  Backup scheduler stops naturally (tokio task cancelled)                │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Backup File Naming Convention

```
workshop_manager_backup_YYYYMMDD_HHMMSS.sql.gz.enc
│                         │         │     │ │  │
│                         │         │     │ │  └─ Encrypted (AES-256-GCM)
│                         │         │     │ └──── Compressed (gzip)
│                         │         │     └────── SQL dump
│                         │         └──────────── Time (UTC)
│                         └────────────────────── Date (UTC)
```

---

## 6. Audit Trail Flow

### 6.1 Audit Log Structure

```sql
CREATE TABLE audit_log (
    id          BIGSERIAL PRIMARY KEY,
    user_id     UUID REFERENCES users(id),
    action      VARCHAR(100) NOT NULL,    -- 'create', 'update', 'delete'
    entity_type VARCHAR(100),              -- 'sale', 'product', 'repair', etc.
    entity_id   UUID,
    old_values  JSONB,                     -- Previous state (for updates)
    new_values  JSONB,                     -- New state (for creates/updates)
    ip_address  INET,
    user_agent  VARCHAR(255),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 6.2 Audit Flow for a Sale Creation

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     AUDIT TRAIL FLOW                                     │
│                                                                         │
│  1. Handler creates sale (success)                                      │
│  ─────────────────────────────────                                      │
│                                                                         │
│  2. Prepare audit data                                                  │
│  ──────────────────────                                                 │
│  old_values: None (new entity)                                          │
│  new_values: serde_json::to_value(&sale_detail)                        │
│                                                                         │
│  3. Redact sensitive fields                                             │
│  ─────────────────────────                                              │
│  redact_sensitive(&mut new_values)                                      │
│  │                                                                       │
│  │  Checks for keys:                                                    │
│  │  - "password_hash" → "[REDACTED]"                                   │
│  │  - "password"      → "[REDACTED]"                                   │
│  │  - "crypto_key"    → "[REDACTED]"                                   │
│  │  - "jwt_secret"    → "[REDACTED]"                                   │
│  │                                                                       │
│  4. Insert audit log (best-effort)                                      │
│  ────────────────────────────────                                       │
│  audit::log_change(                                                     │
│      pool,                                                              │
│      Some(user.id),          // user_id                                 │
│      "create",               // action                                  │
│      "sale",                 // entity_type                             │
│      sale.id,                // entity_id                               │
│      None,                   // old_values                              │
│      Some(new_values),       // new_values                              │
│      None,                   // ip_address                              │
│      None,                   // user_agent                              │
│  ).await;                                                               │
│                                                                         │
│  5. Failure handling                                                    │
│  ───────────────────                                                    │
│  if let Err(e) = audit_result {                                        │
│      tracing::warn!("Audit log failed: {}", e);                        │
│      // Request still succeeds                                          │
│      // Audit is non-blocking                                           │
│  }                                                                      │
│                                                                         │
│  SQL executed:                                                          │
│  INSERT INTO audit_log                                                  │
│  (user_id, action, entity_type, entity_id,                             │
│   old_values, new_values, ip_address, user_agent, created_at)          │
│  VALUES ($1, $2, $3, $4, $5, $6, $7::inet, $8, NOW())                │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.3 Audit Query Patterns

```sql
-- Recent activity for a user
SELECT * FROM audit_log
WHERE user_id = $1
ORDER BY created_at DESC
LIMIT 50;

-- All changes to a specific entity
SELECT * FROM audit_log
WHERE entity_type = 'sale' AND entity_id = $1
ORDER BY created_at DESC;

-- Activity summary by day
SELECT DATE(created_at) as day, action, entity_type, COUNT(*)
FROM audit_log
WHERE created_at > NOW() - INTERVAL '30 days'
GROUP BY DATE(created_at), action, entity_type
ORDER BY day DESC;
```

---

## 7. Error Propagation Flow

### 7.1 Error Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     ERROR PROPAGATION FLOW                               │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     ERROR ORIGINS                                │   │
│  │                                                                 │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐   │   │
│  │  │  Input   │  │ Database │  │  Auth    │  │  Business    │   │   │
│  │  │Validation│  │ Errors   │  │ Failures │  │  Logic       │   │   │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────┬───────┘   │   │
│  │       │              │              │               │           │   │
│  └───────┼──────────────┼──────────────┼───────────────┼───────────┘   │
│          │              │              │               │               │
│          ▼              ▼              ▼               ▼               │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     AppError ENUM                                │   │
│  │                                                                 │   │
│  │  Unauthorized ───────────────────────────────────► 401         │   │
│  │  Forbidden ───────────────────────────────────────► 403         │   │
│  │  NotFound(msg) ───────────────────────────────────► 404         │   │
│  │  BadRequest(msg) ─────────────────────────────────► 400         │   │
│  │  Conflict(msg) ───────────────────────────────────► 409         │   │
│  │  Validation(msg) ─────────────────────────────────► 422         │   │
│  │  TooManyRequests ─────────────────────────────────► 429         │   │
│  │  Internal(msg) ───────────────────────────────────► 500         │   │
│  │    └─ sanitize_db_error(msg)                                     │   │
│  │       ├─ "duplicate key"     → "El recurso ya existe"           │   │
│  │       ├─ "foreign key"       → "Referencia inválida"            │   │
│  │       ├─ "not-null"          → "Faltan campos obligatorios"     │   │
│  │       ├─ "invalid syntax"    → "Formato de dato inválido"      │   │
│  │       └─ (other)             → "Error interno del servidor"     │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│          │                                                             │
│          ▼                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     RESPONSE FORMAT                              │   │
│  │                                                                 │   │
│  │  HTTP {status_code}                                             │   │
│  │  Content-Type: application/json                                 │   │
│  │                                                                 │   │
│  │  {                                                              │   │
│  │    "success": false,                                            │   │
│  │    "error": "Human-readable message"                            │   │
│  │  }                                                              │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│          │                                                             │
│          ▼                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     CLIENT HANDLING                              │   │
│  │                                                                 │   │
│  │  ApiClient.handle_response()                                    │   │
│  │  │                                                              │   │
│  │  ├─ 200-299: Deserialize ApiResponse<T>, return Ok(data)       │   │
│  │  ├─ 401: Return ApiError::Unauthorized                         │   │
│  │  ├─ 403: Return ApiError::Forbidden                            │   │
│  │  ├─ 404: Return ApiError::NotFound(body)                       │   │
│  │  ├─ 422: Return ApiError::Validation(body)                     │   │
│  │  ├─ 500-599: Return ApiError::Server(body)                     │   │
│  │  └─ other: Return ApiError::Unknown(body)                      │   │
│  │                                                                 │   │
│  │  User-facing message:                                           │   │
│  │  ApiError.user_message() → Spanish UI string                    │   │
│  │  "No se pudo conectar con el servidor. Verifica que esté       │   │
│  │   encendido."                                                   │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Error Scenarios Matrix

| Operation | Error | HTTP | AppError | Client Message (Spanish) |
|-----------|-------|------|----------|--------------------------|
| Login | Invalid credentials | 401 | Unauthorized | La sesión ha expirado. |
| Login | Rate limited | 429 | TooManyRequests | Demasiados intentos. |
| Create Sale | Invalid email | 422 | Validation | Revisa los datos ingresados. |
| Create Sale | Insufficient stock | 409 | Conflict | Conflicto de stock. |
| Create Sale | Product not found | 404 | NotFound | El recurso solicitado no fue encontrado. |
| Create Sale | Wrong role | 403 | Forbidden | No tienes permisos para realizar esta acción. |
| Create Sale | DB constraint | 500 | Internal | Error interno del servidor (sanitized). |
| Create Repair | Invalid patente | 422 | Validation | Formato de patente inválido. |
| Update Repair | Not found | 404 | NotFound | Reparación no encontrada. |
| List Products | DB error | 500 | Internal | Error interno del servidor. |
| Any | Network failure | — | Network | No se pudo conectar con el servidor. |
| Any | Expired token | 401 | Unauthorized | La sesión ha expirado. |

### 7.3 DB Error Sanitization

The `sanitize_db_error()` function prevents internal database details from leaking to clients:

```
Raw DB Error                          Sanitized Message
─────────────────────────────────     ─────────────────────────────────
"duplicate key value violates         "El recurso ya existe"
 unique constraint products_sku_key"
 
"update or delete on table            "Referencia inválida"
 products violates foreign key
 constraint sale_items_product_id_fkey"
 
"null value in column name            "Faltan campos obligatorios"
 violates not-null constraint"
 
"invalid input syntax for type uuid   "Formato de dato inválido"
 
(anything else)                       "Error interno del servidor"
```

---

## 8. Cross-References

| Document | Description |
|----------|-------------|
| [Architecture Overview](./overview.md) | High-level system purpose and components |
| [System Design](./system-design.md) | C4 diagrams, integration patterns, concurrency model |
| [Technology Stack](./technology-stack.md) | Dependencies, library rationale, version matrix |
