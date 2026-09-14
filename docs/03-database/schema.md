# Database Schema Reference — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, DBAs, technical stakeholders

---

## 1. Database Engine and Configuration

| Property | Value |
|----------|-------|
| Engine | PostgreSQL (embedded via `postgresql_embedded` crate) |
| Database name | `workshop_manager` |
| Connection | Localhost only (`127.0.0.1`) |
| Pool | `sqlx::PgPool` with configurable connections |
| Migrations | SQLx `migrate!()` macro, compiled into binary |
| Backup | `pg_dump` + gzip + AES-256-GCM encryption |
| Data directory | `$LOCALAPPDATA/WorkshopManager/data/` |

---

## 2. Custom Enum Types

PostgreSQL enum types enforce valid values at the database level. They are defined as `sqlx::Type` in `workshop-common` and shared across server/viewer.

### 2.1 `payment_method`

Methods of payment accepted at point of sale.

| Value | Description |
|-------|-------------|
| `cash` | Physical currency (CLP) |
| `card` | Debit or credit card |
| `transfer` | Bank transfer |

### 2.2 `repair_status`

Lifecycle states for repair orders. Includes a `deleted` soft-delete state.

| Value | Description |
|-------|-------------|
| `pending` | Created, awaiting technician assignment |
| `in_progress` | Work actively underway |
| `completed` | Finished, ready for customer pickup |
| `cancelled` | Customer cancelled or abandoned |
| `deleted` | Soft-deleted (excluded from queries via partial indexes) |

### 2.3 `priority`

Urgency level for repair orders.

| Value | Description |
|-------|-------------|
| `high` | Urgent, customer waiting or safety issue |
| `medium` | Standard priority, normal queue |
| `low` | Non-urgent, can be deferred |

### 2.4 `user_role`

Role-based access control for application users.

| Value | Description |
|-------|-------------|
| `admin` | Full access: user management, configuration, all CRUD |
| `mechanic` | Repairs and parts only; read-only products/sales |
| `seller` | POS and sales only; read-only repairs |

---

## 3. Table Definitions

### 3.1 `workshops`

Top-level entity. All other business tables are scoped to a workshop via `workshop_id`.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `UUID` | `PRIMARY KEY` | — | Unique identifier |
| `name` | `VARCHAR(200)` | `NOT NULL` | — | Workshop display name |
| `address` | `VARCHAR(300)` | `NOT NULL` | — | Physical address |
| `city` | `VARCHAR(120)` | `NOT NULL` | — | City name |
| `barcode_prefix` | `VARCHAR(6)` | — | `NULL` | 1-6 char prefix for generated barcodes |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Creation timestamp |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Last modification timestamp |

### 3.2 `users`

Application users. Each user belongs to exactly one workshop.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `UUID` | `PRIMARY KEY` | — | Unique identifier |
| `email` | `VARCHAR(200)` | `NOT NULL`, `UNIQUE` | — | Login email (global uniqueness) |
| `display_name` | `VARCHAR(200)` | — | `NULL` | Human-readable name |
| `password_hash` | `VARCHAR(255)` | `NOT NULL` | — | Argon2id hash |
| `role` | `user_role` | `NOT NULL` | — | RBAC role |
| `status` | `VARCHAR(50)` | `NOT NULL` | `'active'` | Account status (`active`/`inactive`) |
| `workshop_id` | `UUID` | `NOT NULL`, `FK → workshops(id)` | — | Owning workshop |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Creation timestamp |

**Note:** `email` has a global `UNIQUE` constraint — two workshops cannot share the same email address.

### 3.3 `suppliers`

Vendor directory, scoped per workshop.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `UUID` | `PRIMARY KEY` | — | Unique identifier |
| `workshop_id` | `UUID` | `NOT NULL`, `FK → workshops(id)` | — | Owning workshop |
| `name` | `VARCHAR(200)` | `NOT NULL` | — | Supplier business name |
| `contact_person` | `VARCHAR(200)` | — | `NULL` | Primary contact |
| `email` | `VARCHAR(200)` | — | `NULL` | Contact email |
| `phone` | `VARCHAR(20)` | — | `NULL` | Contact phone |
| `address` | `TEXT` | — | `NULL` | Physical address |
| `tax_id` | `VARCHAR(50)` | — | `NULL` | RUT or tax identifier |
| `payment_terms` | `VARCHAR(200)` | — | `NULL` | Payment conditions (e.g., "30 días") |
| `status` | `VARCHAR(50)` | `NOT NULL` | `'active'` | Supplier status (`active`/`inactive`) |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Creation timestamp |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Last modification timestamp |

### 3.4 `products`

Product catalog with inventory tracking. Barcode and SKU uniqueness enforced per workshop via partial indexes.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `UUID` | `PRIMARY KEY` | — | Unique identifier |
| `workshop_id` | `UUID` | `NOT NULL`, `FK → workshops(id)` | — | Owning workshop |
| `name` | `VARCHAR(200)` | `NOT NULL` | — | Product name |
| `description` | `TEXT` | — | `NULL` | Detailed description |
| `category` | `VARCHAR(100)` | — | `NULL` | Product category |
| `brand` | `VARCHAR(100)` | — | `NULL` | Manufacturer brand |
| `model` | `VARCHAR(100)` | — | `NULL` | Product model |
| `sku` | `VARCHAR(100)` | — | `NULL` | Stock keeping unit |
| `barcode` | `VARCHAR(50)` | — | `NULL` | EAN-13 or custom barcode |
| `price` | `NUMERIC(19,4)` | `NOT NULL` | — | Sale price (CLP, includes IVA) |
| `cost` | `NUMERIC(19,4)` | `NOT NULL` | — | Purchase cost from supplier |
| `stock` | `INTEGER` | `NOT NULL` | `0` | Current quantity in stock |
| `min_stock` | `INTEGER` | `NOT NULL` | `0` | Low-stock alert threshold |
| `location` | `VARCHAR(200)` | — | `NULL` | Physical shelf/aisle location |
| `supplier_id` | `UUID` | `FK → suppliers(id)` | `NULL` | Primary supplier |
| `status` | `VARCHAR(50)` | `NOT NULL` | `'active'` | Status (`active`/`inactive`/`deleted`) |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Creation timestamp |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Last modification timestamp |

**Soft-delete pattern:** Setting `status = 'deleted'` excludes the product from unique barcode/SKU indexes, allowing reuse of barcodes for new products.

### 3.5 `sales`

Sales transactions. Includes tax breakdown fields for Chilean IVA compliance.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `UUID` | `PRIMARY KEY` | — | Unique identifier |
| `workshop_id` | `UUID` | `NOT NULL`, `FK → workshops(id)` | — | Owning workshop |
| `customer_name` | `VARCHAR(200)` | — | `NULL` | Customer name |
| `customer_email` | `VARCHAR(200)` | — | `NULL` | Customer email |
| `customer_phone` | `VARCHAR(20)` | — | `NULL` | Customer phone |
| `subtotal` | `NUMERIC(19,4)` | `NOT NULL` | `0` | Sum of line items before tax |
| `discount_amount` | `NUMERIC(19,4)` | `NOT NULL` | `0` | Total discount applied |
| `taxable_amount` | `NUMERIC(19,4)` | `NOT NULL` | `0` | Amount subject to IVA |
| `tax_amount` | `NUMERIC(19,4)` | `NOT NULL` | `0` | IVA 19% amount |
| `total` | `NUMERIC(19,4)` | `NOT NULL` | — | Final amount (taxable + tax - discount) |
| `payment_method` | `payment_method` | `NOT NULL` | — | Payment type |
| `status` | `VARCHAR(50)` | `NOT NULL` | `'completed'` | Sale status |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Transaction timestamp |

### 3.6 `sale_items`

Individual line items within a sale. Cascades on sale deletion.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `UUID` | `PRIMARY KEY` | — | Unique identifier |
| `sale_id` | `UUID` | `NOT NULL`, `FK → sales(id) ON DELETE CASCADE` | — | Parent sale |
| `product_id` | `UUID` | `NOT NULL`, `FK → products(id)` | — | Referenced product |
| `product_name` | `VARCHAR(200)` | — | `NULL` | Denormalized product name |
| `quantity` | `INTEGER` | `NOT NULL` | — | Units sold |
| `unit_price` | `NUMERIC(19,4)` | `NOT NULL` | — | Price per unit at time of sale |
| `total` | `NUMERIC(19,4)` | `NOT NULL` | — | Line total (quantity × unit_price) |

**Note:** `product_name` and `unit_price` are denormalized to preserve historical accuracy even if the product is later modified or deleted.

### 3.7 `repairs`

Repair orders (work orders) with full lifecycle tracking.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `UUID` | `PRIMARY KEY` | — | Unique identifier |
| `workshop_id` | `UUID` | `NOT NULL`, `FK → workshops(id)` | — | Owning workshop |
| `customer_name` | `VARCHAR(200)` | — | `NULL` | Customer name |
| `customer_email` | `VARCHAR(200)` | — | `NULL` | Customer email |
| `customer_phone` | `VARCHAR(20)` | — | `NULL` | Customer phone |
| `vehicle` | `VARCHAR(200)` | — | `NULL` | Vehicle description (make/model/year) |
| `license_plate` | `VARCHAR(50)` | — | `NULL` | Chilean license plate (PPU) |
| `description` | `TEXT` | — | `NULL` | Customer-reported issue |
| `diagnosis` | `TEXT` | — | `NULL` | Technician diagnosis |
| `technician_id` | `UUID` | `FK → users(id)` | `NULL` | Assigned mechanic |
| `estimated_delivery` | `DATE` | — | `NULL` | Expected completion date |
| `priority` | `priority` | `NOT NULL` | — | Urgency level |
| `status` | `repair_status` | `NOT NULL` | — | Current lifecycle state |
| `estimated_cost` | `NUMERIC(19,4)` | — | `NULL` | Pre-repair cost estimate |
| `final_cost` | `NUMERIC(19,4)` | — | `NULL` | Actual total charged |
| `labor_cost` | `NUMERIC(19,4)` | — | `NULL` | Labor component of final cost |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Creation timestamp |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Last modification timestamp |

### 3.8 `repair_updates`

Status change history for repairs. Acts as an audit trail per repair order.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `UUID` | `PRIMARY KEY` | — | Unique identifier |
| `repair_id` | `UUID` | `NOT NULL`, `FK → repairs(id) ON DELETE CASCADE` | — | Parent repair |
| `status` | `repair_status` | — | `NULL` | New status (null if description-only update) |
| `description` | `TEXT` | — | `NULL` | Free-text update note |
| `created_by` | `UUID` | `FK → users(id)` | `NULL` | User who made the update |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Timestamp of update |

### 3.9 `repair_parts`

Parts and materials consumed during repairs. Links to inventory products when applicable.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `UUID` | `PRIMARY KEY`, `DEFAULT gen_random_uuid()` | `gen_random_uuid()` | Unique identifier |
| `repair_id` | `UUID` | `NOT NULL`, `FK → repairs(id) ON DELETE CASCADE` | — | Parent repair |
| `name` | `VARCHAR(200)` | `NOT NULL` | — | Part description |
| `quantity` | `NUMERIC(10,2)` | `NOT NULL` | `1` | Quantity used (supports decimals for fluids) |
| `unit_cost` | `NUMERIC(19,4)` | — | `NULL` | Cost per unit |
| `total_cost` | `NUMERIC(19,4)` | — | `NULL` | Line total (quantity × unit_cost) |
| `product_id` | `UUID` | `FK → products(id) ON DELETE SET NULL` | `NULL` | Link to inventory product |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Creation timestamp |

### 3.10 `device_keys`

Machine-binding keys for license enforcement. Hashed via SHA-256.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `UUID` | `PRIMARY KEY`, `DEFAULT gen_random_uuid()` | `gen_random_uuid()` | Unique identifier |
| `key_hash` | `VARCHAR(64)` | `NOT NULL`, `UNIQUE` | — | SHA-256 hash of device key |
| `bound_ip` | `VARCHAR(45)` | — | `NULL` | IP address at binding time (IPv4 or IPv6) |
| `active` | `BOOLEAN` | `NOT NULL` | `TRUE` | Whether key is currently active |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Binding timestamp |
| `last_seen_at` | `TIMESTAMPTZ` | — | `NULL` | Last successful authentication |

### 3.11 `audit_log`

Immutable audit trail for all data mutations. Uses `BIGSERIAL` primary key (not UUID) for append-heavy workload.

| Column | Type | Constraints | Default | Description |
|--------|------|-------------|---------|-------------|
| `id` | `BIGSERIAL` | `PRIMARY KEY` | auto-increment | Sequential log ID |
| `user_id` | `UUID` | `FK → users(id)` | `NULL` | User who performed the action |
| `action` | `VARCHAR(100)` | `NOT NULL` | — | Action type (e.g., `create_product`, `update_repair`) |
| `entity_type` | `VARCHAR(100)` | — | `NULL` | Entity type (e.g., `product`, `sale`) |
| `entity_id` | `UUID` | — | `NULL` | ID of the affected entity |
| `old_values` | `JSONB` | — | `NULL` | Previous state (before mutation) |
| `new_values` | `JSONB` | — | `NULL` | New state (after mutation) |
| `ip_address` | `INET` | — | `NULL` | Client IP address |
| `user_agent` | `VARCHAR(255)` | — | `NULL` | Client user-agent string |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL` | `NOW()` | Event timestamp |

**Sensitive field redaction:** `password_hash`, `crypto_key`, and `jwt_secret` are replaced with `[REDACTED]` before storage.

---

## 4. Indexes

### 4.1 Primary Key Indexes

All primary keys are automatically indexed by PostgreSQL.

| Table | Column | Type |
|-------|--------|------|
| `workshops` | `id` | UUID |
| `users` | `id` | UUID |
| `suppliers` | `id` | UUID |
| `products` | `id` | UUID |
| `sales` | `id` | UUID |
| `sale_items` | `id` | UUID |
| `repairs` | `id` | UUID |
| `repair_updates` | `id` | UUID |
| `repair_parts` | `id` | UUID |
| `device_keys` | `id` | UUID |
| `audit_log` | `id` | BIGSERIAL |

### 4.2 Unique Constraint Indexes

| Index Name | Table | Columns | Type | Purpose |
|------------|-------|---------|------|---------|
| `users_email_key` | users | `email` | Global unique | Prevent duplicate accounts |
| `device_keys_key_hash_key` | device_keys | `key_hash` | Global unique | Prevent duplicate device registrations |
| `idx_products_workshop_barcode` | products | `(workshop_id, barcode)` | Partial unique | Per-workshop barcode uniqueness, excluding soft-deleted |
| `idx_products_workshop_sku` | products | `(workshop_id, sku)` | Partial unique | Per-workshop SKU uniqueness, excluding soft-deleted |

### 4.3 Foreign Key Indexes

| Index Name | Table | Column | Purpose |
|------------|-------|--------|---------|
| `idx_users_workshop_id` | users | `workshop_id` | Fast user lookup by workshop |
| `idx_suppliers_workshop_id` | suppliers | `workshop_id` | Fast supplier listing per workshop |
| `idx_sales_workshop_id` | sales | `workshop_id` | Fast sales query by workshop |
| `idx_repairs_workshop_id` | repairs | `workshop_id` | Fast repair listing per workshop |
| `idx_repair_parts_repair_id` | repair_parts | `repair_id` | Fast parts lookup per repair |

### 4.4 Query Optimization Indexes

| Index Name | Table | Column(s) | Purpose |
|------------|-------|-----------|---------|
| `idx_products_name` | products | `name` | Product name search |
| `idx_products_sku` | products | `sku` | SKU lookup |
| `idx_sales_created_at` | sales | `created_at` | Date-range sales queries |
| `idx_repairs_status` | repairs | `status` | Filter by repair status |
| `idx_repairs_license_plate` | repairs | `license_plate` | Vehicle lookup by plate |
| `idx_repairs_created_at` | repairs | `created_at` | Date-range repair queries |
| `idx_repair_updates_repair_id` | repair_updates | `repair_id` | Timeline per repair |
| `idx_repair_parts_product_id` | repair_parts | `product_id` | Link parts to inventory |
| `idx_audit_log_user_id` | audit_log | `user_id` | Audit queries per user |
| `idx_audit_log_entity` | audit_log | `(entity_type, entity_id)` | Audit history per entity |
| `idx_audit_log_created_at` | audit_log | `created_at` | Date-range audit queries |

---

## 5. Foreign Key Relationships

### 5.1 Relationship Map

```
workshops
├── users.workshop_id
├── products.workshop_id
├── sales.workshop_id
├── suppliers.workshop_id
└── repairs.workshop_id

users
├── repairs.technician_id
├── repair_updates.created_by
└── audit_log.user_id

suppliers
└── products.supplier_id

products
├── sale_items.product_id
├── repair_parts.product_id (ON DELETE SET NULL)
└── (unique indexes: workshop_barcode, workshop_sku)

sales
└── sale_items.sale_id (ON DELETE CASCADE)

repairs
├── repair_updates.repair_id (ON DELETE CASCADE)
└── repair_parts.repair_id (ON DELETE CASCADE)
```

### 5.2 Cascade Rules

| Parent | Child | Rule | Effect |
|--------|-------|------|--------|
| `sales` | `sale_items` | `ON DELETE CASCADE` | Deleting a sale removes its line items |
| `repairs` | `repair_updates` | `ON DELETE CASCADE` | Deleting a repair removes its history |
| `repairs` | `repair_parts` | `ON DELETE CASCADE` | Deleting a repair removes its parts |
| `products` | `repair_parts` | `ON DELETE SET NULL` | Deleting a product unlinks it from parts (preserves part records) |

### 5.3 Optional vs. Required FKs

| FK Column | Parent | Required | Rationale |
|-----------|--------|----------|-----------|
| `users.workshop_id` | `workshops` | **Required** | Every user must belong to a workshop |
| `products.workshop_id` | `workshops` | **Required** | Products are multi-tenant scoped |
| `sales.workshop_id` | `workshops` | **Required** | Sales are multi-tenant scoped |
| `suppliers.workshop_id` | `workshops` | **Required** | Suppliers are multi-tenant scoped |
| `repairs.workshop_id` | `workshops` | **Required** | Repairs are multi-tenant scoped |
| `products.supplier_id` | `suppliers` | Optional | Not all products have a supplier |
| `products.barcode` | — | Optional | Not all products use barcodes |
| `products.sku` | — | Optional | Not all products use SKUs |
| `repairs.technician_id` | `users` | Optional | May be unassigned |
| `repairs.license_plate` | — | Optional | Not all repairs involve vehicles |
| `sale_items.product_id` | `products` | **Required** | Every line item references a product |
| `repair_parts.product_id` | `products` | Optional | Parts can exist without inventory link |
| `audit_log.user_id` | `users` | Optional | System-generated events may lack a user |

---

## 6. Data Type Rationale

### 6.1 UUID Primary Keys

**Decision:** All entity tables use `UUID` (v4 random) as primary keys.

**Rationale:**
- No sequential ID guessing (security: prevents enumeration attacks)
- Client-generated IDs enable offline-first creation without server round-trips
- Globally unique: no collision risk across workshops
- SQLx native support with `uuid::Uuid`

**Exception:** `audit_log` uses `BIGSERIAL` — append-only, no client generation needed, and sequential IDs are acceptable for log indexing.

### 6.2 NUMERIC(19,4) for Money

**Decision:** All monetary columns use `NUMERIC(19,4)` instead of `FLOAT` or `INTEGER`.

**Rationale:**
- **No floating-point rounding errors** — critical for financial calculations (19.99 + 0.01 ≠ 20.000000001)
- **19 integer digits** supports values up to ~999 trillion CLP (sufficient for any workshop transaction)
- **4 decimal places** accommodates fractional CLP during tax calculations (IVA 19% can produce sub-peso intermediates)
- **PostgreSQL `NUMERIC`** stores exact values with arbitrary precision — no overflow risk
- `rust_decimal::Decimal` in Rust mirrors this type exactly via `sqlx::Type`

### 6.3 JSONB for Audit Trail

**Decision:** `old_values` and `new_values` in `audit_log` use `JSONB` (not `TEXT` or `JSON`).

**Rationale:**
- **Binary storage** — faster than `JSON` text type, smaller on disk
- **Queryable** — can use `@>` containment operators for specific field searches
- **Schema-flexible** — different entity types produce different JSON structures
- **Indexable** — GIN indexes possible for high-volume audit queries
- **PostgreSQL-native** — no application-level serialization needed

### 6.4 VARCHAR Lengths

| Length | Usage | Rationale |
|--------|-------|-----------|
| `VARCHAR(50)` | Barcodes, license plates, status fields | EAN-13 is 13 chars; Chilean plates max 8 chars |
| `VARCHAR(100)` | SKUs, categories, brands, models | Sufficient for product identifiers |
| `VARCHAR(200)` | Names, emails, descriptions | Generous limit for business names |
| `VARCHAR(255)` | Password hashes, user-agent | Argon2id hashes fit; UA strings vary |
| `VARCHAR(300)` | Addresses | Multi-line addresses need space |
| `TEXT` | Descriptions, diagnoses, notes | Unlimited length for free-form content |

### 6.5 TIMESTAMPTZ vs. TIMESTAMP

All timestamp columns use `TIMESTAMPTZ` (timestamp with time zone) to:
- Store timestamps in UTC internally
- Display in the user's local timezone on read
- Avoid ambiguity during daylight saving transitions

---

## 7. Multi-Tenant Data Isolation

All business tables (`users`, `products`, `sales`, `suppliers`, `repairs`) include a `workshop_id` foreign key. Every query is scoped by this column:

```sql
-- Example: all product queries include workshop filter
SELECT * FROM products WHERE workshop_id = $1 AND status != 'deleted';
```

**Enforcement points:**
1. **Application layer:** All route handlers extract `workshop_id` from JWT claims
2. **Index layer:** Partial unique indexes on `(workshop_id, barcode)` and `(workshop_id, sku)` ensure per-workshop uniqueness
3. **No cross-workshop joins:** Business logic never joins across workshops

---

## 8. Cross-References

| Document | Description |
|----------|-------------|
| [Migrations](./migrations.md) | Migration history and version tracking |
| [Index Strategy](./indexes.md) | Detailed index catalog and performance analysis |
| [ER Diagram](./er-diagram.md) | Visual entity-relationship diagram |
| [Architecture Overview](../01-architecture/overview.md) | System design and technology choices |
