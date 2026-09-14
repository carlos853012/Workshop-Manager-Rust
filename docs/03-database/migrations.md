# Database Migrations — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, DBAs

---

## 1. Migration Framework

| Property | Value |
|----------|-------|
| Tool | SQLx `migrate!()` macro |
| Location | `crates/workshop-server/migrations/` |
| Execution | Embedded in server binary, runs automatically at startup |
| Naming | Sequential numeric prefix: `0001_description.sql` |
| Order | Lexicographic (numeric prefix ensures correct order) |
| Idempotency | SQLx tracks applied migrations in `_sqlx_migrations` table |

SQLx records each applied migration in a `_sqlx_migrations` table with a timestamp hash, preventing re-execution.

---

## 2. Migration History

### 2.1 `0001_initial_schema.sql`

**Purpose:** Foundation schema — all core tables, enums, and initial indexes.

**Changes:**
- Created 4 custom enum types: `payment_method`, `repair_status`, `priority`, `user_role`
- Created 7 tables: `suppliers`, `users`, `products`, `sales`, `sale_items`, `repairs`, `repair_updates`, `audit_log`
- Created initial indexes on `products.name`, `products.sku`, `sales.created_at`, `repairs.status`, `repairs.license_plate`, `repairs.created_at`, `audit_log.user_id`, `audit_log.(entity_type, entity_id)`, `audit_log.created_at`

**Rationale:** Establishes the complete domain model before multi-tenant support.

### 2.2 `0002_workshops.sql`

**Purpose:** Add multi-tenant support via the `workshops` table.

**Changes:**
- Created `workshops` table with `id`, `name`, `address`, `city`, timestamps
- Added `workshop_id` column to `users` (initially nullable)
- Inserted a default workshop for existing data
- Backfilled `workshop_id` for existing users
- Made `workshop_id` `NOT NULL` with foreign key constraint
- Created `idx_users_workshop_id` index

**Rationale:** Introduces multi-tenancy by adding a workshop entity and linking users to workshops.

### 2.3 `0003_rename_motorcycle_to_vehicle.sql`

**Purpose:** Rename column to support broader vehicle types.

**Changes:**
- `ALTER TABLE repairs RENAME COLUMN motorcycle TO vehicle`

**Rationale:** WorkshopManager supports all vehicles (cars, trucks, ATVs), not just motorcycles.

### 2.4 `0004_device_keys.sql`

**Purpose:** Add device-binding mechanism for license enforcement.

**Changes:**
- Created `device_keys` table with `id`, `key_hash` (SHA-256), `bound_ip`, `active`, `created_at`, `last_seen_at`

**Rationale:** Enables machine-specific licensing by binding to hardware fingerprints.

### 2.5 `0005_pos_and_barcode.sql`

**Purpose:** Extend schema for POS functionality, barcode support, and full multi-tenant scoping.

**Changes:**
- Added `workshop_id` to `products`, `sales`, `suppliers`, `repairs` (initially nullable)
- Added `barcode` column to `products`
- Created partial unique indexes: `idx_products_workshop_barcode`, `idx_products_workshop_sku`
- Added tax breakdown columns to `sales`: `subtotal`, `discount_amount`, `taxable_amount`, `tax_amount`
- Backfilled `workshop_id` for all tables from default workshop
- Made `workshop_id` `NOT NULL` on all four tables
- Created `idx_sales_workshop_id`, `idx_suppliers_workshop_id`, `idx_repairs_workshop_id`

**Rationale:** Largest migration — transforms the schema from single-tenant to fully multi-tenant, adds POS infrastructure, and enables barcode-based product lookup.

### 2.6 `0006_barcode_prefix.sql`

**Purpose:** Add barcode prefix configuration to workshops.

**Changes:**
- Added `barcode_prefix VARCHAR(6)` to `workshops`

**Rationale:** Each workshop can configure a short prefix used when generating EAN-13 barcodes for products.

### 2.7 `0007_repair_parts.sql`

**Purpose:** Track parts and materials consumed during repairs.

**Changes:**
- Created `repair_parts` table with `id`, `repair_id`, `name`, `quantity`, `unit_cost`, `total_cost`, `created_at`
- Created `idx_repair_parts_repair_id` index

**Rationale:** Enables accurate cost tracking for repairs by recording which parts were used and their costs.

### 2.8 `0008_fix_unique_indexes_soft_delete.sql`

**Purpose:** Fix unique indexes to exclude soft-deleted products.

**Changes:**
- Dropped `idx_products_workshop_barcode` and `idx_products_workshop_sku`
- Recreated with `WHERE barcode IS NOT NULL AND status != 'deleted'` clause

**Rationale:** Without the soft-delete exclusion, a deleted product's barcode/SKU would block reuse for a new product. The partial index allows reusing identifiers after soft-deletion.

### 2.9 `0009_repair_parts_product_fk.sql`

**Purpose:** Link repair parts to inventory products.

**Changes:**
- Added `product_id UUID` to `repair_parts` with `FK → products(id) ON DELETE SET NULL`
- Created `idx_repair_parts_product_id` index

**Rationale:** Enables linking repair parts to inventory products for automatic stock deduction and cost tracking. `ON DELETE SET NULL` preserves part records if a product is deleted.

### 2.10 `0010_add_labor_cost.sql`

**Purpose:** Add labor cost tracking to repairs.

**Changes:**
- Added `labor_cost NUMERIC(19,4)` to `repairs`

**Rationale:** Separates labor cost from parts cost, enabling accurate total repair cost calculation (labor + parts).

---

## 3. Migration Summary

| # | File | Tables Affected | Columns Added | Indexes Added |
|---|------|----------------|---------------|---------------|
| 1 | `0001_initial_schema` | 8 created | All initial columns | 11 |
| 2 | `0002_workshops` | 1 created | `users.workshop_id` | 1 |
| 3 | `0003_rename` | 1 modified | — (rename only) | — |
| 4 | `0004_device_keys` | 1 created | All initial columns | 0 (PK only) |
| 5 | `0005_pos_and_barcode` | 4 modified | 6 columns | 5 |
| 6 | `0006_barcode_prefix` | 1 modified | `workshops.barcode_prefix` | 0 |
| 7 | `0007_repair_parts` | 1 created | All initial columns | 1 |
| 8 | `0008_fix_unique_indexes` | 0 | 0 (recreated) | 2 (replaced) |
| 9 | `0009_repair_parts_fk` | 1 modified | `repair_parts.product_id` | 1 |
| 10 | `0010_add_labor_cost` | 1 modified | `repairs.labor_cost` | 0 |

**Totals:** 11 tables, 4 enums, ~20 indexes, 10 migrations

---

## 4. Naming Convention

```
NNNN_descriptive_name.sql
│    │
│    └─ Lowercase, underscores, descriptive
└──── 4-digit zero-padded sequence number
```

**Rules:**
- Sequence numbers are zero-padded to 4 digits for correct sorting
- Names describe the primary change (not the affected table)
- Use past-tense or noun form: `add_labor_cost`, `fix_unique_indexes`
- Special characters and uppercase avoided

---

## 5. Rollback Strategy

SQLx `migrate!()` does not provide automatic rollback. Rollbacks are manual.

### 5.1 Manual Rollback Procedure

```sql
-- 1. List applied migrations
SELECT version_name, applied_at FROM _sqlx_migrations ORDER BY version;

-- 2. Remove migration record (does NOT undo changes)
DELETE FROM _sqlx_migrations WHERE version = N;

-- 3. Manually reverse the migration
-- Example: undo migration 0010
ALTER TABLE repairs DROP COLUMN labor_cost;
```

### 5.2 Rollback Scripts (by Migration)

| Migration | Rollback Script |
|-----------|----------------|
| 0010 | `ALTER TABLE repairs DROP COLUMN labor_cost;` |
| 0009 | `ALTER TABLE repair_parts DROP COLUMN product_id; DROP INDEX idx_repair_parts_product_id;` |
| 0008 | Recreate indexes without soft-delete filter |
| 0007 | `DROP TABLE repair_parts;` |
| 0006 | `ALTER TABLE workshops DROP COLUMN barcode_prefix;` |
| 0005 | `ALTER TABLE products DROP COLUMN workshop_id; ...` (complex — data migration needed) |
| 0004 | `DROP TABLE device_keys;` |
| 0003 | `ALTER TABLE repairs RENAME COLUMN vehicle TO motorcycle;` |
| 0002 | `DROP TABLE workshops; ALTER TABLE users DROP COLUMN workshop_id;` |
| 0001 | `DROP TABLE ... CASCADE; DROP TYPE ...;` (drop all in reverse dependency order) |

### 5.3 Backup Before Migrations

Before applying any migration in production:

```powershell
# Create pre-migration backup
cargo build -p workshop-server
# Server auto-backups run daily; manual backup via pg_dump:
pg_dump -h 127.0.0.1 -p 5432 workshop_manager > pre_migration_backup.sql
```

---

## 6. Version Tracking

| Version | Applied | Notes |
|---------|---------|-------|
| 0001 | At first boot | Initial schema |
| 0002 | At first boot | Multi-tenant |
| 0003 | At first boot | Column rename |
| 0004 | At first boot | Device keys |
| 0005 | At first boot | POS + full multi-tenant |
| 0006 | At first boot | Barcode prefix |
| 0007 | At first boot | Repair parts |
| 0008 | At first boot | Fix unique indexes |
| 0009 | At first boot | Repair parts FK |
| 0010 | At first boot | Labor cost |

All 10 migrations execute on every fresh installation. No upgrade path exists yet (single-version product).

---

## 7. Cross-References

| Document | Description |
|----------|-------------|
| [Schema Reference](./schema.md) | Complete table and column definitions |
| [Index Strategy](./indexes.md) | Index catalog and performance analysis |
| [ER Diagram](./er-diagram.md) | Visual entity-relationship diagram |
| [System Design](../01-architecture/system-design.md) | High-level architecture |
