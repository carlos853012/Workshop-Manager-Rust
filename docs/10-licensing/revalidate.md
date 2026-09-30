# POST /api/v1/licenses/revalidate

**Service:** workshop-license-worker (Cloudflare Worker)
**Status:** Implemented (Worker + Rust client) — GAP-1/GAP-3/GAP-6, Fase 5.4
**Last Updated:** 2026-09-30
**Audience:** Developers (server/client integration)

---

## 1. Overview

`/revalidate` lets the client verify that a license is **still valid on the vendor side** and obtain a **freshly signed blob**. Unlike `/validate` (legacy, returns only `valid: true/false`), `/revalidate`:

- Returns a **structured reason** for each failure mode (5 cases).
- Applies **rate limiting** (same pattern as `/activate`).
- **Logs every attempt** to `activation_attempts`.
- On success, **re-signs** the license with Ed25519 and `issued_at = now`.

> **Client integration (5.4-B/C): DONE** — `workshop-server` calls this endpoint opportunistically every 24 h (first attempt ~5 min after boot). Design is **offline-first**: if the Worker is unreachable, the attempt is skipped silently (no grace period, no shutdown); only an explicit `revoked`/`expired`/`invalid` answer shuts the server down.

---

## 2. Request

```
POST /api/v1/licenses/revalidate
Content-Type: application/json
```

```json
{
  "license_key": "AAAA-BBBB-CCCC-DDDD",
  "hardware_hash": "a3f8b2c1..."
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `license_key` | string | Yes | License key to revalidate |
| `hardware_hash` | string | Yes | Current machine hardware hash (SHA-256, 64 hex chars) |

Missing fields → `400 { "error": "Missing license_key or hardware_hash" }`.

---

## 3. Response

All 5 logical outcomes return **HTTP 200**; the client must inspect the `estado` field.

### Decision order (per spec)

```
not_found → revoked → expired → hardware_mismatch → active
```

| Case | Response body | Logged as |
|------|---------------|-----------|
| License does not exist | `{ "estado": "invalid", "reason": "not_found" }` | `success=0`, `error_message=not_found` |
| `active = 0` (revoked) | `{ "estado": "revoked", "reason": "revoked" }` | `success=0`, `error_message=revoked` |
| `expires_at < now` | `{ "estado": "expired", "expires_at": "2026-09-20T00:00:00.000Z" }` | `success=0`, `error_message=expired` |
| Hash differs (or `hardware_hash IS NULL`) | `{ "estado": "invalid", "reason": "hardware_mismatch" }` | `success=0`, `error_message=hardware_mismatch` |
| All checks pass | `{ "estado": "active", "signed_license": "<base64>" }` | `success=1` |

### Non-200 responses

| Status | Body | When |
|--------|------|------|
| `400` | `{ "error": "Missing license_key or hardware_hash" }` | Invalid body |
| `429` | `{ "error": "Too many failed attempts" }` | Rate limited (see §4) |
| `500` | `{ "error": "Signing failed" }` | Ed25519 signing error (check `VENDOR_SECRET_KEY`) |

---

## 4. Rate limiting

Identical pattern to `/activate` (H8): count **failures only**, per `license_key`, last hour.

```sql
SELECT COUNT(*) FROM activation_attempts
WHERE license_key = ? AND success = 0
  AND created_at > datetime('now', '-1 hour')
```

- ≥ 5 failures → `429`. The blocked attempt is also logged (`success=0`).
- **Shared counter:** failures from `/revalidate` also count toward `/activate`'s limit for the same key (and vice versa). Intentional — same key, same abuse budget.

---

## 5. Signed blob (success case)

Wire format is **unchanged**:

```
[4 bytes LE length][JSON payload][64 bytes Ed25519 signature]
```

The payload is the standard license object plus one extra field:

```json
{
  "license_key": "AAAA-BBBB-CCCC-DDDD",
  "tier": "Base",
  "hardware_hash": "a3f8b2c1...",
  "max_viewers": 2,
  "max_transfers": 3,
  "transfer_count": 0,
  "activated_at": "2026-09-01T00:00:00.000Z",
  "expires_at": null,
  "issued_at": "2026-09-24T12:00:00.000Z"
}
```

**Backward compatibility:** the Rust `License` struct (`workshop-common/src/features.rs`) does **not** use `#[serde(deny_unknown_fields)]`, so `issued_at` is silently ignored by current clients. `activated_at` remains the original activation date; `issued_at` is minted fresh on every successful revalidation (used by 5.4-B for renewal logic).

---

## 6. Examples

### Success

```bash
curl -X POST https://<worker>/api/v1/licenses/revalidate \
  -H "Content-Type: application/json" \
  -d '{"license_key":"AAAA-BBBB-CCCC-DDDD","hardware_hash":"a3f8b2c1..."}'
```

```json
{ "estado": "active", "signed_license": "NCj+Fw..." }
```

### Revoked

```json
{ "estado": "revoked", "reason": "revoked" }
```

### Expired

```json
{ "estado": "expired", "expires_at": "2026-09-20T00:00:00.000Z" }
```

---

## 7. Client integration status

| Piece | Status |
|-------|--------|
| Worker endpoint | **Done** (this change) |
| Unit tests (`evaluateRevalidate`) | **Done** — `src/revalidate.test.ts`, 8 tests |
| Rust caller (`/revalidate` every 24h) | **Done** — `license::revalidate_online()` + task 7c in `main.rs` |
| Grace period on `Unreachable` | **N/A by design** — offline-first: unreachable never shuts down |
| Revocation detected in client | **Done** — `revalidation_fatal()` shows dialog + graceful shutdown + `exit(1)` |

---

## 8. Related Documents

| Document | Description |
|----------|-------------|
| [overview.md](overview.md) | Licensing system overview |
| [license-tool.md](license-tool.md) | CLI reference for offline licenses |
| [../../docs/CLOUDFLARE.md](../CLOUDFLARE.md) | Worker deployment reference |
