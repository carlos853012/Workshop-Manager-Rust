# Audit Trail — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, security engineers, auditors

---

## 1. Audit Log Schema

### 1.1 Table Definition

```sql
CREATE TABLE audit_log (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID,
    action      VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id   UUID NOT NULL,
    old_values  JSONB,
    new_values  JSONB,
    ip_address  INET,
    user_agent  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 1.2 Column Descriptions

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `id` | UUID | No | Primary key (auto-generated) |
| `user_id` | UUID | Yes | ID of the user who performed the action |
| `action` | VARCHAR(50) | No | Action type (e.g., `CREATE`, `UPDATE`, `DELETE`) |
| `entity_type` | VARCHAR(50) | No | Entity type (e.g., `product`, `sale`, `repair`) |
| `entity_id` | UUID | No | ID of the affected entity |
| `old_values` | JSONB | Yes | Previous state of the entity (before change) |
| `new_values` | JSONB | Yes | New state of the entity (after change) |
| `ip_address` | INET | Yes | Client IP address |
| `user_agent` | TEXT | Yes | Client user agent string |
| `created_at` | TIMESTAMPTZ | No | Timestamp of the audit event |

---

## 2. What Gets Logged

### 2.1 CRUD Operations

| Operation | `action` | `old_values` | `new_values` | Logged When |
|-----------|----------|--------------|--------------|-------------|
| Create | `CREATE` | `null` | Full new entity | Entity created |
| Read | — | — | — | **Not logged** (reads are not audited) |
| Update | `UPDATE` | Previous state | Updated state | Entity modified |
| Delete | `DELETE` | Full entity | `null` | Entity deleted |

### 2.2 Logged Entities

| Entity Type | `entity_type` | Examples |
|------------|---------------|----------|
| Products | `product` | Create, update, delete products |
| Sales | `sale` | Create, update, delete sales |
| Sale Items | `sale_item` | Add/remove items from sales |
| Repairs | `repair` | Create, update status, delete repairs |
| Repair Updates | `repair_update` | Add status updates to repairs |
| Suppliers | `supplier` | Create, update, delete suppliers |
| Users | `user` | Create, update, delete users |
| Workshops | `workshop` | Update workshop settings |
| Device Keys | `device_key` | Create, revoke device keys |

### 2.3 Audit Function Signature

**Source:** `crates/inventory-server/src/audit.rs:7-44`

```rust
pub async fn log_change(
    pool: &PgPool,
    user_id: Option<Uuid>,        // None for system actions
    action: &str,                 // "CREATE", "UPDATE", "DELETE"
    entity_type: &str,            // "product", "sale", etc.
    entity_id: Uuid,              // ID of affected entity
    old_values: Option<Value>,    // Previous state (JSONB)
    new_values: Option<Value>,    // New state (JSONB)
    ip_address: Option<&str>,     // Client IP
    user_agent: Option<&str>,     // Client user agent
) -> anyhow::Result<()>
```

---

## 3. What Gets Redacted

### 3.1 Redaction Rules

**Source:** `crates/inventory-server/src/audit.rs:47-55`

| Field Name | Redacted Value | Reason |
|-----------|---------------|--------|
| `password_hash` | `[REDACTED]` | Cryptographic material |
| `password` | `[REDACTED]` | User credential |
| `crypto_key` | `[REDACTED]` | Encryption key |
| `jwt_secret` | `[REDACTED]` | Signing secret |

### 3.2 Redaction Implementation

```rust
pub fn redact_sensitive(value: &mut Value) {
    if let Value::Object(map) = value {
        for key in ["password_hash", "password", "crypto_key", "jwt_secret"] {
            if map.contains_key(key) {
                map.insert(key.to_string(), Value::String("[REDACTED]".to_string()));
            }
        }
    }
}
```

### 3.3 Redaction Scope

| Context | Redacted? | Notes |
|---------|-----------|-------|
| `old_values` JSONB | Yes | Sensitive fields replaced with `[REDACTED]` |
| `new_values` JSONB | Yes | Sensitive fields replaced with `[REDACTED]` |
| SQL query logs | No | Parameterized queries, no sensitive data in logs |
| Tracing output | No | Only entity_type, action, entity_id logged |

---

## 4. Audit Log Query Examples

### 4.1 Recent Activity for a User

```sql
SELECT
    al.created_at,
    al.action,
    al.entity_type,
    al.entity_id,
    al.new_values->>'name' AS entity_name
FROM audit_log al
WHERE al.user_id = '550e8400-e29b-41d4-a716-446655440000'
ORDER BY al.created_at DESC
LIMIT 50;
```

### 4.2 All Changes to a Specific Product

```sql
SELECT
    al.created_at,
    al.action,
    al.old_values,
    al.new_values
FROM audit_log al
WHERE al.entity_type = 'product'
  AND al.entity_id = 'some-product-uuid'
ORDER BY al.created_at DESC;
```

### 4.3 Deletion History (Last 30 Days)

```sql
SELECT
    al.created_at,
    al.user_id,
    al.entity_type,
    al.entity_id,
    al.old_values->>'name' AS deleted_name
FROM audit_log al
WHERE al.action = 'DELETE'
  AND al.created_at > NOW() - INTERVAL '30 days'
ORDER BY al.created_at DESC;
```

### 4.4 Failed Login Attempts (via Rate Limiter)

```sql
-- Note: Rate limiter is in-memory, not in audit_log
-- Failed logins are recorded via rate_limiter.record_attempt()
-- and traced in server logs
```

### 4.5 Entity Change Diff

```sql
SELECT
    al.created_at,
    al.action,
    jsonb_pretty(al.old_values) AS before,
    jsonb_pretty(al.new_values) AS after
FROM audit_log al
WHERE al.entity_id = 'some-entity-uuid'
  AND al.action = 'UPDATE'
ORDER BY al.created_at DESC
LIMIT 5;
```

---

## 5. Retention Policy

### 5.1 Current Policy

| Aspect | Value |
|--------|-------|
| Retention period | **Indefinite** (no automatic pruning) |
| Storage format | PostgreSQL JSONB |
| Compression | None (native PostgreSQL storage) |
| Backup | Included in encrypted database backups |

### 5.2 Recommended Retention

| Timeframe | Recommendation |
|-----------|---------------|
| 0–90 days | Keep all records (active investigation window) |
| 90 days – 1 year | Archive to cold storage, compress |
| 1+ year | Consider deletion unless legally required |

### 5.3 Future Improvements

| ID | Improvement | Priority |
|----|------------|----------|
| A1 | Add configurable retention period | Medium |
| A2 | Implement automatic audit log pruning | Medium |
| A3 | Add audit log export (CSV/JSON) | Low |
| A4 | Add audit log statistics dashboard | Low |

---

## 6. Audit Trail Integrity

### 6.1 Integrity Properties

| Property | Implementation |
|----------|---------------|
| Append-only | Audit records are never updated or deleted |
| UUID primary key | Non-sequential, prevents enumeration |
| Timestamp | Server-generated (`NOW()`), not client-controlled |
| JSONB storage | Native PostgreSQL, supports complex nested data |
| Transaction-safe | Audit writes are part of the same transaction as the data change |

### 6.2 Audit Failure Handling

| Scenario | Behavior |
|----------|----------|
| Audit write fails | Operation still succeeds (best-effort logging) |
| Database down | Audit write fails silently, operation proceeds |
| Redaction fails | Original value logged (redaction is best-effort) |

**Note:** Audit failures do NOT block business operations. This is a deliberate design decision to avoid availability issues.

---

## 7. Integration with Incident Response

### 7.1 Incident Response Workflow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Incident    │────▶│  Query       │────▶│  Analyze     │────▶│  Respond     │
│  Detected    │     │  Audit Log   │     │  Changes     │     │              │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

### 7.2 Useful Queries for Incident Response

| Query | Purpose |
|-------|---------|
| All changes by user | Identify unauthorized modifications |
| All changes to entity | Track data tampering |
| Deletion history | Identify data destruction |
| Time-range analysis | Correlate with incident timeline |
| IP address tracking | Identify source of malicious requests |

### 7.3 Log Sources for Correlation

| Source | Information |
|--------|------------|
| `audit_log` table | CRUD operations with old/new values |
| Server tracing output | Request lifecycle, errors, timing |
| Rate limiter | Login attempt patterns |
| PostgreSQL logs | Query performance, connection issues |

---

## 8. Cross-References

| Document | Description |
|----------|-------------|
| [Overview](./overview.md) | Security architecture overview |
| [Hardening Checklist](./hardening.md) | Security hardening steps |
| [Database Schema](../03-database/schema.md) | Full database schema including audit_log |
| [Database ER Diagram](../03-database/er-diagram.md) | Entity relationships |
