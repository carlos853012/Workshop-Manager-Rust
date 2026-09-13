# Data Protection — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Compliance officers, data protection officers, developers

---

## 1. Data Classification

### 1.1 Classification Levels

| Level | Description | Examples | Protection Required |
|-------|-------------|----------|---------------------|
| **Public** | Non-sensitive information | Product names, prices, public catalogs | Integrity |
| **Internal** | Business data not for public | Sales records, repair logs, supplier info | Access control, audit |
| **Confidential** | Sensitive business data | Financial reports, employee data | Encryption, RBAC, audit |
| **Sensitive** | Personal data, credentials | Customer PII, passwords, JWT secrets | Encryption at rest/transit, strict access |

### 1.2 Data Inventory

| Data Type | Classification | Storage Location | Retention |
|-----------|---------------|------------------|-----------|
| Product names, prices | Public | PostgreSQL | Indefinite |
| Sales transactions | Internal | PostgreSQL | 7 years (tax) |
| Repair records | Internal | PostgreSQL | 3 years |
| Supplier information | Internal | PostgreSQL | Indefinite |
| Customer email/phone | Confidential | PostgreSQL | Until deletion request |
| User passwords | Sensitive | PostgreSQL (hashed) | Until account deletion |
| JWT secrets | Sensitive | Filesystem (encrypted) | Until rotation |
| Audit logs | Internal | PostgreSQL | 7 years |
| Backups | Confidential | Filesystem (encrypted) | 7 days rolling |

---

## 2. Personal Data Handling

### 2.1 Personal Data Collected

| Field | Purpose | Legal Basis | Storage |
|-------|---------|-------------|---------|
| Customer email | Communication, receipts | Contract performance | PostgreSQL |
| Customer phone | Service updates | Contract performance | PostgreSQL |
| Customer name | Identification | Contract performance | PostgreSQL |
| User email | Authentication | Contract performance | PostgreSQL |
| User name | Identification | Contract performance | PostgreSQL |

### 2.2 Data Processing Principles

| Principle | Implementation |
|-----------|---------------|
| Purpose limitation | Data collected only for workshop management |
| Data minimization | Only necessary fields collected |
| Accuracy | Users can update their own data |
| Storage limitation | Retention policies enforced |
| Integrity & confidentiality | Encryption at rest and in transit |
| Accountability | Audit trail of all data access |

### 2.3 Consent and Legal Basis

- **Contract performance**: Customer data processed for service delivery
- **Legal obligation**: Financial data retained for tax compliance (7 years)
- **Legitimate interest**: Audit logs for security and compliance

---

## 3. Data Retention Policies

### 3.1 Retention Schedule

| Data Type | Retention Period | Legal Basis | Deletion Method |
|-----------|-----------------|-------------|-----------------|
| Sales transactions | 7 years | Chilean tax law | Automated purge |
| Repair records | 3 years | Business need | Automated purge |
| Customer PII | Until deletion request | GDPR/Ley 19.628 | Soft-delete |
| User accounts | Until deletion request | User rights | Soft-delete |
| Audit logs | 7 years | Compliance | Automated purge |
| Backups | 7 days rolling | Business continuity | Automated rotation |
| Rate limiter data | 5 minutes | Security | In-memory expiry |

### 3.2 Automated Purge

- **Backups**: 7-day rolling window, oldest deleted automatically
- **Rate limiter**: In-memory, resets on server restart
- **Audit logs**: Configurable retention (currently 7 years)

---

## 4. Right to Deletion

### 4.1 Soft-Delete Implementation

WorkshopManager uses soft-delete for user and customer data:

```sql
-- Users table
ALTER TABLE users ADD COLUMN deleted_at TIMESTAMP NULL;

-- Products table
ALTER TABLE products ADD COLUMN deleted_at TIMESTAMP NULL;

-- Customers table (if exists)
ALTER TABLE customers ADD COLUMN deleted_at TIMESTAMP NULL;
```

### 4.2 Deletion Process

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Deletion    │────▶│  Soft-Delete │────▶│  Anonymize   │────▶│  Physical    │
│  Request     │     │  (set NULL)  │     │  PII Fields  │     │  Purge       │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

### 4.3 Deletion Rules

| Data Type | Soft-Delete | Anonymization | Physical Purge |
|-----------|-------------|---------------|----------------|
| User account | Yes | Email → `deleted_<hash>` | After 30 days |
| Customer PII | Yes | Name, email, phone → anonymized | After 30 days |
| Products | Yes | Name → `DELETED_<id>` | After 30 days |
| Sales records | No (legal retention) | N/A | After 7 years |

### 4.4 Deletion Request Process

1. User submits deletion request via API
2. System verifies identity (JWT required)
3. Soft-delete flag set (`deleted_at = NOW()`)
4. PII fields anonymized immediately
5. Physical purge after 30-day grace period
6. Audit log entry created

---

## 5. Data Encryption

### 5.1 Encryption at Rest

| Data Type | Algorithm | Implementation |
|-----------|-----------|---------------|
| Passwords | Argon2id (hashing) | `auth.rs:13` |
| JWT secret | AES-256-GCM | `crypto.rs:40` |
| Database credentials | AES-256-GCM | `crypto.rs:40` |
| Backups | AES-256-GCM | `backup.rs:10` |
| Device keys | SHA-256 (hashing) | `device_key.rs:14` |

### 5.2 Encryption in Transit

| Channel | Protocol | Implementation |
|---------|----------|---------------|
| Client ↔ Server | TLS 1.3 | `tls.rs` |
| Server ↔ Database | Localhost (no network) | Embedded PostgreSQL |

### 5.3 Key Management

| Key Type | Generation | Storage | Rotation |
|----------|-----------|---------|----------|
| AES-256 crypto key | OsRng CSPRNG | Filesystem (encrypted) | Not implemented |
| JWT signing secret | OsRng CSPRNG | Filesystem (encrypted) | Not implemented |
| TLS certificates | rcgen | Filesystem | Self-signed (manual) |
| Database password | OsRng CSPRNG | Filesystem (encrypted) | Not implemented |

---

## 6. Backup Security

### 6.1 Backup Encryption

All backups are encrypted before storage:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  pg_dump     │────▶│  gzip        │────▶│  AES-256-GCM │────▶│  Store       │
│  (SQL dump)  │     │  (compress)  │     │  (encrypt)   │     │  (filesystem)│
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

### 6.2 Backup Access Controls

| Control | Implementation |
|---------|---------------|
| File permissions | Restricted to service account |
| Encryption key | Same as database credentials |
| Location | `$LOCALAPPDATA/WorkshopManager/data/backups/` |
| Retention | 7 days rolling window |

### 6.3 Backup Integrity

- Backups are compressed and encrypted
- No integrity verification checksums (backlog item)
- Restore process tested periodically

---

## 7. Cross-Border Data Transfer

### 7.1 Data Residency

WorkshopManager is designed for local deployment:

| Aspect | Implementation |
|--------|---------------|
| Deployment | Local desktop application |
| Database | Embedded PostgreSQL (local) |
| Network | localhost only (no external connections) |
| Cloud | No cloud services required |

### 7.2 Cross-Border Considerations

- **No cross-border transfer**: All data stays on local machine
- **No cloud services**: No third-party data processing
- **No external API calls**: Server only communicates with local viewer

### 7.3 Future Considerations

If cloud deployment is added:
- Data residency requirements for Chile
- GDPR compliance for EU customers
- Data processing agreements with cloud providers

---

## 8. Chilean Data Protection Law (Ley 19.628)

### 8.1 Compliance Checklist

| Requirement | Status | Implementation |
|-------------|--------|---------------|
| Lawful basis for processing | Implemented | Contract performance, legal obligation |
| Purpose limitation | Implemented | Data collected only for workshop management |
| Data minimization | Implemented | Only necessary fields collected |
| Accuracy | Implemented | Users can update their data |
| Storage limitation | Implemented | Retention policies enforced |
| Security measures | Implemented | Encryption, RBAC, audit trail |
| Data subject rights | Partial | Soft-delete implemented, right to access planned |
| Breach notification | Not implemented | Backlog item |

### 8.2 Data Subject Rights

| Right | Implementation | Status |
|-------|---------------|--------|
| Right to access | API endpoint to retrieve own data | Planned |
| Right to rectification | Users can update own profile | Implemented |
| Right to deletion | Soft-delete with anonymization | Implemented |
| Right to portability | Export own data as JSON | Planned |
| Right to object | Opt-out of non-essential processing | N/A (all processing essential) |

### 8.3 Data Protection Officer

- **Role**: Not required for small businesses (< 200 employees)
- **Contact**: Workshop administrator
- **Responsibilities**: Handle data subject requests, maintain records

### 8.4 Data Processing Records

All data processing activities are documented:
- **Audit trail**: All CRUD operations logged
- **Data inventory**: This document
- **Retention schedule**: Section 3.1
- **Security measures**: Chapter 5

---

## 9. Data Breach Response

### 9.1 Breach Detection

| Indicator | Detection Method |
|-----------|-----------------|
| Unauthorized access | Audit log anomalies |
| Data exfiltration | Unusual backup access patterns |
| System compromise | Integrity checks |

### 9.2 Response Process

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Detection   │────▶│  Assessment  │────▶│  Containment │────▶│  Notification│
│              │     │              │     │              │     │              │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

### 9.3 Notification Requirements

- **Chilean law**: 72-hour notification to data protection authority
- **Affected individuals**: Without undue delay if high risk
- **Documentation**: All breaches documented regardless of notification

---

## 10. Cross-References

| Document | Description |
|----------|-------------|
| [Security Overview](../04-security/overview.md) | Security architecture |
| [Encryption](../04-security/encryption.md) | Cryptographic mechanisms |
| [Audit Trail](../04-security/audit.md) | Audit logging |
| [Backup Recovery](../05-deployment/backup-recovery.md) | Backup procedures |
| [OWASP Top 10](./owasp-top10.md) | Security compliance |
| [CONSTITUCION.md](../../CONSTITUCION.md) | Constitutional rules |
