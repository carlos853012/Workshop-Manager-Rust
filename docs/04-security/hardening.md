# Security Hardening Checklist — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, security engineers, DevOps, auditors

---

## 1. OWASP Top 10 (2021) Checklist

### A01: Broken Access Control

| Check | Status | Implementation |
|-------|--------|---------------|
| RBAC enforced on all endpoints | ✅ | Middleware stack (API key → device key → JWT → admin) |
| Workshop-level data isolation | ✅ | `workshop_id` filter on all queries |
| Deny by default | ✅ | Public routes explicitly defined, all others protected |
| CORS restricted to localhost | ✅ | `AllowOrigin::list` with localhost/127.0.0.1 |
| Method restrictions | ✅ | GET, POST, PUT, DELETE only |
| Header restrictions | ✅ | Authorization + Content-Type only |
| Self-protection rules | ✅ | Cannot demote/delete self |
| No directory traversal | ✅ | No file serving endpoints |

### A02: Cryptographic Failures

| Check | Status | Implementation |
|-------|--------|---------------|
| Strong encryption algorithm | ✅ | AES-256-GCM (NIST-approved) |
| Strong password hashing | ✅ | Argon2id (OWASP-recommended) |
| TLS for transit | ✅ | TLS 1.3 via rustls |
| Secrets not in code | ✅ | Stored in filesystem, not source code |
| CSPRNG for key generation | ✅ | `OsRng` from `rand` crate |
| No hardcoded credentials | ✅ | API key configurable, secrets generated |

### A03: Injection

| Check | Status | Implementation |
|-------|--------|---------------|
| Parameterized SQL queries | ✅ | sqlx with `$1, $2, ...` bind parameters |
| No string concatenation in SQL | ✅ | All queries use `sqlx::query().bind()` |
| Input validation | ✅ | Email regex, password length, field lengths |
| Type-safe queries | ✅ | sqlx compile-time query checking |

### A04: Insecure Design

| Check | Status | Implementation |
|-------|--------|---------------|
| Defense in depth | ✅ | 6-layer security model |
| No `unsafe` code | ✅ | Constitutional rule enforced |
| No `unwrap()` in production | ✅ | `Result<T, E>` throughout |
| Minimal attack surface | ✅ | Desktop-only, localhost binding |
| Threat model defined | ✅ | See [overview.md](./overview.md) |

### A05: Security Misconfiguration

| Check | Status | Implementation |
|-------|--------|---------------|
| Secure defaults | ✅ | localhost binding, device key optional |
| Error messages sanitized | ✅ | DB errors mapped to generic messages |
| File permissions set | ✅ | `0o600` on Unix, hidden on Windows |
| No debug mode in production | ✅ | Config-driven, no debug features exposed |
| Security headers set | ✅ | HSTS, X-Content-Type-Options, X-Frame-Options |

### A06: Vulnerable and Outdated Components

| Check | Status | Implementation |
|-------|--------|---------------|
| Rust ecosystem (memory-safe) | ✅ | No buffer overflows, use-after-free |
| `cargo audit` in CI | ⚠️ | **Recommended** — not yet implemented |
| Dependency pinning | ✅ | `Cargo.lock` committed |
| Minimal dependencies | ✅ | Only essential crates |

### A07: Identification and Authentication Failures

| Check | Status | Implementation |
|-------|--------|---------------|
| Strong password hashing | ✅ | Argon2id (64MB, 3 iter, 4 parallelism) |
| Rate limiting on login | ✅ | 5 attempts / 300s per email |
| No credential stuffing protection | ⚠️ | **Limited** — rate limiting only |
| JWT validation | ✅ | HS256 signature + expiration check |
| Session timeout | ✅ | 8-hour token expiry |

### A08: Software and Data Integrity Failures

| Check | Status | Implementation |
|-------|--------|---------------|
| Audit trail | ✅ | All CRUD operations logged |
| Backup integrity | ✅ | AES-256-GCM encrypted backups |
| Transaction integrity | ✅ | PostgreSQL ACID transactions |
| No unsigned updates | ✅ | Desktop app, no auto-update mechanism |
| Secure deserialization | ✅ | serde with type-safe DTOs |

### A09: Security Logging and Monitoring Failures

| Check | Status | Implementation |
|-------|--------|---------------|
| Audit logging | ✅ | `audit_log` table with old/new values |
| Credential redaction | ✅ | Passwords, keys redacted in audit logs |
| Error logging | ✅ | Tracing with structured output |
| Login attempt tracking | ✅ | Rate limiter records attempts |
| Centralized logging | ⚠️ | **Limited** — local tracing only |

### A10: Server-Side Request Forgery (SSRF)

| Check | Status | Implementation |
|-------|--------|---------------|
| Localhost binding | ✅ | `127.0.0.1` only |
| No external requests | ✅ | Server does not make outbound HTTP calls |
| No URL input fields | ✅ | No user-supplied URLs processed |

---

## 2. NIST SP 800-63B Password Guidelines

### 2.1 Compliance Matrix

| Requirement | NIST Guideline | WorkshopManager | Compliant |
|------------|---------------|-----------------|-----------|
| Minimum length | 8 characters | 8 characters | ✅ |
| Maximum length | 64+ characters | No upper limit | ✅ |
| No composition rules | No mandatory complexity | Length-only policy | ✅ |
| No password hints | Not stored | Not stored | ✅ |
| No password reuse | Check last N passwords | Not implemented | ❌ |
| No knowledge-based auth | Not required | Not required | ✅ |
| Secure storage | Salted, hashed | Argon2id with random salt | ✅ |
| Rate limiting | Throttle attempts | 5 attempts / 300s | ✅ |
| No truncation | Accept all lengths | No truncation | ✅ |
| Unicode support | Accept Unicode | Accepts any characters | ✅ |

---

## 3. Server Hardening

### 3.1 Network Configuration

| Setting | Value | File |
|---------|-------|------|
| Bind address | `127.0.0.1` | `config/server.toml` |
| Port | `8443` | `config/server.toml` |
| TLS | TLS 1.3 (rustls) | `tls.rs` |
| CORS origins | localhost, 127.0.0.1 | `main.rs:126` |

### 3.2 Request Limits

| Setting | Value | File |
|---------|-------|------|
| Body size limit | 10 MB | `main.rs:123` |
| Rate limit (login) | 5 attempts / 300s | `routes/auth.rs` |

### 3.3 Security Headers

| Header | Value | File |
|--------|-------|------|
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | `main.rs:144` |
| `X-Content-Type-Options` | `nosniff` | `main.rs:148` |
| `X-Frame-Options` | `DENY` | `main.rs:152` |

### 3.4 TLS Configuration

| Setting | Value |
|---------|-------|
| Protocol | TLS 1.3 |
| Certificate | Self-signed (auto-generated) |
| Key storage | PEM file with `0o600` permissions |
| Client auth | Not required |

---

## 4. File System Permissions

### 4.1 Data Directory

| Path | Permissions | Content |
|------|------------|---------|
| `$LOCALAPPDATA/WorkshopManager/data/` | User only | All application data |
| `.jwt_secret` | `0o600` (Unix), hidden (Windows) | JWT signing secret |
| `.crypto_key` | `0o600` (Unix), hidden (Windows) | AES-256-GCM key |
| `server.key` | `0o600` (Unix), hidden (Windows) | TLS private key |
| `server.crt` | Standard | TLS certificate |
| `backups/` | User only | Encrypted backup files |

### 4.2 Permission Implementation

**Unix:**
```rust
use std::os::unix::fs::PermissionsExt;
let mut perms = std::fs::metadata(&key_path)?.permissions();
perms.set_mode(0o600);
std::fs::set_permissions(&key_path, perms)?;
```

**Windows:**
```rust
std::process::Command::new("attrib")
    .arg("+H")
    .arg(&key_path)
    .output();
```

---

## 5. Monitoring and Alerting

### 5.1 Current Monitoring

| Source | What to Monitor | Tool |
|--------|----------------|------|
| Server tracing | Request lifecycle, errors | `tracing_subscriber` |
| Rate limiter | Login brute-force attempts | In-memory (logged) |
| Audit log | Data modifications | PostgreSQL query |
| PostgreSQL | Connection pool, query performance | Embedded |

### 5.2 Recommended Alerting

| Alert | Trigger | Severity |
|-------|---------|----------|
| Repeated login failures | >10 failures in 5 minutes | High |
| Admin account changes | Any user role modification | High |
| Backup failure | Backup creation error | Medium |
| TLS certificate expiry | <30 days to expiry | Medium |
| Database connection pool exhaustion | >80% pool utilization | High |

### 5.3 Log Analysis Queries

**Brute-force detection:**
```sql
-- Note: Rate limiter is in-memory. Check server traces for failed login patterns.
SELECT created_at, user_id, action, entity_type
FROM audit_log
WHERE action = 'DELETE'
  AND created_at > NOW() - INTERVAL '1 hour'
ORDER BY created_at DESC;
```

**Suspicious activity:**
```sql
SELECT
    user_id,
    COUNT(*) as change_count,
    MIN(created_at) as first_change,
    MAX(created_at) as last_change
FROM audit_log
WHERE created_at > NOW() - INTERVAL '24 hours'
GROUP BY user_id
HAVING COUNT(*) > 100
ORDER BY change_count DESC;
```

---

## 6. Incident Response Procedures

### 6.1 Incident Classification

| Severity | Description | Response Time |
|----------|-------------|---------------|
| Critical | Data breach, unauthorized admin access | Immediate |
| High | Brute-force attack, suspicious deletions | < 1 hour |
| Medium | Failed backup, configuration changes | < 24 hours |
| Low | Unusual activity patterns | < 72 hours |

### 6.2 Response Steps

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  1. Detect   │────▶│  2. Contain  │────▶│  3. Analyze  │────▶│  4. Remediate│
│  (monitoring)│     │  (isolate)   │     │  (audit log) │     │  (fix)       │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
                                                                       │
                                                                       ▼
                                                                 ┌──────────────┐
                                                                 │  5. Review   │
                                                                 │  (postmortem)│
                                                                 └──────────────┘
```

### 6.3 Containment Actions

| Scenario | Containment |
|----------|------------|
| Compromised credentials | Revoke affected device keys, force re-login |
| Brute-force attack | Increase rate limit, block IP at OS level |
| Unauthorized admin access | Revoke admin role, audit all changes |
| Data breach | Stop server, preserve audit logs, encrypt backups |
| Malicious insider | Revoke all access, preserve audit trail |

### 6.4 Evidence Preservation

| Evidence | Location | Preservation |
|----------|----------|-------------|
| Audit logs | `audit_log` table | Export to secure location |
| Server traces | stdout/log files | Copy to secure storage |
| Database backup | `backups/` directory | Encrypted copy offsite |
| Configuration | `config/server.toml` | Version control history |
| TLS certificates | `data/` directory | Preserve for analysis |

---

## 7. Security Review Schedule

### 7.1 Regular Reviews

| Review | Frequency | Responsible |
|--------|-----------|-------------|
| Dependency audit (`cargo audit`) | Monthly | Developer |
| Configuration review | Quarterly | Developer |
| Access control review | Quarterly | Workshop owner |
| Backup restoration test | Quarterly | Developer |
| TLS certificate check | Annually | Developer |
| Security documentation update | Per release | Developer |

### 7.2 Pre-Release Checklist

| Check | Owner | Status |
|-------|-------|--------|
| Run `cargo clippy --workspace -- -D warnings` | Developer | Required |
| Run `cargo test --workspace` | Developer | Required |
| Run `cargo audit` (if available) | Developer | Recommended |
| Review security documentation | Developer | Required |
| Test backup/restore cycle | Developer | Recommended |
| Verify file permissions | Developer | Required |
| Review rate limiting configuration | Developer | Required |

### 7.3 Production Security Checklist

| Check | Status | Notes |
|-------|--------|-------|
| API key changed from default | ⚠️ | Verify `config/server.toml` |
| Argon2id params hardened | ✅ | 64MB memory |
| Body size limit configured | ✅ | 10MB |
| CORS restricted | ✅ | localhost only |
| Security headers set | ✅ | HSTS, nosniff, DENY |
| Rate limiting active | ✅ | 5 attempts / 300s |
| Device keys scoped | ⚠️ | Not scoped to workshop_id |
| Secrets use OS protection | ⚠️ | File permissions only (DPAPI/Keychain backlog) |
| Token revocation available | ❌ | Not implemented (backlog) |
| Email validation | ⚠️ | Basic regex only |

---

## 8. Cross-References

| Document | Description |
|----------|-------------|
| [Overview](./overview.md) | Security architecture overview |
| [Authentication](./authentication.md) | Password hashing, JWT lifecycle |
| [Authorization](./authorization.md) | RBAC model, middleware stack |
| [Encryption](./encryption.md) | Cryptographic mechanisms |
| [Audit Trail](./audit.md) | Audit log schema, redaction |
| [CONSTITUCION.md](../../CONSTITUCION.md) | Constitutional rules (no unsafe, parameterized SQL) |
| [AGENTS.md](../../AGENTS.md) | Security checklist and known issues |
