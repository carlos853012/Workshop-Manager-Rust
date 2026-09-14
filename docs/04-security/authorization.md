# Authorization Model — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, security engineers, auditors

---

## 1. RBAC Roles and Permissions

### 1.1 Role Definitions

| Role | Description | Typical User |
|------|-------------|-------------|
| `admin` | Full system access, user management, device key management | Workshop owner |
| `mechanic` | Read/write products, repairs, sales; read suppliers/analytics | Workshop mechanic |
| `seller` | Read/write products, sales; read repairs, suppliers, analytics | POS operator |

### 1.2 Permissions Matrix

| Resource | Operation | Admin | Mechanic | Seller |
|----------|-----------|-------|----------|--------|
| **Products** | | | | |
| Products | List/Read | ✅ | ✅ | ✅ |
| Products | Create | ✅ | ✅ | ✅ |
| Products | Update | ✅ | ✅ | ✅ |
| Products | Delete | ✅ | ✅ | ✅ |
| **Sales** | | | | |
| Sales | List/Read | ✅ | ✅ | ✅ |
| Sales | Create | ✅ | ✅ | ✅ |
| Sales | Update | ✅ | ✅ | ✅ |
| Sales | Delete | ✅ | ✅ | ✅ |
| **Repairs** | | | | |
| Repairs | List/Read | ✅ | ✅ | ✅ |
| Repairs | Create | ✅ | ✅ | ❌ |
| Repairs | Update | ✅ | ✅ | ❌ |
| Repairs | Delete | ✅ | ✅ | ❌ |
| **Suppliers** | | | | |
| Suppliers | List/Read | ✅ | ✅ | ✅ |
| Suppliers | Create | ✅ | ✅ | ✅ |
| Suppliers | Update | ✅ | ✅ | ✅ |
| Suppliers | Delete | ✅ | ✅ | ✅ |
| **Analytics** | | | | |
| Analytics | Read | ✅ | ✅ | ✅ |
| **Reports** | | | | |
| Reports | Read | ✅ | ✅ | ✅ |
| **Auth** | | | | |
| Auth Status | Read | ✅ | ✅ | ✅ |
| **Users** | | | | |
| Users | List | ✅ | ❌ | ❌ |
| Users | Create | ✅ | ❌ | ❌ |
| Users | Update | ✅ | ❌ | ❌ |
| Users | Delete | ✅ | ❌ | ❌ |
| **Device Keys** | | | | |
| Device Keys | List | ✅ | ❌ | ❌ |
| Device Keys | Create | ✅ | ❌ | ❌ |
| Device Keys | Revoke | ✅ | ❌ | ❌ |

---

## 2. Middleware Stack

### 2.1 Middleware Pipeline

All API requests pass through the middleware stack in order. Each layer can reject the request with an appropriate HTTP error.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         REQUEST LIFECYCLE                                │
│                                                                         │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐│
│  │  CORS   │──▶│  HSTS   │──▶│ Headers │──▶│  Body   │──▶│  Trace  ││
│  │         │   │         │   │         │   │  Limit  │   │         ││
│  │ Origin  │   │ max-age │   │ nosniff │   │  10MB   │   │ logging ││
│  │ check   │   │ 31536000│   │ DENY    │   │         │   │         ││
│  └─────────┘   └─────────┘   └─────────┘   └─────────┘   └─────────┘│
│                                                                         │
│  PUBLIC ROUTES (/api/auth/login, /api/auth/register)                    │
│  ───────────────────────────────────────────────────────────────────────│
│                                                                         │
│  PROTECTED ROUTES (/api/products, /api/sales, etc.)                     │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐                              │
│  │ API Key │──▶│ Device  │──▶│  JWT    │──▶ Route Handler              │
│  │ Check   │   │ Key     │   │  Auth   │                              │
│  │         │   │ Check   │   │         │                              │
│  └─────────┘   └─────────┘   └─────────┘                              │
│                                                                         │
│  ADMIN ROUTES (/api/users, /api/device-keys)                            │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐               │
│  │ API Key │──▶│ Device  │──▶│  JWT    │──▶│ Admin   │──▶ Handler     │
│  │ Check   │   │ Key     │   │  Auth   │   │ Check   │               │
│  └─────────┘   └─────────┘   └─────────┘   └─────────┘               │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Middleware Descriptions

| Middleware | File | Function | Error Code |
|-----------|------|----------|------------|
| CORS | `main.rs:125` | Validates request origin (localhost only) | 403 |
| HSTS | `main.rs:143` | Adds Strict-Transport-Security header | — |
| Headers | `main.rs:147` | Adds X-Content-Type-Options, X-Frame-Options | — |
| Body Limit | `main.rs:123` | Rejects requests > 10MB | 413 |
| API Key | `middleware.rs:48` | Validates `X-WorkshopManager-Key` header | 401 |
| Device Key | `device_key.rs:23` | Validates `X-WorkshopManager-Device-Key` header | 401 |
| JWT Auth | `middleware.rs:65` | Validates `Authorization: Bearer <token>` | 401 |
| Admin Check | `middleware.rs:81` | Requires `role == admin` | 403 |

---

## 3. Endpoint Authorization Table

### 3.1 Public Endpoints (No Authentication)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/auth/login` | User login |
| POST | `/api/auth/register` | Initial admin registration (first user only) |
| GET | `/health` | Health check |

### 3.2 Protected Endpoints (API Key + Device Key + JWT)

| Method | Path | Description | Roles |
|--------|------|-------------|-------|
| GET | `/api/auth/status` | Current user profile | All |
| GET | `/api/products` | List products | All |
| POST | `/api/products` | Create product | All |
| PUT | `/api/products/:id` | Update product | All |
| DELETE | `/api/products/:id` | Delete product | All |
| GET | `/api/sales` | List sales | All |
| POST | `/api/sales` | Create sale | All |
| PUT | `/api/sales/:id` | Update sale | All |
| DELETE | `/api/sales/:id` | Delete sale | All |
| GET | `/api/repairs` | List repairs | All |
| POST | `/api/repairs` | Create repair | Admin, Mechanic |
| PUT | `/api/repairs/:id` | Update repair | Admin, Mechanic |
| DELETE | `/api/repairs/:id` | Delete repair | Admin, Mechanic |
| GET | `/api/suppliers` | List suppliers | All |
| POST | `/api/suppliers` | Create supplier | All |
| PUT | `/api/suppliers/:id` | Update supplier | All |
| DELETE | `/api/suppliers/:id` | Delete supplier | All |
| GET | `/api/analytics` | Dashboard analytics | All |
| GET | `/api/reports/*` | Generate reports | All |

### 3.3 Admin Endpoints (API Key + Device Key + JWT + Admin Check)

| Method | Path | Description | Roles |
|--------|------|-------------|-------|
| GET | `/api/users` | List all users | Admin only |
| POST | `/api/users` | Create user | Admin only |
| PUT | `/api/users/:id` | Update user | Admin only |
| DELETE | `/api/users/:id` | Delete user | Admin only |
| GET | `/api/device-keys` | List device keys | Admin only |
| POST | `/api/device-keys` | Create device key | Admin only |
| DELETE | `/api/device-keys/:id` | Revoke device key | Admin only |

**Source:** `crates/workshop-server/src/routes/mod.rs:17-38`

---

## 4. Workshop-Level Data Isolation

### 4.1 Isolation Model

All business data queries are filtered by `workshop_id`, extracted from the JWT token. This ensures that a user can only access data belonging to their workshop.

```
┌─────────────────────────────────────────────────────────────────┐
│                    DATA ISOLATION                                │
│                                                                 │
│  JWT Token                                                     │
│  ┌──────────────────────────────────────┐                      │
│  │ workshop_id: "6ba7b810-..."          │                      │
│  └──────────────────┬───────────────────┘                      │
│                     │                                          │
│                     ▼                                          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  SELECT * FROM products WHERE workshop_id = $1           │  │
│  │  SELECT * FROM sales WHERE workshop_id = $1              │  │
│  │  SELECT * FROM repairs WHERE workshop_id = $1            │  │
│  │  SELECT * FROM suppliers WHERE workshop_id = $1          │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  Workshop A data ──────▶ Accessible by Workshop A users only   │
│  Workshop B data ──────▶ Accessible by Workshop B users only   │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Implementation

The `workshop_id` is:
1. Embedded in the JWT token at creation (`auth.rs:27`)
2. Extracted by the JWT middleware (`middleware.rs:32`)
3. Stored in `AuthenticatedUser.workshop_id` (`middleware.rs:19`)
4. Used as a filter in all data queries

---

## 5. Self-Protection Rules

### 5.1 Rules

| Rule | Description | Implementation |
|------|-------------|---------------|
| Cannot demote self | Admin cannot change their own role to non-admin | Route handler check |
| Cannot delete self | Admin cannot delete their own account | Route handler check |
| Cannot revoke own device key | Prevents lockout scenarios | Route handler check |

### 5.2 Privilege Escalation Prevention

| Attack Vector | Mitigation |
|--------------|------------|
| JWT role tampering | HMAC signature verification (HS256) |
| Forged JWT | Secret stored server-side, never exposed |
| Role injection via header | Role extracted from JWT claims, not headers |
| Admin endpoint access | `require_admin_middleware` checks `role == admin` |
| Workshop ID tampering | Embedded in JWT, verified by HMAC |

---

## 6. Middleware Implementation Details

### 6.1 API Key Middleware

**Source:** `crates/workshop-server/src/middleware.rs:48-62`

```
Header: X-WorkshopManager-Key
  │
  ├── Key present? ──No──▶ 401 Unauthorized
  │
  └── Key matches config.api_key? ──No──▶ 401 Unauthorized
      │
      └── Yes ──▶ Continue to next middleware
```

### 6.2 Device Key Middleware

**Source:** `crates/workshop-server/src/device_key.rs:23-58`

```
Header: X-WorkshopManager-Device-Key
  │
  ├── require_device_key disabled? ──Yes──▶ Continue (skip check)
  │
  ├── Key present? ──No──▶ 401 Unauthorized
  │
  └── Key hash exists in DB AND active=true? ──No──▶ 401 Unauthorized
      │
      └── Yes ──▶ Update last_seen_at ──▶ Continue
```

### 6.3 JWT Auth Middleware

**Source:** `crates/workshop-server/src/middleware.rs:65-78`

```
Header: Authorization: Bearer <token>
  │
  ├── Token present? ──No──▶ 401 Unauthorized
  │
  ├── Token valid (HS256 + not expired)? ──No──▶ 401 Unauthorized
  │
  ├── Claims valid (UUID, role)? ──No──▶ 401 Unauthorized
  │
  └── Yes ──▶ Insert AuthenticatedUser into request extensions
              ──▶ Continue to next middleware
```

### 6.4 Admin Check Middleware

**Source:** `crates/workshop-server/src/middleware.rs:81-95`

```
AuthenticatedUser in request?
  │
  ├── No ──▶ 401 Unauthorized
  │
  └── Yes ──▶ user.is_admin()?
      │
      ├── No ──▶ 403 Forbidden
      │
      └── Yes ──▶ Continue to route handler
```

---

## 7. Error Responses

### 7.1 Authentication Errors

| Status | Error | When |
|--------|-------|------|
| 401 | Unauthorized | Missing/invalid API key, device key, or JWT |
| 403 | Forbidden | Valid JWT but insufficient role |

### 7.2 Standard Error Format

```json
{
  "success": false,
  "error": "Unauthorized"
}
```

**Source:** `crates/workshop-server/src/error.rs`

---

## 8. Cross-References

| Document | Description |
|----------|-------------|
| [Overview](./overview.md) | Security architecture overview |
| [Authentication](./authentication.md) | Password hashing, JWT lifecycle |
| [Encryption](./encryption.md) | Cryptographic mechanisms |
| [Hardening Checklist](./hardening.md) | Security hardening steps |
| [API Users](../02-api-reference/users.md) | User management API reference |
| [API Device Keys](../02-api-reference/device-keys.md) | Device key API reference |
