# Entity-Relationship Diagram — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, DBAs, technical stakeholders

---

## 1. Overview Diagram

```
                              ┌─────────────────────────┐
                              │        workshops         │
                              │─────────────────────────│
                              │ PK id          UUID      │
                              │    name        VARCHAR   │
                              │    address     VARCHAR   │
                              │    city        VARCHAR   │
                              │    barcode_prefix VARCHAR │
                              │    created_at  TIMESTAMPTZ│
                              │    updated_at  TIMESTAMPTZ│
                              └────────────┬────────────┘
                                           │
              ┌────────────────────────────┼────────────────────────────┐
              │                            │                            │
              ▼ 1:N                        ▼ 1:N                        ▼ 1:N
┌─────────────────────────┐   ┌─────────────────────────┐   ┌─────────────────────────┐
│         users           │   │        products          │   │        suppliers        │
│─────────────────────────│   │─────────────────────────│   │─────────────────────────│
│ PK id          UUID     │   │ PK id          UUID      │   │ PK id          UUID      │
│    email       VARCHAR  │   │ FK workshop_id UUID      │   │ FK workshop_id UUID      │
│    display_name VARCHAR │   │    name        VARCHAR   │   │    name        VARCHAR   │
│    password_hash VARCHAR│   │    description TEXT      │   │    contact_person VARCHAR│
│    role        ENUM     │   │    category    VARCHAR   │   │    email       VARCHAR   │
│    status      VARCHAR  │   │    brand       VARCHAR   │   │    phone       VARCHAR   │
│ FK workshop_id UUID     │   │    model       VARCHAR   │   │    address     TEXT      │
│    created_at  TIMESTAMPTZ│  │    sku         VARCHAR   │   │    tax_id      VARCHAR   │
└────────────┬────────────┘   │    barcode     VARCHAR   │   │    payment_terms VARCHAR │
             │                │    price       NUMERIC   │   │    status      VARCHAR   │
             │                │    cost        NUMERIC   │   │    created_at  TIMESTAMPTZ│
             │                │    stock       INTEGER   │   │    updated_at  TIMESTAMPTZ│
             │                │    min_stock   INTEGER   │   └────────────┬────────────┘
             │                │    location    VARCHAR   │                │
             │                │ FK supplier_id UUID      │                │
             │                │    status      VARCHAR   │                │
             │                │    created_at  TIMESTAMPTZ│               │
             │                │    updated_at  TIMESTAMPTZ│               │
             │                └────────────┬────────────┘                │
             │                             │                             │
             │              ┌──────────────┼──────────────┐              │
             │              │              │              │              │
             │              ▼ 1:N          ▼ 1:N          ▼ N:1         │
             │  ┌───────────────────┐ ┌──────────────┐ ┌───────────────┘
             │  │    sale_items     │ │ repair_parts │ │
             │  │───────────────────│ │──────────────│ │
             │  │ PK id      UUID   │ │ PK id  UUID  │ │
             │  │ FK sale_id UUID   │ │ FK repair_id │ │
             │  │ FK product_id UUID│ │ FK product_id│──┘
             │  │    product_name   │ │    name      │
             │  │    quantity INT   │ │    quantity  │
             │  │    unit_price NUM │ │    unit_cost │
             │  │    total    NUM   │ │    total_cost│
             │  └────────┬──────────┘ │    created_at│
             │           │            └──────┬───────┘
             │           │                   │
             │           ▼ N:1               ▼ N:1
             │  ┌───────────────────┐ ┌──────────────────┐
             │  │       sales       │ │     repairs      │
             │  │───────────────────│ │──────────────────│
             │  │ PK id      UUID   │ │ PK id      UUID  │
             │  │ FK workshop_id    │ │ FK workshop_id   │
             │  │    customer_name  │ │ FK technician_id──┼──► users
             │  │    customer_email │ │    customer_name │
             │  │    customer_phone │ │    customer_email│
             │  │    subtotal  NUM  │ │    vehicle       │
             │  │    discount  NUM  │ │    license_plate │
             │  │    taxable   NUM  │ │    description   │
             │  │    tax       NUM  │ │    diagnosis     │
             │  │    total     NUM  │ │    estimated_del │
             │  │    payment_method │ │    priority  ENUM│
             │  │    status    VARCHAR│  │    status   ENUM│
             │  │    created_at TIM │ │    estimated_cost│
             │  └───────────────────┘ │    final_cost    │
             │                        │    labor_cost    │
             │                        │    created_at    │
             │                        │    updated_at    │
             │                        └────────┬─────────┘
             │                                 │
             │                                 ▼ 1:N
             │                        ┌──────────────────┐
             │                        │  repair_updates  │
             │                        │──────────────────│
             │                        │ PK id      UUID  │
             │                        │ FK repair_id UUID│
             │                        │ FK created_by UUID┼──► users
             │                        │    status   ENUM │
             │                        │    description  │
             │                        │    created_at   │
             │                        └─────────────────┘
             │
             ├──► repairs.technician_id (FK, optional)
             ├──► repair_updates.created_by (FK, optional)
             └──► audit_log.user_id (FK, optional)

┌─────────────────────────┐      ┌─────────────────────────┐
│      device_keys        │      │       audit_log          │
│─────────────────────────│      │─────────────────────────│
│ PK id          UUID     │      │ PK id          BIGSERIAL│
│    key_hash    VARCHAR  │      │ FK user_id     UUID      │
│    bound_ip    VARCHAR  │      │    action      VARCHAR   │
│    active      BOOLEAN  │      │    entity_type VARCHAR   │
│    created_at  TIMESTAMPTZ│    │    entity_id   UUID      │
│    last_seen_at TIMESTAMPTZ│   │    old_values  JSONB    │
└─────────────────────────┘      │    new_values  JSONB     │
                                 │    ip_address  INET      │
                                 │    user_agent  VARCHAR   │
                                 │    created_at  TIMESTAMPTZ│
                                 └─────────────────────────┘
```

---

## 2. Cardinality Summary

| Parent | Child | Cardinality | FK Column | Cascade | Required |
|--------|-------|-------------|-----------|---------|----------|
| `workshops` | `users` | 1 : N | `users.workshop_id` | RESTRICT | Yes |
| `workshops` | `products` | 1 : N | `products.workshop_id` | RESTRICT | Yes |
| `workshops` | `sales` | 1 : N | `sales.workshop_id` | RESTRICT | Yes |
| `workshops` | `suppliers` | 1 : N | `suppliers.workshop_id` | RESTRICT | Yes |
| `workshops` | `repairs` | 1 : N | `repairs.workshop_id` | RESTRICT | Yes |
| `sales` | `sale_items` | 1 : N | `sale_items.sale_id` | CASCADE | Yes |
| `repairs` | `repair_updates` | 1 : N | `repair_updates.repair_id` | CASCADE | Yes |
| `repairs` | `repair_parts` | 1 : N | `repair_parts.repair_id` | CASCADE | Yes |
| `products` | `sale_items` | 1 : N | `sale_items.product_id` | RESTRICT | Yes |
| `products` | `repair_parts` | 1 : N | `repair_parts.product_id` | SET NULL | No |
| `suppliers` | `products` | 1 : N | `products.supplier_id` | RESTRICT | No |
| `users` | `repairs` | 1 : N | `repairs.technician_id` | RESTRICT | No |
| `users` | `repair_updates` | 1 : N | `repair_updates.created_by` | RESTRICT | No |
| `users` | `audit_log` | 1 : N | `audit_log.user_id` | RESTRICT | No |

---

## 3. Cascade Rules

### 3.1 ON DELETE CASCADE

When a parent row is deleted, all child rows are automatically removed.

| Parent | Child | Effect |
|--------|-------|--------|
| `sales` | `sale_items` | Deleting a sale removes all its line items |
| `repairs` | `repair_updates` | Deleting a repair removes its status history |
| `repairs` | `repair_parts` | Deleting a repair removes its parts list |

**Usage pattern:** Dependent data that has no meaning without the parent.

### 3.2 ON DELETE SET NULL

When a parent row is deleted, the child's FK column is set to `NULL`.

| Parent | Child | Effect |
|--------|-------|--------|
| `products` | `repair_parts.product_id` | Deleting a product unlinks it from repair parts |

**Usage pattern:** The child record should persist (as a historical record) but the FK reference is no longer valid.

### 3.3 ON DELETE RESTRICT (Implicit)

Most foreign keys use the default `RESTRICT` behavior — deletion of a parent with existing children is blocked.

| Parent | Protected By | Effect |
|--------|-------------|--------|
| `workshops` | All business tables | Cannot delete a workshop with data |
| `products` | `sale_items` | Cannot delete a product that has been sold |
| `suppliers` | `products` | Cannot delete a supplier with linked products |
| `users` | `repairs`, `repair_updates`, `audit_log` | Cannot delete a user with history |

---

## 4. Optional vs. Required Relationships

### 4.1 Required Foreign Keys (NOT NULL)

| FK Column | Parent | Every child MUST have |
|-----------|--------|----------------------|
| `users.workshop_id` | `workshops` | Yes — users are multi-tenant scoped |
| `products.workshop_id` | `workshops` | Yes — products are multi-tenant scoped |
| `sales.workshop_id` | `workshops` | Yes — sales are multi-tenant scoped |
| `suppliers.workshop_id` | `workshops` | Yes — suppliers are multi-tenant scoped |
| `repairs.workshop_id` | `workshops` | Yes — repairs are multi-tenant scoped |
| `sale_items.sale_id` | `sales` | Yes — line items belong to a sale |
| `sale_items.product_id` | `products` | Yes — line items reference a product |
| `repair_updates.repair_id` | `repairs` | Yes — updates belong to a repair |
| `repair_parts.repair_id` | `repairs` | Yes — parts belong to a repair |

### 4.2 Optional Foreign Keys (NULLable)

| FK Column | Parent | Can be NULL | Rationale |
|-----------|--------|-------------|-----------|
| `products.supplier_id` | `suppliers` | Yes | Not all products have a known supplier |
| `repairs.technician_id` | `users` | Yes | Repair may be unassigned initially |
| `repair_updates.created_by` | `users` | Yes | System-generated updates may lack a user |
| `repair_parts.product_id` | `products` | Yes | Parts may not link to inventory (ad-hoc purchases) |
| `audit_log.user_id` | `users` | Yes | System events may lack a user context |
| `products.barcode` | — | Yes | Not all products use barcodes |
| `products.sku` | — | Yes | Not all products use SKUs |
| `repairs.license_plate` | — | Yes | Not all repairs involve vehicles |
| `repairs.estimated_delivery` | — | Yes | May not have a delivery estimate |
| `repairs.estimated_cost` | — | Yes | May not have a cost estimate yet |
| `repairs.final_cost` | — | Yes | Only set when repair is completed |
| `repairs.labor_cost` | — | Yes | Only set when labor is tracked |
| `repair_parts.unit_cost` | — | Yes | Cost may be unknown at time of entry |
| `repair_parts.total_cost` | — | Yes | Derived from quantity × unit_cost |
| `device_keys.bound_ip` | — | Yes | May not have IP at creation time |
| `device_keys.last_seen_at` | — | Yes | Only set after first authentication |

---

## 5. Entity Grouping

### 5.1 Multi-Tenant Core (workshop_id scoped)

```
workshops ─── users
          ├── products ─── sale_items
          ├── sales ─── sale_items
          ├── suppliers ─── products
          └── repairs ─── repair_updates
                       └── repair_parts
```

### 5.2 System Tables (global scope)

```
device_keys    (machine binding, no workshop FK)
audit_log      (global audit trail, references users)
```

---

## 6. Multi-Tenant Isolation Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        workshops (id = W1)                       │
│─────────────────────────────────────────────────────────────────│
│                                                                  │
│  users          products       sales        suppliers   repairs  │
│  ┌─────┐       ┌─────┐       ┌─────┐      ┌─────┐    ┌─────┐  │
│  │U1   │       │P1   │       │S1   │      │V1   │    │R1   │  │
│  │work=W1│     │work=W1│     │work=W1│    │work=W1│  │work=W1│ │
│  └─────┘       └─────┘       └─────┘      └─────┘    └─────┘  │
│                                                                  │
│  All queries: WHERE workshop_id = W1                             │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                        workshops (id = W2)                       │
│─────────────────────────────────────────────────────────────────│
│                                                                  │
│  users          products       sales        suppliers   repairs  │
│  ┌─────┐       ┌─────┐       ┌─────┐      ┌─────┐    ┌─────┐  │
│  │U2   │       │P2   │       │S2   │      │V2   │    │R2   │  │
│  │work=W2│     │work=W2│     │work=W2│    │work=W2│  │work=W2│ │
│  └─────┘       └─────┘       └─────┘      └─────┘    └─────┘  │
│                                                                  │
│  All queries: WHERE workshop_id = W2                             │
└─────────────────────────────────────────────────────────────────┘

device_keys and audit_log are NOT scoped by workshop_id.
audit_log references users (which have workshop_id).
```

---

## 7. Cross-References

| Document | Description |
|----------|-------------|
| [Schema Reference](./schema.md) | Complete table and column definitions |
| [Migrations](./migrations.md) | Migration history |
| [Index Strategy](./indexes.md) | Index catalog and performance analysis |
| [Architecture Overview](../01-architecture/overview.md) | System design and technology choices |
