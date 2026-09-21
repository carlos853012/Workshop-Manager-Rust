# Audit Log

Base URL: `https://localhost:8443/api/audit`

All endpoints require admin authentication.

---

## GET /api/audit

List audit log entries with pagination.

**Auth level**: Admin only

### Query Parameters

| Parameter  | Type | Default | Constraints |
|------------|------|---------|-------------|
| `page`     | i32  | 1       | `>= 1`      |
| `per_page` | i32  | 20      | `1..=100`   |

### Response (200)

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": 42,
        "user_id": "550e8400-e29b-41d4-a716-446655440000",
        "action": "cancel",
        "entity_type": "sale",
        "entity_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
        "old_values": { "status": "completed" },
        "new_values": null,
        "ip_address": "192.168.1.100",
        "user_agent": "Mozilla/5.0 ...",
        "created_at": "2025-01-15T14:30:00Z"
      }
    ],
    "total": 156,
    "page": 1,
    "per_page": 20
  }
}
```

| Field         | Type          | Description |
|---------------|---------------|-------------|
| `id`          | i64           | Auto-incrementing audit log ID |
| `user_id`     | UUID or null  | The user who performed the action |
| `action`      | string        | Action: `"create"`, `"update"`, `"delete"`, `"cancel"` |
| `entity_type` | string or null | Entity: `"sale"`, `"product"`, `"repair"`, `"supplier"`, `"user"` |
| `entity_id`   | UUID or null  | The affected entity's ID |
| `old_values`  | JSON or null  | Previous state (sensitive fields redacted) |
| `new_values`  | JSON or null  | New state (sensitive fields redacted) |
| `ip_address`  | string or null | Client IP address |
| `user_agent`  | string or null | Client user agent |
| `created_at`  | datetime      | When the action occurred |

Results are ordered by `created_at DESC` (newest first).

### Status Codes

| Status | Condition |
|--------|-----------|
| 200 | Success |
| 401 | JWT missing, invalid, or expired |
| 403 | User is not admin |
| 422 | Invalid pagination parameters |

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/audit?page=1&per_page=50" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```
