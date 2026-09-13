# Users

Base URL: `https://localhost:8443/api/users`

All endpoints require admin authentication.

---

## GET /api/users

List all users in the workshop (including inactive). Password hashes are stripped from responses.

**Auth level**: Admin

### Query Parameters

| Parameter  | Type | Default | Constraints  |
|------------|------|---------|--------------|
| `page`     | i32  | 1       | `>= 1`      |
| `per_page` | i32  | 20      | `1..=100`   |

### Response (200)

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "email": "admin@workshop.cl",
        "display_name": "Carlos Admin",
        "role": "admin",
        "status": "active",
        "created_at": "2025-01-15T10:30:00Z"
      },
      {
        "id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
        "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "email": "mekaniko@workshop.cl",
        "display_name": "Pedro Mecanico",
        "role": "mechanic",
        "status": "active",
        "created_at": "2025-01-16T09:00:00Z"
      }
    ],
    "total": 3,
    "page": 1,
    "per_page": 20
  }
}
```

**Note**: `password_hash` is always an empty string in responses (stripped server-side).

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/users?page=1&per_page=20" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## POST /api/users

Create a new user in the workshop.

**Auth level**: Admin

### Request Body

```json
{
  "email": "vendedor@workshop.cl",
  "display_name": "Maria Vendedora",
  "password": "password123",
  "role": "seller"
}
```

| Field          | Type   | Required | Constraints                          |
|----------------|--------|----------|--------------------------------------|
| `email`        | string | Yes      | Valid email, max 200 chars           |
| `display_name` | string | No       | Free text                            |
| `password`     | string | Yes      | Min 8 characters                     |
| `role`         | enum   | Yes      | `admin`, `mechanic`, or `seller`     |

### Response (200)

```json
{
  "success": true,
  "data": {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "email": "vendedor@workshop.cl",
    "display_name": "Maria Vendedora",
    "role": "seller",
    "status": "active",
    "created_at": "2025-01-17T11:00:00Z"
  }
}
```

### Error Responses

| Status | Error            | Condition                       |
|--------|------------------|----------------------------------|
| 409    | Conflict         | Email already registered         |
| 422    | Validation error | Invalid email or short password  |

### User Roles

| Role       | Capabilities                                              |
|------------|-----------------------------------------------------------|
| `admin`    | Full access: all endpoints, user/device-key management    |
| `seller`   | Products (CRUD), Sales (create/read), Reports (read)     |
| `mechanic` | Repairs (read/update), Products (read), Reports (read)   |

### cURL

```bash
curl -k -X POST https://localhost:8443/api/users \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  -H "Content-Type: application/json" \
  -d '{
    "email": "vendedor@workshop.cl",
    "display_name": "Maria Vendedora",
    "password": "password123",
    "role": "seller"
  }'
```

---

## GET /api/users/:id

Get a single user by ID.

**Auth level**: Admin

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | User ID     |

### Response (200)

Returns the user object (password hash stripped).

### Error Responses

| Status | Error      | Condition          |
|--------|------------|--------------------|
| 404    | Not Found  | User not found     |

### cURL

```bash
curl -k -X GET https://localhost:8443/api/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## PUT /api/users/:id

Update a user's display name, role, or status. An admin cannot demote themselves.

**Auth level**: Admin

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | User ID     |

### Request Body

```json
{
  "display_name": "Maria V. Actualizada",
  "role": "mechanic",
  "status": "active"
}
```

| Field          | Type   | Required | Constraints                          |
|----------------|--------|----------|--------------------------------------|
| `display_name` | string | No       | Free text (preserved if omitted)     |
| `role`         | enum   | No       | `admin`, `mechanic`, or `seller`     |
| `status`       | string | No       | `active` or `inactive`               |

### Response (200)

Returns the updated user object.

### Error Responses

| Status | Error            | Condition                                   |
|--------|------------------|---------------------------------------------|
| 403    | Forbidden        | Admin trying to demote themselves            |
| 404    | Not Found        | User not found                              |
| 422    | Validation error | Invalid role or status value                 |

### Business Rules

- An admin cannot change their own role to non-admin
- Status must be exactly `active` or `inactive`
- `email` cannot be changed after creation
- Audit log records old and new values

### cURL

```bash
curl -k -X PUT https://localhost:8443/api/users/7c9e6679-7425-40de-944b-e07fc1f90ae7 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "Pedro Mecanico Senior",
    "role": "mechanic"
  }'
```

---

## DELETE /api/users/:id

Soft-delete a user (sets `status` to `inactive`). An admin cannot delete themselves.

**Auth level**: Admin

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | User ID     |

### Response (200)

```json
{
  "success": true,
  "data": null
}
```

### Error Responses

| Status | Error      | Condition                         |
|--------|------------|------------------------------------|
| 403    | Forbidden  | Admin trying to delete themselves   |
| 404    | Not Found  | User not found                     |

### Business Rules

- Users are soft-deleted (`status` = `inactive`), not physically removed
- The deleted user can no longer log in (login filters by `status = 'active'`)
- An admin cannot delete their own account
- Audit log records the deletion

### cURL

```bash
curl -k -X DELETE https://localhost:8443/api/users/7c9e6679-7425-40de-944b-e07fc1f90ae7 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```
