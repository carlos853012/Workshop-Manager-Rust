# Common API Patterns

Base URL: `https://localhost:8443`

All responses use HTTPS with self-signed TLS certificates.

---

## Response Envelope

Every API response is wrapped in a standard envelope:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

On error:

```json
{
  "success": false,
  "data": null,
  "error": "Error message"
}
```

---

## Pagination

Most list endpoints accept query parameters for pagination:

| Parameter  | Type   | Default | Constraints   | Description              |
|------------|--------|---------|---------------|--------------------------|
| `page`     | i32    | 1       | `>= 1`       | Page number (1-indexed)  |
| `per_page` | i32    | 20      | `1..=100`    | Items per page           |

Paginated responses use this structure:

```json
{
  "success": true,
  "data": {
    "items": [ ... ],
    "total": 150,
    "page": 1,
    "per_page": 20
  }
}
```

The `total` field reflects the total count of matching records across all pages.

---

## Authentication Headers

All protected endpoints require three headers:

| Header                       | Required | Description                                      |
|------------------------------|----------|--------------------------------------------------|
| `Authorization`              | Yes      | `Bearer <jwt_token>`                             |
| `X-WorkshopManager-Key`      | Yes      | Shared API key (matches server config)           |
| `X-WorkshopManager-Device-Key` | Conditional | Device key (required if `require_device_key=true` in config) |

### Header format

```
Authorization: Bearer eyJhbGciOiJIUzI1NiJ9...
X-WorkshopManager-Key: your-api-key-here
X-WorkshopManager-Device-Key: wm_<uuid>
```

Public endpoints (`/api/auth/login`, `/api/auth/register`) do not require any headers.

---

## Error Codes

| HTTP Status | Error Type        | Description                                                |
|-------------|-------------------|------------------------------------------------------------|
| 400         | Bad Request       | Malformed request body or invalid JSON                     |
| 401         | Unauthorized      | Missing, invalid, or expired JWT; invalid API key          |
| 403         | Forbidden         | Insufficient permissions (requires admin role)             |
| 404         | Not Found         | Resource does not exist or belongs to another workshop     |
| 409         | Conflict          | Duplicate email, insufficient stock, or business conflict  |
| 422         | Validation Error  | Input validation failed (bad email, short password, etc.)  |
| 429         | Too Many Requests | Rate limit exceeded (login)                                |
| 500         | Internal Error    | Server-side failure (DB error, etc.) — sanitized message   |

### Validation Error Details

Validation errors return the specific field problem in the `error` field:

```json
{
  "success": false,
  "error": "name must be between 1 and 200 characters"
}
```

---

## Rate Limiting

### Login Rate Limiting

- **Key**: `login:<email>`
- **Max attempts**: 5 per 300-second window
- **Response on limit**: HTTP 429

```json
{
  "success": false,
  "error": "Too many requests"
}
```

The window resets after 300 seconds from the first attempt in the window.

---

## Content Types

- **Request**: `Content-Type: application/json` (required for POST/PUT)
- **Response**: `Content-Type: application/json`
- **PDF download**: `Content-Type: application/pdf` (certificate endpoint only)
- **Body limit**: 10 MB maximum

---

## CORS Configuration

| Setting       | Value                                                           |
|---------------|-----------------------------------------------------------------|
| Origins       | `http://localhost`, `https://localhost`, `http://127.0.0.1`, `https://127.0.0.1` |
| Methods       | GET, POST, PUT, DELETE                                          |
| Headers       | `Authorization`, `Content-Type`                                 |
| Credentials   | Not allowed                                                     |

---

## Security Headers

Every response includes:

| Header                 | Value                                         |
|------------------------|-----------------------------------------------|
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains`     |
| `X-Content-Type-Options`    | `nosniff`                                |
| `X-Frame-Options`           | `DENY`                                   |

---

## Health Check

```
GET /health
```

Response (not JSON, plain text):

```
OK
```

Status: 200

---

## Role-Based Access Control

| Role      | Capabilities                                                  |
|-----------|---------------------------------------------------------------|
| `admin`   | Full access: all endpoints including user/device-key mgmt     |
| `seller`  | Products (CRUD), Sales (create/read), Repairs (read/update), Suppliers (read), Reports |
| `mechanic`| Repairs (read/update), Products (read), Reports (read)       |

Write operations for products/sales require `admin` or `seller` role.
User and device-key management require `admin` role.
