# License Tool — CLI Reference

**Version:** 0.1.0
**Status:** Implemented
**Last Updated:** 2026-09-14
**Audience:** Vendors (license generators)

---

## 1. Overview

`license-tool` is a command-line utility for generating, verifying, and migrating WorkshopManager licenses. It uses Ed25519 signatures to ensure only the vendor can create valid licenses.

---

## 2. Installation

```powershell
# Build from source
cargo build --release -p license-tool

# Binary location
target\release\license-tool.exe
```

---

## 3. Commands

### 3.1 generate-keypair

Generates an Ed25519 key pair for license signing.

```powershell
license-tool generate-keypair --secret secret_key.bin --public public_key.bin
```

| Flag | Default | Description |
|------|---------|-------------|
| `--secret` | `secret_key.bin` | Output file for the secret key |
| `--public` | `public_key.bin` | Output file for the public key |

**Output:**
```
Claves generadas:
  Secreta: secret_key.bin
  Pública: public_key.bin

Pública (hex para embeber en el server):
  [32, 145, 78, ...]
```

**Important:** The secret key must be kept secure and never distributed. The public key is embedded in the server binary.

---

### 3.2 generate

Generates a signed license for a specific machine.

```powershell
license-tool generate `
  --key "ABC-123" `
  --hw "a3f8b2c1d4e5f6..." `
  --tier base `
  --max-viewers 2 `
  --max-transfers 3 `
  --secret-key secret_key.bin `
  --output license.dat
```

| Flag | Default | Description |
|------|---------|-------------|
| `--key` | *(required)* | Human-readable license key |
| `--hw` | *(required)* | Hardware hash from client's machine |
| `--tier` | `base` | License tier: `trial`, `base`, `reports`, `advanced`, `api` |
| `--max-viewers` | *(tier default)* | Maximum concurrent viewers |
| `--max-transfers` | *(tier default)* | Maximum license migrations |
| `--secret-key` | `secret_key.bin` | Path to vendor's secret key |
| `--output` | `license.dat` | Output license file |

**Output:**
```
Licencia generada:
  Key: ABC-123
  Tier: Base
  Viewers: 2
  Migraciones: 3
  Hardware: a3f8b2c1d4e5f6...
  Archivo: license.dat
```

---

### 3.3 verify

Verifies a license file against the vendor's public key.

```powershell
license-tool verify `
  --input license.dat `
  --public-key public_key.bin
```

| Flag | Default | Description |
|------|---------|-------------|
| `--input` | `license.dat` | License file to verify |
| `--public-key` | `public_key.bin` | Vendor's public key |

**Output (valid):**
```
Licencia válida:
  Key: ABC-123
  Tier: Base
  Viewers: 2
  Migraciones: 0/3
  Hardware: a3f8b2c1d4e5f6...
  Activada: 2026-09-14 12:00:00 UTC
```

**Output (invalid):**
```
Licencia inválida: Firma inválida
```

---

### 3.4 migrate

Migrates a license to a new machine (increments transfer count).

```powershell
license-tool migrate `
  --key "ABC-123" `
  --new-hw "x9y2z1a2b3c4..." `
  --secret-key secret_key.bin `
  --output license.dat
```

| Flag | Default | Description |
|------|---------|-------------|
| `--key` | *(required)* | License key |
| `--new-hw` | *(required)* | New machine's hardware hash |
| `--secret-key` | `secret_key.bin` | Path to vendor's secret key |
| `--output` | `license.dat` | Output license file |

**Output:**
```
Licencia migrada:
  Key: ABC-123
  Tier: Base
  Nueva hardware: x9y2z1a2b3c4...
  Migración: 1/3
  Archivo: license.dat
```

---

### 3.5 hardware-id

Displays the hardware fingerprint of the current machine.

```powershell
license-tool hardware-id
```

**Output:**
```
Hardware ID: a3f8b2c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0
Código de activación: a3f8b2c1d4e5f6a7
```

The activation code (first 16 characters) is what the client sends to the vendor.

---

## 4. Workflow

### Initial Setup (Vendor)

```powershell
# 1. Generate key pair (once)
license-tool generate-keypair

# 2. Embed public key in server binary
# Edit: crates/workshop-server/src/license.rs
# Replace VENDOR_PUBLIC_KEY constant with the generated public key bytes

# 3. Keep secret key secure (never share)
```

### License Generation

```powershell
# 1. Client runs: license-tool hardware-id
# 2. Client sends you the activation code
# 3. Generate license:
license-tool generate --key "CUSTOMER-001" --hw "client_hash" --tier base
# 4. Send license.dat to client
# 5. Client copies license.dat to server data directory
```

### License Migration

```powershell
# 1. Client's PC breaks
# 2. Client runs license-tool hardware-id on new PC
# 3. Migrate:
license-tool migrate --key "CUSTOMER-001" --new-hw "new_hash"
# 4. Send new license.dat to client
# 5. Old PC invalid on restart
```

---

## 5. Tier Defaults

| Tier | max_viewers | max_transfers |
|------|-------------|---------------|
| Trial | 1 | 0 |
| Base | 2 | 3 |
| Reports | 5 | 5 |
| Advanced | 10 | 10 |
| API | 999 | 999 |

---

## 6. Error Handling

| Error | Cause | Solution |
|-------|-------|----------|
| `Error leyendo clave secreta` | File not found | Generate keypair first |
| `Error firmando licencia` | Invalid secret key | Check key file |
| `Clave pública inválida` | Wrong key size (need 32 bytes) | Regenerate keypair |
| `Licencia inválida: Firma inválida` | Wrong public key or corrupted file | Verify key matches |
