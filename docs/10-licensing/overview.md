# Licensing System — WorkshopManager

**Version:** 0.1.0
**Status:** Implemented
**Last Updated:** 2026-09-14
**Audience:** Vendors, developers, operations

---

## 1. Overview

WorkshopManager uses a **hardware-bound, Ed25519-signed licensing system** to control feature access and prevent unauthorized software distribution. The system works offline after initial activation and supports license migration between machines.

---

## 2. License Tiers

| Tier | Viewers | Transfers | Price Model | Features |
|------|---------|-----------|-------------|----------|
| **Trial** | 1 | 0 | Free (7 days) | Inventory, Sales, Repairs, Suppliers, Dashboard, Auth, AuditLog |
| **Base** | 2 | 3 | One-time | All Trial + MultiViewer |
| **Reports** | 5 | 5 | DLC | All Base + PdfReports, ClientHistory, ExcelExport |
| **Advanced** | 10 | 10 | DLC | All Reports + AdvancedAnalytics, AutoBackup, MultiWorkshop |
| **API** | 999 | 999 | DLC | All Advanced + RestApi, Webhooks, Integrations |

---

## 3. Feature Flags

```rust
// 17 features across 5 tiers
pub enum Feature {
    // Base tier (7)
    Inventory, Sales, Repairs, Suppliers, Dashboard, Auth, AuditLog,
    // Reports DLC (3)
    PdfReports, ClientHistory, ExcelExport,
    // Advanced DLC (3)
    AdvancedAnalytics, AutoBackup, MultiWorkshop,
    // API DLC (3)
    RestApi, Webhooks, Integrations,
    // Multi-viewer (1)
    MultiViewer,
}
```

---

## 4. Hardware Binding

### Fingerprinting

The system extracts a hardware fingerprint from three components:

| Component | Source (Windows) |
|-----------|-----------------|
| CPU | `wmic cpu get ProcessorId` |
| Motherboard | `wmic baseboard get SerialNumber` |
| Disk | `wmic diskdrive get SerialNumber` |

These are combined and hashed with SHA-256 to produce a 64-character hex string.

### Binding Flow

```
Server start → Extract hardware hash → Compare with license.hardware_hash
  ↓ Match → License valid
  ↓ No match → License invalid → Trial mode or reject
```

---

## 5. Ed25519 Signing

### Key Management

| Key | Purpose | Storage |
|-----|---------|---------|
| **Secret key** (32 bytes) | Signs licenses | Vendor only (never in binary) |
| **Public key** (32 bytes) | Verifies licenses | Embedded in server binary |

### License File Format (`license.dat`)

```
[4 bytes]  JSON length (little-endian u32)
[N bytes]  License JSON (serialized)
[64 bytes] Ed25519 signature
```

### Signing Flow

```
1. Vendor creates License struct with tier, hardware_hash, max_viewers
2. Serialize to JSON
3. Sign JSON with Ed25519 secret key
4. Write [len][json][signature] to license.dat
5. Send license.dat to client
```

### Verification Flow

```
1. Server reads license.dat
2. Extract JSON and signature
3. Verify signature with embedded public key
4. Compare hardware_hash with current machine
5. Validate tier and features
```

---

## 6. License Lifecycle

### First Run (No License)

```
1. Server starts
2. No license.dat found
3. Extract hardware hash → display activation code
4. Try online validation (if internet available)
   → Success: Save license, continue
   → Failure: Create trial license (7 days)
5. Server runs in trial mode
```

### Activation (Vendor Action)

```
1. Client sends activation code (first 16 chars of hardware hash)
2. Vendor generates license with license-tool
3. Vendor sends license.dat to client
4. Client copies license.dat to server data directory
5. Server validates on next restart
```

### Migration

```
1. Client reports hardware failure
2. Vendor requests new activation code from new machine
3. Vendor runs: license-tool migrate --key ABC-123 --new-hw "new_hash"
4. New license.dat generated with incremented transfer_count
5. Old machine invalid on restart (hardware mismatch)
```

---

## 7. Online Validation

On first run, the server attempts to validate the license online:

```
POST https://tudominio.com/api/validate
{
  "license_key": "ABC-123",
  "hardware_hash": "a3f8b2c1..."
}

Response:
{
  "valid": true,
  "tier": "base",
  "max_viewers": 2,
  "features": ["inventory", "sales", ...]
}
```

If offline, the server falls back to trial mode.

---

## 8. Trial License

- Created automatically on first run
- No internet required
- Expires after 7 days
- Max 1 viewer, no transfers
- All base features enabled

---

## 9. Anti-Piracy Measures

| Measure | Protection Level |
|---------|-----------------|
| Hardware binding | Prevents copying to different PC |
| Ed25519 signatures | Prevents license forgery |
| Max transfers | Limits migration abuse |
| Online validation | Prevents multi-PC activation |
| Periodic re-validation | Detects license migration at restart |

### Limitations (Offline)

| Scenario | Detection |
|----------|-----------|
| Copy to same hardware | Prevented (hash match) |
| Copy to different hardware | Prevented (hash mismatch) |
| 2 PCs running simultaneously | Partial (re-validation at restart) |
| Never restart old PC | Not detectable offline |

---

## 10. Configuration

### server.toml

```toml
[server]
max_viewers = 2  # Overridden by license tier
```

The `max_viewers` field is read from the license, not from config. The config value is used as fallback if no license is loaded.

---

## 11. Related Documents

| Document | Description |
|----------|-------------|
| [license-tool.md](license-tool.md) | CLI reference for license generation |
| [../01-architecture/overview.md](../01-architecture/overview.md) | Architecture decision D5 |
| [../05-deployment/configuration.md](../05-deployment/configuration.md) | Server configuration |
