# Authentication Mechanisms — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, security engineers, auditors

---

## 1. Password Hashing

### 1.1 Algorithm: Argon2id

WorkshopManager uses Argon2id, the OWASP-recommended password hashing algorithm, implemented via the `argon2 0.5` crate.

**Parameters:**

| Parameter | Value | OWASP Minimum | Compliant |
|-----------|-------|---------------|-----------|
| Algorithm | Argon2id | Argon2id | Yes |
| Version | 0x13 | 0x13 | Yes |
| Memory | 64 MB (65536 KB) | 19 MB (19456 KB) | Yes |
| Iterations | 3 | 2 | Yes |
| Parallelism | 4 | 1 | Yes |
| Salt length | 16 bytes (default) | 16 bytes | Yes |
| Hash length | 32 bytes (default) | 32 bytes | Yes |

**Source:** `crates/inventory-server/src/auth.rs:13-16`

### 1.2 Hash Format

Argon2id produces hashes in the standard PHC string format:

```
$argon2id$v=19$m=65536,t=3,p=4$<salt>$<hash>
```

Example (not a real password):
```
$argon2id$v=19$m=65536,t=3,p=4$YWJjZGVmZ2hpams$abcdef1234567890...
```

### 1.3 Hashing Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Password   │────▶│  Generate    │────▶│  Argon2id    │────▶│  Store Hash  │
│   (input)    │     │  Salt (OsRng)│     │  Hash        │     │  in DB       │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

**Source:** `crates/inventory-server/src/auth.rs:34-43`

### 1.4 Verification Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Password   │────▶│  Parse Hash  │────▶│  Argon2id    │────▶│  Return      │
│   (input)    │     │  from DB     │     │  Verify      │     │  true/false  │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

**Source:** `crates/inventory-server/src/auth.rs:46-54`

---

## 2. JWT Token Structure

### 2.1 Claims

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "email": "admin@workshop.cl",
  "role": "admin",
  "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "exp": 1737043800
}
```

| Claim | Type | Description | Source |
|-------|------|-------------|--------|
| `sub` | string (UUID) | User ID | `users.id` |
| `email` | string | User email | `users.email` |
| `role` | string | One of: `admin`, `mechanic`, `seller` | `users.role` |
| `workshop_id` | string (UUID) | Workshop ID | `users.workshop_id` |
| `exp` | number (UTC timestamp) | Expiration time | `now + 8 hours` |

**Source:** `crates/inventory-server/src/auth.rs:19-31`

### 2.2 Token Properties

| Property | Value | Notes |
|----------|-------|-------|
| Algorithm | HS256 (HMAC-SHA256) | Symmetric signing |
| Secret | 64 hex chars (32 bytes) | Generated via OsRng CSPRNG |
| Expiration | 8 hours from creation | No refresh mechanism |
| Revocation | Not supported | Stateless design |
| Header | Default (`{"alg":"HS256","typ":"JWT"}`) | Standard header |

**Source:** `crates/inventory-server/src/auth.rs:10`

---

## 3. Token Lifecycle

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        TOKEN LIFECYCLE                                   │
│                                                                         │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐         │
│  │  Login   │───▶│  Create  │───▶│  Use     │───▶│  Expire  │         │
│  │  (POST)  │    │  Token   │    │  (API)   │    │  (8h)    │         │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘         │
│       │                                             │                  │
│       │              ┌──────────┐                   │                  │
│       └─────────────▶│  Invalid │◀──────────────────┘                  │
│         (bad creds)  │  Token   │    (expired/wrong secret)            │
│                      └──────────┘                                      │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.1 Token Creation

**Source:** `crates/inventory-server/src/auth.rs:57-83`

```
Input: user_id, email, role, workshop_id, jwt_secret
  │
  ├── Compute expiration: now + 8 hours
  ├── Build Claims struct
  ├── Encode with HS256 (EncodingKey::from_secret)
  └── Return JWT string
```

### 3.2 Token Validation

**Source:** `crates/inventory-server/src/auth.rs:86-96`

```
Input: jwt_string, jwt_secret
  │
  ├── Decode with HS256 (DecodingKey::from_secret)
  ├── Validate expiration (jsonwebtoken crate)
  ├── Parse Claims
  │   ├── Validate UUID format (sub, workshop_id)
  │   ├── Validate role string
  │   └── Return AuthenticatedUser
  └── On any error → 401 Unauthorized
```

---

## 4. Rate Limiting

### 4.1 Configuration

| Parameter | Value | Source |
|-----------|-------|--------|
| Max attempts | 5 | `login.rs` |
| Window | 300 seconds (5 minutes) | `login.rs` |
| Key format | `login:{email}` | `routes/auth.rs:33` |
| Storage | In-memory (`RwLock<HashMap>`) | `rate_limiter.rs:7` |

**Source:** `crates/inventory-server/src/rate_limiter.rs:6-10`

### 4.2 Rate Limiting Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Login       │────▶│  Check       │────▶│  Under limit?│
│  Request     │     │  rate_key    │     │              │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
                                            ┌─────┴─────┐
                                            │           │
                                         Yes│           │No
                                            ▼           ▼
                                     ┌──────────┐ ┌──────────┐
                                     │  Verify  │ │  Return  │
                                     │  Password│ │  429     │
                                     └──────────┘ └──────────┘
```

**Source:** `crates/inventory-server/src/routes/auth.rs:33-39`

### 4.3 Key Behaviors

- **Sliding window**: Counter resets after 300 seconds from first attempt
- **Per-email**: Each email address has independent rate limiting
- **Records all attempts**: Both successful and failed attempts are counted
- **In-memory only**: Resets on server restart (acceptable for desktop use)

---

## 5. Registration Flow (First-Time Only)

### 5.1 Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  POST        │────▶│  Check       │────▶│  Validate    │────▶│  Create      │
│  /api/auth/  │     │  users count │     │  fields      │     │  Workshop +  │
│  register    │     │  == 0        │     │              │     │  Admin User  │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
       │                    │                    │                     │
       │                    │                    │                     ▼
       │                    │                    │              ┌──────────┐
       │                    │                    │              │  Create  │
       │                    │                    │              │  JWT     │
       │                    │                    │              └──────────┘
       │                    │                    │
       │              ┌─────┴─────┐         ┌────┴────┐
       │              │           │         │         │
       │           Count>0     Count=0   Invalid   Valid
       │              │           │         │         │
       │              ▼           │         ▼         │
       │         ┌────────┐      │    ┌────────┐     │
       │         │  403   │      │    │  422   │     │
       │         │  Admin │      │    │  Valid. │     │
       │         └────────┘      │    └────────┘     │
       │                         │                    │
       │                         ▼                    ▼
       │                   ┌──────────┐        ┌──────────┐
       │                   │  Check   │        │  Return  │
       │                   │  email   │        │  200 +   │
       │                   │  unique  │        │  token   │
       │                   └──────────┘        └──────────┘
```

### 5.2 Request Body

```json
{
  "workshop_name": "Taller Moto Chile",
  "workshop_address": "Av. Libertador 1234",
  "workshop_city": "Santiago",
  "admin_name": "Carlos Admin",
  "email": "admin@workshop.cl",
  "password": "mypassword123"
}
```

### 5.3 Validation Rules

| Field | Rule | Error |
|-------|------|-------|
| `workshop_name` | 1–200 characters | Validation error |
| `workshop_address` | 1–300 characters | Validation error |
| `workshop_city` | 1–120 characters | Validation error |
| `admin_name` | 1–200 characters | Validation error |
| `email` | Valid format (`^[^\s@]+@[^\s@]+\.[^\s@]+$`), max 200 chars | Validation error |
| `password` | Minimum 8 characters | Validation error |

### 5.4 Business Rules

1. Registration is only allowed when `users` table is empty (count == 0)
2. The first user is automatically assigned the `admin` role
3. Workshop and user are created in a single database transaction
4. JWT token is returned immediately (no separate login needed)
5. Password is hashed with Argon2id before storage

**Source:** `crates/inventory-server/src/routes/auth.rs:84-196`

---

## 6. Login Flow

### 6.1 Sequence Diagram

```
┌────────┐          ┌────────┐          ┌────────┐          ┌────────┐
│ Client │          │ Server │          │  Rate  │          │   DB   │
│        │          │        │          │ Limiter│          │        │
└───┬────┘          └───┬────┘          └───┬────┘          └───┬────┘
    │  POST /auth/login │                   │                   │
    │──────────────────▶│                   │                   │
    │                   │  check(key)       │                   │
    │                   │──────────────────▶│                   │
    │                   │◀──────────────────│                   │
    │                   │  allowed?         │                   │
    │                   │                   │                   │
    │                   │  SELECT user      │                   │
    │                   │──────────────────────────────────────▶│
    │                   │◀──────────────────────────────────────│
    │                   │                   │                   │
    │                   │  verify_password  │                   │
    │                   │  (Argon2id)       │                   │
    │                   │                   │                   │
    │                   │  record_attempt   │                   │
    │                   │──────────────────▶│                   │
    │                   │◀──────────────────│                   │
    │                   │                   │                   │
    │                   │  create_token     │                   │
    │                   │  (HS256, 8h)      │                   │
    │                   │                   │                   │
    │  200 + JWT        │                   │                   │
    │◀──────────────────│                   │                   │
```

### 6.2 Error Responses

| Status | Error | Condition |
|--------|-------|-----------|
| 401 | Unauthorized | Invalid email or password |
| 422 | Validation error | Invalid email format |
| 429 | Too many requests | >5 attempts in 300s window |

**Source:** `crates/inventory-server/src/routes/auth.rs:27-82`

---

## 7. Session Management

### 7.1 Session Properties

| Property | Value |
|----------|-------|
| Type | Stateless (JWT) |
| Duration | 8 hours |
| Storage | Client-side (header) |
| Revocation | Not supported |
| Refresh | Not supported |

### 7.2 Session Termination

| Method | Implementation |
|--------|---------------|
| Token expiry | Automatic after 8 hours |
| Server restart | Does NOT invalidate tokens (stateless) |
| Password change | Does NOT invalidate tokens (backlog) |
| User deletion | Token invalid on next DB lookup (status check) |

---

## 8. Token Storage Recommendations

### 8.1 Client-Side Storage

| Method | Security | Recommended |
|--------|----------|-------------|
| Memory (Rust state) | High — lost on restart | Yes (current) |
| Encrypted file | Medium — depends on file permissions | Alternative |
| OS keychain | High — OS-level protection | Backlog (DPAPI/Keychain) |
| LocalStorage (web) | Low — XSS vulnerable | No |

### 8.2 Current Implementation

The viewer stores the JWT token in-memory via the `AuthProvider` state. The token is:
- Sent via `Authorization: Bearer <token>` header on every request
- Not persisted to disk
- Lost on application restart (user must re-login)

### 8.3 Transmission Security

- All requests use HTTPS (TLS 1.3)
- Token is never sent in URL query parameters
- Token is never logged by the server (audit redaction covers sensitive data)

---

## 9. API Key Authentication

### 9.1 Mechanism

A shared secret (`api_key`) is configured in `config/server.toml` and sent via the `X-WorkshopManager-Key` header.

| Property | Value |
|----------|-------|
| Header | `X-WorkshopManager-Key` |
| Default value | `dev-key-change-in-production` |
| Validation | Exact string match |

**Source:** `crates/inventory-server/src/middleware.rs:48-62`

### 9.2 Purpose

- Prevents unauthorized applications from accessing the API
- Acts as an application-level authentication layer
- Independent of JWT authentication

---

## 10. Device Key Authentication

### 10.1 Mechanism

Device keys bind the application to specific hardware using SHA-256 hashed keys.

| Property | Value |
|----------|-------|
| Header | `X-WorkshopManager-Device-Key` |
| Format | `wm_<uuid>` (e.g., `wm_550e8400e29b41d4a716446655440000`) |
| Storage | SHA-256 hash in `device_keys` table |
| Validation | Hash lookup + active status check |

**Source:** `crates/inventory-server/src/device_key.rs:14-58`

### 10.2 Device Key Lifecycle

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ Generate │───▶│  Store   │───▶│  Use     │───▶│  Revoke  │
│ (admin)  │    │  (hash)  │    │  (check) │    │  (admin) │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
```

---

## 11. Cross-References

| Document | Description |
|----------|-------------|
| [Overview](./overview.md) | Security architecture overview |
| [Authorization](./authorization.md) | RBAC model and middleware stack |
| [Encryption](./encryption.md) | Cryptographic mechanisms |
| [Hardening Checklist](./hardening.md) | Security hardening steps |
| [API Authentication](../02-api-reference/authentication.md) | API endpoint reference |
