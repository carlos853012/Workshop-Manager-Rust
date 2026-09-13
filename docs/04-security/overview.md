# Security Architecture Overview — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Security engineers, auditors, developers

---

## 1. Defense in Depth Strategy

WorkshopManager implements a layered security model where no single control is relied upon for overall security. Each layer provides independent protection, so compromising one layer does not grant access to the system.

```
┌─────────────────────────────────────────────────────────────────────┐
│  LAYER 6: AUDIT & MONITORING                                       │
│  Audit trail, log redaction, backup integrity                      │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 5: APPLICATION LOGIC                                        │
│  Input validation, RBAC, rate limiting, self-protection rules      │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 4: AUTHENTICATION & AUTHORIZATION                           │
│  JWT (HS256), Argon2id, API key, device key, admin middleware      │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 3: TRANSPORT ENCRYPTION                                     │
│  TLS 1.3 (rustls), self-signed certificates, HSTS                 │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 2: DATA ENCRYPTION                                          │
│  AES-256-GCM (data at rest), encrypted backups, secret storage     │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 1: INFRASTRUCTURE                                           │
│  localhost binding, file permissions, embedded PostgreSQL, no unsafe│
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. Security Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         CLIENT (inventory-viewer)                       │
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────────┐  │
│  │ API Key      │  │ Device Key   │  │ JWT Bearer Token             │  │
│  │ (static)     │  │ (per-device) │  │ (8h expiry)                  │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────────┬───────────────┘  │
└─────────┼─────────────────┼────────────────────────┼──────────────────┘
          │                 │                        │
          ▼                 ▼                        ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    SERVER MIDDLEWARE STACK (Axum 0.7)                   │
│                                                                         │
│  ┌─────────┐   ┌──────────┐   ┌──────────┐   ┌──────┐   ┌──────────┐ │
│  │  CORS   │──▶│  HSTS    │──▶│ Headers  │──▶│ Body │──▶│  API Key │ │
│  │ localhost│   │ max-age  │   │ nosniff  │   │ 10MB │   │  Check   │ │
│  └─────────┘   └──────────┘   └──────────┘   └──────┘   └────┬─────┘ │
│                                                                │       │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   │       │
│  │ Device   │◀──│   JWT    │◀──│  Admin   │◀──│  Route   │◀──┘       │
│  │ Key Check│   │  Auth    │   │  Check   │   │ Handler  │           │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘           │
└──────────────────────────────────────┬──────────────────────────────────┘
                                       │
                  ┌────────────────────┼────────────────────┐
                  ▼                    ▼                    ▼
        ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
        │  PostgreSQL   │    │   Secrets    │    │   Audit Log  │
        │  (embedded)   │    │  (encrypted) │    │  (all CRUD)  │
        └──────────────┘    └──────────────┘    └──────────────┘
```

---

## 3. Threat Model Summary

### 3.1 Asset Classification

| Asset | Sensitivity | Protection |
|-------|------------|------------|
| User passwords | Critical | Argon2id hashing, never stored in plaintext |
| JWT signing secret | Critical | AES-256-GCM encrypted at rest, file permissions |
| AES-256 crypto key | Critical | File permissions, hidden on Windows |
| Database credentials | High | AES-256-GCM encrypted at rest |
| Business data (products, sales, repairs) | Medium | TLS in transit, RBAC, audit trail |
| Audit logs | Medium | Append-only, credential redaction |
| TLS private key | High | File permissions, hidden on Windows |
| Device keys (hashed) | Medium | SHA-256, database storage, revocable |
| Backups | High | AES-256-GCM encrypted, 7-day retention |

### 3.2 Threat Actors

| Actor | Capability | Mitigation |
|-------|-----------|------------|
| Unauthorized local user | Access to filesystem | File permissions, OS-level protection |
| Compromised viewer instance | Network access | API key, device key, JWT validation |
| Brute-force attacker | Login attempts | Rate limiting (5 attempts / 300s) |
| Man-in-the-middle | Network interception | TLS 1.3 encryption |
| Malicious admin | Privilege abuse | Self-protection rules, audit trail |
| Stolen backup file | Data exfiltration | AES-256-GCM encryption |

### 3.3 Trust Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                    TRUST BOUNDARY: HOST MACHINE                 │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              TRUST BOUNDARY: APPLICATION                  │  │
│  │                                                           │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌────────────┐ │  │
│  │  │ Viewer  │  │ Server  │  │   DB    │  │  Secrets   │ │  │
│  │  └─────────┘  └─────────┘  └─────────┘  └────────────┘ │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  TRUST BOUNDARY: FILE SYSTEM                                    │
│  $LOCALAPPDATA/WorkshopManager/data/                            │
│  (.jwt_secret, .crypto_key, backups/, *.pem)                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. Security Controls Inventory

### 4.1 Preventive Controls

| Control | Implementation | File | Status |
|---------|---------------|------|--------|
| Password hashing | Argon2id (64MB, 3 iter, 4 parallelism) | `auth.rs:13` | Implemented |
| JWT authentication | HS256, 8h expiry, Bearer token | `auth.rs:57` | Implemented |
| AES-256-GCM encryption | Data at rest (passwords, backups) | `crypto.rs:40` | Implemented |
| TLS 1.3 | Transport encryption (rustls) | `tls.rs` | Implemented |
| API key | Shared secret header validation | `middleware.rs:48` | Implemented |
| Device key | Hardware-bound SHA-256 hashed keys | `device_key.rs:23` | Implemented |
| RBAC | Admin/Mechanic/Seller roles | `middleware.rs:81` | Implemented |
| Rate limiting | 5 attempts / 300s per email | `rate_limiter.rs` | Implemented |
| Input validation | Email regex, password length, field lengths | `routes/auth.rs:244` | Implemented |
| SQL injection prevention | Parameterized queries only (sqlx) | All routes | Implemented |
| Body size limit | 10MB maximum request body | `main.rs:123` | Implemented |
| CORS restriction | localhost only, specific methods/headers | `main.rs:125` | Implemented |
| Security headers | HSTS, X-Content-Type-Options, X-Frame-Options | `main.rs:143` | Implemented |

### 4.2 Detective Controls

| Control | Implementation | File | Status |
|---------|---------------|------|--------|
| Audit trail | All CRUD operations logged with old/new values | `audit.rs:7` | Implemented |
| Credential redaction | Sensitive fields masked in audit logs | `audit.rs:47` | Implemented |
| Login attempt tracking | Rate limiter records failed attempts | `rate_limiter.rs:44` | Implemented |
| Error sanitization | DB errors mapped to generic messages | `error.rs` | Implemented |

### 4.3 Corrective Controls

| Control | Implementation | File | Status |
|---------|---------------|------|--------|
| Automatic backups | Daily encrypted backups (pg_dump + gzip + AES-256) | `backup.rs:10` | Implemented |
| Backup retention | 7-day rolling window | `backup.rs:55` | Implemented |
| Device key revocation | Admin can revoke active keys | `device_key.rs:91` | Implemented |
| Graceful shutdown | 10-second timeout, PostgreSQL stop | `main.rs:189` | Implemented |

---

## 5. Compliance Mapping

### 5.1 OWASP Top 10 (2021) Mapping

| # | Vulnerability | Mitigation | Status |
|---|--------------|------------|--------|
| A01 | Broken Access Control | RBAC + middleware stack + workshop_id isolation | Implemented |
| A02 | Cryptographic Failures | AES-256-GCM, TLS 1.3, Argon2id, CSPRNG | Implemented |
| A03 | Injection | Parameterized SQL queries (sqlx compile-time) | Implemented |
| A04 | Insecure Design | Defense in depth, no `unsafe`, constitutional rules | Implemented |
| A05 | Security Misconfiguration | Secure defaults, file permissions, hidden secrets | Implemented |
| A06 | Vulnerable Components | Rust ecosystem, `cargo audit` (recommended) | Partial |
| A07 | Auth Failures | Rate limiting, Argon2id, JWT validation | Implemented |
| A08 | Data Integrity Failures | Audit trail, backup encryption, transaction integrity | Implemented |
| A09 | Logging Failures | Audit log with redaction, tracing | Implemented |
| A10 | SSRF | localhost-only binding, no external requests | Implemented |

### 5.2 NIST SP 800-63B Password Guidelines

| Requirement | WorkshopManager | Compliant |
|------------|----------------|-----------|
| Minimum 8 characters | Yes (`validate_password`) | Yes |
| No maximum length limit | Yes (no upper bound enforced) | Yes |
| Password hashing | Argon2id (memory-hard) | Yes |
| Salt generation | OsRng CSPRNG | Yes |
| No composition rules | Yes (length-only policy) | Yes |
| No password hints | Yes (not stored) | Yes |
| No password reuse | Not implemented | No |
| Rate limiting on login | 5 attempts / 300s | Yes |

### 5.3 NIST SP 800-57 Key Management

| Requirement | Implementation | Status |
|------------|---------------|--------|
| Key generation | OsRng (CSPRNG) | Implemented |
| Key storage | Filesystem with permissions | Implemented |
| Key rotation | Not implemented (backlog) | Backlog |
| Key destruction | Not implemented (backlog) | Backlog |

---

## 6. Known Limitations and Backlog

### 6.1 Current Limitations

| ID | Limitation | Risk | Mitigation |
|----|-----------|------|------------|
| L1 | JWT tokens cannot be revoked (stateless) | Compromised token valid until expiry | 8h expiry limits window |
| L2 | No token refresh mechanism | User must re-login after 8h | Acceptable for desktop use |
| L3 | Secrets stored as raw files | Filesystem access exposes secrets | File permissions, OS protection |
| L4 | No password history enforcement | User can reuse old passwords | Low risk for single-workshop |
| L5 | Device keys not scoped to workshop_id | Any workshop can use any key | Physical access required |
| L6 | Self-signed TLS certificates | Browser warnings, no CA validation | Desktop app, localhost only |

### 6.2 Security Backlog

| ID | Item | Priority | Effort |
|----|------|----------|--------|
| B1 | Migrate secrets to DPAPI (Windows) / Keychain (macOS) | High | Medium |
| B2 | Implement token revocation (blacklist or volatile secret) | Medium | Low |
| B3 | Add `workshop_id` scoping to device_keys | Medium | Low |
| B4 | Add `cargo audit` to CI pipeline | Medium | Low |
| B5 | Implement password history (last N passwords) | Low | Medium |
| B6 | Add security headers audit (CSP, Permissions-Policy) | Low | Low |
| B7 | Implement automated key rotation for crypto_key | Low | High |

---

## 7. Cross-References

| Document | Description |
|----------|-------------|
| [Authentication](./authentication.md) | Password hashing, JWT lifecycle, rate limiting |
| [Authorization](./authorization.md) | RBAC model, middleware stack, endpoint permissions |
| [Encryption](./encryption.md) | AES-256-GCM, TLS 1.3, secret management |
| [Audit Trail](./audit.md) | Audit log schema, redaction, retention |
| [Hardening Checklist](./hardening.md) | OWASP checklist, server hardening, incident response |
| [Architecture Overview](../01-architecture/overview.md) | System design, technology choices |
| [CONSTITUCION.md](../../CONSTITUCION.md) | Constitutional rules (no unsafe, parameterized SQL) |
