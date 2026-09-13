# Device Keys

Base URL: `https://localhost:8443/api/device-keys`

All endpoints require admin authentication.

Device keys are used to authorize specific devices to access the API. When `require_device_key=true` in the server config, every request must include a valid device key in the `X-WorkshopManager-Device-Key` header.

---

## GET /api/device-keys

List all device keys.

**Auth level**: Admin

### Response (200)

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "bound_ip": "192.168.1.100",
      "active": true,
      "created_at": "2025-01-15T10:30:00Z",
      "last_seen_at": "2025-01-17T14:22:00Z"
    },
    {
      "id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
      "bound_ip": null,
      "active": false,
      "created_at": "2025-01-16T09:00:00Z",
      "last_seen_at": null
    }
  ]
}
```

| Field         | Type     | Description                                      |
|---------------|----------|--------------------------------------------------|
| `id`          | UUID     | Device key ID                                    |
| `bound_ip`    | string   | IP address the key is bound to (if any)          |
| `active`      | boolean  | Whether the key is currently active              |
| `created_at`  | datetime | When the key was generated                       |
| `last_seen_at`| datetime | Last time this key was used in a request (nullable)|

### cURL

```bash
curl -k -X GET https://localhost:8443/api/device-keys \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## POST /api/device-keys

Generate a new device key. The key is only returned once — store it securely.

**Auth level**: Admin

### Response (200)

```json
{
  "success": true,
  "data": {
    "key": "wm_550e8400e29b41d4a716446655440000"
  }
}
```

### Key Format

- Prefix: `wm_`
- Body: UUID without hyphens (32 hex characters)
- Example: `wm_550e8400e29b41d4a716446655440000`

### Business Rules

- The key is stored as a SHA-256 hash in the database (the plaintext key is never persisted)
- The key is only returned in the creation response
- A new key starts as `active` with no IP binding

### cURL

```bash
curl -k -X POST https://localhost:8443/api/device-keys \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## POST /api/device-keys/:id/revoke

Revoke a device key (sets `active` to `false`). The key can no longer be used for API requests.

**Auth level**: Admin

### Path Parameters

| Parameter | Type | Description     |
|-----------|------|-----------------|
| `id`      | UUID | Device Key ID   |

### Response (200)

```json
{
  "success": true,
  "data": null
}
```

### Error Responses

| Status | Error      | Condition              |
|--------|------------|------------------------|
| 404    | Not Found  | Device key not found   |

### Business Rules

- Revoked keys cannot be reactivated (use unbind + new key generation instead)
- The key hash remains in the database
- Audit log records the revocation

### cURL

```bash
curl -k -X POST https://localhost:8443/api/device-keys/550e8400-e29b-41d4-a716-446655440000/revoke \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## POST /api/device-keys/:id/unbind

Unbind a device key from its IP address (sets `bound_ip` to `NULL`). The key remains active but is no longer tied to a specific IP.

**Auth level**: Admin

### Path Parameters

| Parameter | Type | Description     |
|-----------|------|-----------------|
| `id`      | UUID | Device Key ID   |

### Response (200)

```json
{
  "success": true,
  "data": null
}
```

### Error Responses

| Status | Error      | Condition              |
|--------|------------|------------------------|
| 404    | Not Found  | Device key not found   |

### Business Rules

- The key remains active after unbinding
- The key can be used from any IP after unbinding
- Useful when a device changes network or IP

### cURL

```bash
curl -k -X POST https://localhost:8443/api/device-keys/550e8400-e29b-41d4-a716-446655440000/unbind \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## Device Key Lifecycle

```
Generate → Active (unbound) → First request → Active (bound to IP) → Revoke → Inactive
                                     ↑                                    |
                                     └────────── Unbind ─────────────────┘
```

| State    | Description                                      |
|----------|--------------------------------------------------|
| Active   | Key can be used for API requests                 |
| Inactive | Key has been revoked, cannot be used              |
| Unbound  | Key has no IP restriction (can be used from anywhere) |
| Bound    | Key is restricted to a specific IP address        |
