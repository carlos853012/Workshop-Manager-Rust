# Index Strategy — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, DBAs, performance engineers

---

## 1. Index Catalog

### 1.1 Primary Key Indexes (Auto-Created)

PostgreSQL automatically creates a B-tree index on every `PRIMARY KEY` column.

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

**Count:** 11 primary key indexes (not counted in totals below).

### 1.2 Explicit Indexes

#### Multi-Tenant Scoped Indexes

These indexes ensure per-workshop uniqueness and fast multi-tenant queries.

| Index Name | Table | Columns | Type | Purpose |
|------------|-------|---------|------|---------|
| `idx_products_workshop_barcode` | `products` | `(workshop_id, barcode)` | Partial unique | Per-workshop barcode uniqueness, excludes soft-deleted |
| `idx_products_workshop_sku` | `products` | `(workshop_id, sku)` | Partial unique | Per-workshop SKU uniqueness, excludes soft-deleted |
| `idx_users_workshop_id` | `users` | `workshop_id` | B-tree | Fast user lookup by workshop |
| `idx_suppliers_workshop_id` | `suppliers` | `workshop_id` | B-tree | Fast supplier listing per workshop |
| `idx_sales_workshop_id` | `sales` | `workshop_id` | B-tree | Fast sales query by workshop |
| `idx_repairs_workshop_id` | `repairs` | `workshop_id` | B-tree | Fast repair listing per workshop |

#### Query Optimization Indexes

| Index Name | Table | Column(s) | Type | Purpose |
|------------|-------|-----------|------|---------|
| `idx_products_name` | `products` | `name` | B-tree | Product name search |
| `idx_products_sku` | `products` | `sku` | B-tree | SKU lookup (global) |
| `idx_sales_created_at` | `sales` | `created_at` | B-tree | Date-range sales queries |
| `idx_repairs_status` | `repairs` | `status` | B-tree | Filter by repair status |
| `idx_repairs_license_plate` | `repairs` | `license_plate` | B-tree | Vehicle lookup by plate |
| `idx_repairs_created_at` | `repairs` | `created_at` | B-tree | Date-range repair queries |

#### Relationship Indexes

| Index Name | Table | Column | Type | Purpose |
|------------|-------|--------|------|---------|
| `idx_repair_updates_repair_id` | `repair_updates` | `repair_id` | B-tree | Timeline per repair |
| `idx_repair_parts_repair_id` | `repair_parts` | `repair_id` | B-tree | Parts lookup per repair |
| `idx_repair_parts_product_id` | `repair_parts` | `product_id` | B-tree | Link parts to inventory |

#### Audit Indexes

| Index Name | Table | Column(s) | Type | Purpose |
|------------|-------|-----------|------|---------|
| `idx_audit_log_user_id` | `audit_log` | `user_id` | B-tree | Audit queries per user |
| `idx_audit_log_entity` | `audit_log` | `(entity_type, entity_id)` | B-tree | Audit history per entity |
| `idx_audit_log_created_at` | `audit_log` | `created_at` | B-tree | Date-range audit queries |

#### Unique Constraint Indexes (Non-PK)

| Index Name | Table | Column | Type | Purpose |
|------------|-------|--------|------|---------|
| `users_email_key` | `users` | `email` | Global unique | Prevent duplicate accounts |
| `device_keys_key_hash_key` | `device_keys` | `key_hash` | Global unique | Prevent duplicate device registrations |

---

## 2. Partial Indexes

### 2.1 Soft-Delete Exclusion

```sql
-- idx_products_workshop_barcode
CREATE UNIQUE INDEX idx_products_workshop_barcode ON products(workshop_id, barcode)
    WHERE barcode IS NOT NULL AND status != 'deleted';

-- idx_products_workshop_sku
CREATE UNIQUE INDEX idx_products_workshop_sku ON products(workshop_id, sku)
    WHERE sku IS NOT NULL AND status != 'deleted';
```

**Why:** Allows barcode/SKU reuse after soft-deletion. Without the `status != 'deleted'` filter, a deleted product would block a new product from using the same barcode within the same workshop.

**Behavior:**
- Soft-deleted products (`status = 'deleted'`) are excluded from uniqueness checks
- `NULL` barcodes/SKUs are excluded (not all products use them)
- Active products enforce per-workshop uniqueness on these columns

### 2.2 Index Size Impact

Partial indexes are smaller than full indexes because they only index qualifying rows:

| Index | Approximate Size | Notes |
|-------|-----------------|-------|
| `idx_products_workshop_barcode` | ~5% of table | Only non-deleted products with barcodes |
| `idx_products_workshop_sku` | ~5% of table | Only non-deleted products with SKUs |

---

## 3. Unique Constraints

| Constraint | Table | Column | Scope | Effect |
|------------|-------|--------|-------|--------|
| Primary key | All 11 tables | `id` | Global | Prevents duplicate entity IDs |
| `UNIQUE` | `users` | `email` | Global | One account per email across all workshops |
| `UNIQUE` | `device_keys` | `key_hash` | Global | One registration per device key |
| Partial unique | `products` | `(workshop_id, barcode)` | Per-workshop | One barcode per product per workshop |
| Partial unique | `products` | `(workshop_id, sku)` | Per-workshop | One SKU per product per workshop |

**Multi-tenant uniqueness pattern:** The partial unique indexes on `products` enforce that within a single workshop, no two active products share the same barcode or SKU. Across workshops, the same barcode can exist in different workshops.

---

## 4. Performance Considerations

### 4.1 Index Selectivity

| Index | Selectivity | Assessment |
|-------|-------------|------------|
| `idx_products_workshop_barcode` | High | Near-unique per workshop — excellent for point lookups |
| `idx_products_workshop_sku` | High | Near-unique per workshop — excellent for point lookups |
| `users.email` | High | Global unique — one row per lookup |
| `idx_repairs_license_plate` | Medium | Multiple repairs per vehicle over time |
| `idx_repairs_status` | Low-Medium | Only 5 distinct values — good for small result sets |
| `idx_sales_created_at` | Low | Many sales per day — useful for range scans |
| `idx_audit_log_created_at` | Low | Append-only, many entries per day — range scans |

### 4.2 Multi-Tenant Query Patterns

Every business query includes `workshop_id` as the leading column. This design ensures:

1. **Index-driven filtering:** The database can use the `workshop_id` index to immediately narrow results to the relevant workshop
2. **Partition-like isolation:** Each workshop's data occupies a distinct segment of the B-tree
3. **No cross-tenant joins:** Business logic never joins across workshops

**Example query plan:**
```sql
EXPLAIN SELECT * FROM products WHERE workshop_id = $1 AND status != 'deleted';
-- Uses: idx_products_workshop_barcode or seq scan (depends on selectivity)
```

### 4.3 Composite vs. Single-Column Indexes

The `idx_audit_log_entity` index uses a composite key `(entity_type, entity_id)`:

```sql
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
```

**Effective for:**
- `WHERE entity_type = 'product' AND entity_id = $1` (full index usage)
- `WHERE entity_type = 'product'` (prefix usage)

**Not effective for:**
- `WHERE entity_id = $1` alone (column not in leading position)

### 4.4 Unused Indexes

The following indexes may be candidates for removal if query patterns confirm they're unused:

| Index | Table | Concern |
|-------|-------|---------|
| `idx_products_name` | products | Name search may be better served by full-text search |
| `idx_products_sku` | products | Redundant with `idx_products_workshop_sku` for most queries |

**Recommendation:** Monitor `pg_stat_user_indexes` before removing. Keep for now as safety net.

---

## 5. Monitoring Queries

### 5.1 List All Indexes

```sql
SELECT
    schemaname,
    tablename,
    indexname,
    indexdef
FROM pg_indexes
WHERE schemaname = 'public'
ORDER BY tablename, indexname;
```

### 5.2 Index Usage Statistics

```sql
SELECT
    schemaname,
    relname AS table_name,
    indexrelname AS index_name,
    idx_scan AS times_used,
    idx_tup_read AS tuples_read,
    idx_tup_fetch AS tuples_fetched,
    pg_size_pretty(pg_relation_size(indexrelid)) AS index_size
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;
```

### 5.3 Unused Indexes (Never Scanned)

```sql
SELECT
    indexrelname AS index_name,
    relname AS table_name,
    pg_size_pretty(pg_relation_size(indexrelid)) AS index_size
FROM pg_stat_user_indexes
WHERE idx_scan = 0
    AND indexrelname NOT LIKE '%_pkey'
ORDER BY pg_relation_size(indexrelid) DESC;
```

### 5.4 Index Size by Table

```sql
SELECT
    tablename,
    pg_size_pretty(pg_indexes_size('public.' || tablename)) AS total_index_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_indexes_size('public.' || tablename) DESC;
```

### 5.5 Missing Indexes (Sequential Scans on Large Tables)

```sql
SELECT
    relname AS table_name,
    seq_scan,
    seq_tup_read,
    idx_scan,
    n_live_tup AS row_count
FROM pg_stat_user_tables
WHERE seq_scan > 100
    AND n_live_tup > 1000
ORDER BY seq_tup_read DESC;
```

### 5.6 Partial Index Effectiveness

```sql
-- Check how many rows qualify vs. are excluded by partial index
SELECT
    status,
    COUNT(*) AS total,
    COUNT(barcode) AS with_barcode,
    COUNT(sku) AS with_sku
FROM products
GROUP BY status;
```

---

## 6. Index Maintenance

### 6.1 REINDEX

Rebuild indexes to reclaim space and update statistics:

```sql
-- Rebuild specific index
REINDEX INDEX idx_products_workshop_barcode;

-- Rebuild all indexes on a table
REINDEX TABLE products;

-- Full database reindex (offline operation)
REINDEX DATABASE workshop_manager;
```

### 6.2 VACUUM

PostgreSQL's autovacuum handles most maintenance. For heavy-write tables:

```sql
-- Manual vacuum for audit_log (append-heavy)
VACUUM ANALYZE audit_log;

-- Check vacuum status
SELECT
    relname,
    n_dead_tup,
    last_vacuum,
    last_autovacuum
FROM pg_stat_user_tables
WHERE n_dead_tup > 1000;
```

### 6.3 Statistics Update

```sql
-- Update planner statistics
ANALYZE products;
ANALYZE repairs;
ANALYZE audit_log;
```

---

## 7. Cross-References

| Document | Description |
|----------|-------------|
| [Schema Reference](./schema.md) | Complete table and column definitions |
| [Migrations](./migrations.md) | Migration history |
| [ER Diagram](./er-diagram.md) | Visual entity-relationship diagram |
