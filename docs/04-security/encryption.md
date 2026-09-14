# Encryption Mechanisms — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, security engineers, auditors

---

## 1. Encryption Overview

WorkshopManager uses encryption at three layers: data at rest, data in transit, and password storage.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        ENCRYPTION LAYERS                                 │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │  LAYER 3: PASSWORD HASHING                                        │  │
│  │  Algorithm: Argon2id (memory-hard, OWASP-recommended)            │  │
│  │  Purpose: Protect user passwords at rest                          │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │  LAYER 2: DATA AT REST (AES-256-GCM)                             │  │
│  │  Algorithm: AES-256-GCM (authenticated encryption)               │  │
│  │  Purpose: Encrypt DB password, backups, sensitive config          │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │  LAYER 1: DATA IN TRANSIT (TLS 1.3)                              │  │
│  │  Protocol: TLS 1.3 (rustls)                                      │  │
│  │  Purpose: Encrypt all client-server communication                 │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. AES-256-GCM (Data at Rest)

### 2.1 Algorithm Properties

| Property | Value |
|----------|-------|
| Algorithm | AES-256-GCM |
| Key size | 256 bits (32 bytes) |
| Nonce size | 96 bits (12 bytes) |
| Tag size | 128 bits (16 bytes) |
| Mode | Authenticated Encryption with Associated Data (AEAD) |
| Crate | `aes-gcm 0.10` |

### 2.2 Key Management

| Aspect | Implementation |
|--------|---------------|
| Generation | `OsRng` (CSPRNG) — 32 random bytes |
| Storage | Filesystem: `$LOCALAPPDATA/WorkshopManager/data/.crypto_key` |
| File permissions | `0o600` (Unix), hidden via `attrib +H` (Windows) |
| Initialization | `OnceLock` cipher (set once at startup, no `unsafe`) |
| Rotation | Not implemented (backlog) |

**Source:** `crates/workshop-server/src/secrets.rs:54-88`

### 2.3 Encryption Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Plaintext   │────▶│  Generate    │────▶│  AES-256-GCM │────▶│  Base64      │
│  (string/    │     │  Random      │     │  Encrypt     │     │  Encode      │
│   bytes)     │     │  Nonce (12B) │     │              │     │              │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
                                                                  │
                                                                  ▼
                                                           ┌──────────────┐
                                                           │  Output:     │
                                                           │  base64(     │
                                                           │   nonce ||   │
                                                           │   ciphertext │
                                                           │   || tag)    │
                                                           └──────────────┘
```

**Source:** `crates/workshop-server/src/crypto.rs:40-60`

### 2.4 Decryption Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Base64      │────▶│  Split       │────▶│  AES-256-GCM │────▶│  UTF-8       │
│  Ciphertext  │     │  nonce (12B) │     │  Decrypt     │     │  Decode      │
│              │     │  + ciphertext│     │  + Verify    │     │              │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

**Source:** `crates/workshop-server/src/crypto.rs:62-78`

### 2.5 What Gets Encrypted

| Data | Purpose | Location |
|------|---------|----------|
| PostgreSQL password | Protect DB credentials | DB connection string |
| Backup files | Protect data at rest | `$LOCALAPPDATA/.../backups/*.sql.gz.enc` |
| Sensitive config values | Protect secrets | Configuration files |

### 2.6 Nonce Randomness

Each encryption operation generates a fresh random 12-byte nonce via `OsRng`. This ensures that encrypting the same plaintext twice produces different ciphertexts, preventing nonce reuse attacks.

**Test:** `crates/workshop-server/src/crypto.rs:118-128`

---

## 3. TLS 1.3 (Data in Transit)

### 3.1 Protocol Properties

| Property | Value |
|----------|-------|
| Protocol | TLS 1.3 |
| Library | `rustls` (via `axum-server`) |
| Certificate | Self-signed (auto-generated) |
| Key exchange | ECDHE (default in rustls) |
| Cipher suite | TLS 1.3 defaults (AES-256-GCM, ChaCha20-Poly1305) |
| Port | 8443 |

### 3.2 Certificate Management

**Source:** `crates/workshop-server/src/tls.rs:56-101`

| Aspect | Implementation |
|--------|---------------|
| Generation | `rcgen` crate (pure Rust, no OpenSSL) |
| Type | Self-signed X.509 |
| Subject CN | `WorkshopManager Server` |
| SANs | `localhost`, `127.0.0.1` |
| Key pair | Generated via `rcgen::KeyPair::generate()` |
| Storage | PEM files: `server.crt`, `server.key` |
| File permissions | `0o600` for private key (Unix), hidden (Windows) |
| Regeneration | Auto-generated on first run if missing |

### 3.3 Certificate Lifecycle

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Server      │────▶│  Check for   │────▶│  Cert exists?│
│  Startup     │     │  cert files  │     │              │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
                                            ┌─────┴─────┐
                                            │           │
                                         Yes│           │No
                                            ▼           ▼
                                     ┌──────────┐ ┌──────────┐
                                     │  Load    │ │  Generate│
                                     │  from    │ │  self-   │
                                     │  disk    │ │  signed  │
                                     └──────────┘ └──────────┘
```

### 3.4 TLS Configuration

```
rustls::ServerConfig::builder()
    .with_no_client_auth()                    // No client certificates required
    .with_single_cert(certs, key)             // Server certificate + private key
```

**Source:** `crates/workshop-server/src/tls.rs:104-114`

---

## 4. Password Hashing (Argon2id)

### 4.1 Parameters

| Parameter | Value | Source |
|-----------|-------|--------|
| Algorithm | Argon2id | `auth.rs:14` |
| Version | 0x13 | `auth.rs:15` |
| Memory | 65536 KB (64 MB) | `auth.rs:14` |
| Iterations | 3 | `auth.rs:14` |
| Parallelism | 4 | `auth.rs:14` |
| Salt | 16 bytes (OsRng) | `SaltString::generate(&mut OsRng)` |
| Hash length | 32 bytes (default) | Argon2 default |

### 4.2 Hash Storage Format

```
$argon2id$v=19$m=65536,t=3,p=4$<base64-salt>$<base64-hash>
```

### 4.3 OWASP Compliance

| Requirement | Status |
|------------|--------|
| Memory >= 19 MB | ✅ 64 MB |
| Iterations >= 2 | ✅ 3 |
| Parallelism >= 1 | ✅ 4 |
| Salt from CSPRNG | ✅ OsRng |
| Unique salt per password | ✅ Generated at hash time |

---

## 5. Secret Management

### 5.1 Secrets Overview

| Secret | Purpose | Generation | Storage |
|--------|---------|-----------|---------|
| JWT secret | Sign/verify JWT tokens | OsRng (32 bytes, hex-encoded) | `.jwt_secret` file |
| Crypto key | AES-256-GCM encryption | OsRng (32 bytes, raw) | `.crypto_key` file |
| API key | Application authentication | Manual (config) | `config/server.toml` |

### 5.2 Secret Initialization

**Source:** `crates/workshop-server/src/secrets.rs:7-15`

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Startup     │────▶│  Load or     │────▶│  Initialize  │
│              │     │  Generate    │     │  AppState    │
│              │     │  secrets     │     │  Secrets     │
└──────────────┘     └──────────────┘     └──────────────┘
```

### 5.3 JWT Secret

| Aspect | Implementation |
|--------|---------------|
| Length | 64 hex characters (32 bytes) |
| Generation | `OsRng` CSPRNG |
| Storage | `$LOCALAPPDATA/WorkshopManager/data/.jwt_secret` |
| File permissions | `0o600` (Unix), hidden (Windows) |
| Persistence | Generated once, reused across restarts |
| Algorithm | HS256 (HMAC-SHA256) |

**Source:** `crates/workshop-server/src/secrets.rs:17-52`

### 5.4 Crypto Key

| Aspect | Implementation |
|--------|---------------|
| Length | 32 bytes (raw binary) |
| Generation | `OsRng` CSPRNG |
| Storage | `$LOCALAPPDATA/WorkshopManager/data/.crypto_key` |
| File permissions | `0o600` (Unix), hidden (Windows) |
| Persistence | Generated once, reused across restarts |
| Algorithm | AES-256-GCM |

**Source:** `crates/workshop-server/src/secrets.rs:54-88`

### 5.5 Secret File Permissions

| OS | Method | Permission |
|----|--------|------------|
| Unix | `std::os::unix::fs::PermissionsExt` | `0o600` (owner read/write only) |
| Windows | `attrib +H` | Hidden file attribute |

---

## 6. Key Generation (CSPRNG)

### 6.1 Random Number Generator

| Property | Value |
|----------|-------|
| Generator | `OsRng` (from `rand` crate) |
| Type | Cryptographically Secure PRNG |
| Source | Operating system entropy (`/dev/urandom`, `BCryptGenRandom`) |
| Use cases | Salt generation, nonce generation, secret key generation |

### 6.2 What Uses CSPRNG

| Operation | File | Purpose |
|-----------|------|---------|
| Password salt | `auth.rs:35` | Unique salt per password hash |
| JWT secret | `secrets.rs:28` | 32-byte signing secret |
| Crypto key | `secrets.rs:66` | 32-byte encryption key |
| AES nonce | `crypto.rs:48` | 12-byte random nonce per encryption |
| Device key | `device_key.rs:15` | `wm_<uuid>` format key |

---

## 7. Backup Encryption

### 7.1 Backup Pipeline

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  pg_dump     │────▶│  gzip        │────▶│  AES-256-GCM │────▶│  Write to    │
│  (SQL dump)  │     │  Compress    │     │  Encrypt     │     │  disk        │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

**Source:** `crates/workshop-server/src/backup.rs:10-52`

### 7.2 Backup Properties

| Property | Value |
|----------|-------|
| Format | `.sql.gz.enc` (SQL + gzip + AES-256-GCM) |
| Schedule | Every 24 hours (at boot + periodic) |
| Retention | 7 most recent backups |
| Encryption key | Same crypto key as application |
| Location | `$LOCALAPPDATA/WorkshopManager/data/backups/` |

---

## 8. Known Improvements (Backlog)

### 8.1 Current Limitations

| ID | Limitation | Risk | Mitigation |
|----|-----------|------|------------|
| E1 | Secrets stored as raw files | Filesystem access exposes secrets | File permissions, OS protection |
| E2 | No key rotation mechanism | Long-lived keys increase exposure window | Regular manual rotation recommended |
| E3 | Self-signed TLS certificates | No CA validation, trust-on-first-use | Desktop app, localhost only |
| E4 | No certificate pinning | MitM on first connection | API key provides application-level auth |

### 8.2 Planned Improvements

| ID | Improvement | Priority | Effort |
|----|------------|----------|--------|
| I1 | Migrate secrets to DPAPI (Windows) / Keychain (macOS) | High | Medium |
| I2 | Implement automatic key rotation for crypto_key | Medium | High |
| I3 | Add certificate pinning for TLS | Medium | Medium |
| I4 | Implement key derivation for backup encryption (separate key) | Low | Medium |

---

## 9. Cross-References

| Document | Description |
|----------|-------------|
| [Overview](./overview.md) | Security architecture overview |
| [Authentication](./authentication.md) | Password hashing and JWT lifecycle |
| [Hardening Checklist](./hardening.md) | Security hardening steps |
| [Architecture Overview](../01-architecture/overview.md) | System design, technology choices |
