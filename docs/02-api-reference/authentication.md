# Authentication

Base URL: `https://localhost:8443`

---

## POST /api/auth/login

Authenticate a user and obtain a JWT token.

**Auth level**: Public (no headers required)

### Request Body

```json
{
  "email": "admin@workshop.cl",
  "password": "mypassword123"
}
```

| Field      | Type   | Required | Constraints                          |
|------------|--------|----------|--------------------------------------|
| `email`    | string | Yes      | Valid email format, max 200 chars    |
| `password` | string | Yes      | Min 8 characters                     |

### Response (200)

```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiJ9...",
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "email": "admin@workshop.cl",
      "display_name": "Carlos Admin",
      "role": "admin",
      "status": "active",
      "created_at": "2025-01-15T10:30:00Z"
    },
    "workshop": {
      "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "name": "Taller Moto Chile",
      "address": "Av. Libertador 1234",
      "city": "Santiago",
      "barcode_prefix": null,
      "created_at": "2025-01-15T10:30:00Z",
      "updated_at": "2025-01-15T10:30:00Z"
    }
  }
}
```

**Note**: `password_hash` is always stripped from the response (empty string).

### Error Responses

| Status | Error                | Condition                           |
|--------|----------------------|-------------------------------------|
| 401    | Unauthorized         | Invalid email or password           |
| 422    | Validation error     | Invalid email format                |
| 429    | Too many requests    | >5 attempts in 300s window          |

### cURL

```bash
curl -k -X POST https://localhost:8443/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@workshop.cl","password":"mypassword123"}'
```

---

## POST /api/auth/register

Register the initial admin user and workshop. Only allowed when the database has **zero users**.

**Auth level**: Public (no headers required)

### Request Body

```json
{
  "workshop_name": "Taller Moto Chile",
  "workshop_address": "Av. Libertador 1234",
  "workshop_city": "Santiago",
  "admin_name": "Carlos Admin",
  "email": "admin@workshop.cl",
  "password": "mypassword123"
}
```

| Field              | Type   | Required | Constraints                     |
|--------------------|--------|----------|---------------------------------|
| `workshop_name`    | string | Yes      | 1–200 characters                |
| `workshop_address` | string | Yes      | 1–300 characters                |
| `workshop_city`    | string | Yes      | 1–120 characters                |
| `admin_name`       | string | Yes      | 1–200 characters                |
| `email`            | string | Yes      | Valid email, max 200 chars      |
| `password`         | string | Yes      | Min 8 characters                |

### Response (200)

```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiJ9...",
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "email": "admin@workshop.cl",
      "display_name": "Carlos Admin",
      "role": "admin",
      "status": "active",
      "created_at": "2025-01-15T10:30:00Z"
    },
    "workshop": {
      "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "name": "Taller Moto Chile",
      "address": "Av. Libertador 1234",
      "city": "Santiago",
      "barcode_prefix": null,
      "created_at": "2025-01-15T10:30:00Z",
      "updated_at": "2025-01-15T10:30:00Z"
    }
  }
}
```

### Business Logic

1. Checks that `users` table is empty (count == 0)
2. Creates the workshop and admin user in a single transaction
3. The admin user is automatically assigned the `admin` role
4. Returns a JWT token immediately (no separate login needed)

### Error Responses

| Status | Error                | Condition                                |
|--------|----------------------|------------------------------------------|
| 403    | Forbidden            | Users already exist (registration closed)|
| 409    | Conflict             | Email already registered                 |
| 422    | Validation error     | Invalid fields                           |

### cURL

```bash
curl -k -X POST https://localhost:8443/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "workshop_name": "Taller Moto Chile",
    "workshop_address": "Av. Libertador 1234",
    "workshop_city": "Santiago",
    "admin_name": "Carlos Admin",
    "email": "admin@workshop.cl",
    "password": "mypassword123"
  }'
```

---

## GET /api/auth/status

Get the current authenticated user's profile.

**Auth level**: Protected (JWT + API key + device key)

### Response (200)

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "email": "admin@workshop.cl",
    "display_name": "Carlos Admin",
    "role": "admin",
    "status": "active",
    "created_at": "2025-01-15T10:30:00Z"
  }
}
```

### Error Responses

| Status | Error       | Condition                          |
|--------|-------------|-------------------------------------|
| 401    | Unauthorized| Invalid or expired JWT              |

### cURL

```bash
curl -k -X GET https://localhost:8443/api/auth/status \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## JWT Token Structure

Tokens are HS256-signed JSON Web Tokens with the following claims:

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "email": "admin@workshop.cl",
  "role": "admin",
  "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "exp": 1737043800
}
```

| Claim          | Type   | Description                              |
|----------------|--------|------------------------------------------|
| `sub`          | string | User ID (UUID)                           |
| `email`        | string | User email                               |
| `role`         | string | One of: `admin`, `mechanic`, `seller`    |
| `workshop_id`  | string | Workshop ID (UUID)                       |
| `exp`          | number | Expiration timestamp (UTC)               |

### Token Lifecycle

| Property       | Value                            |
|----------------|----------------------------------|
| Algorithm      | HS256                            |
| Expiration     | 24 hours from creation           |
| Refresh        | Not supported (re-login required)|
| Revocation     | Not supported (stateless)        |

### Password Hashing

- Algorithm: Argon2id
- Memory: 64 MB
- Iterations: 3
- Parallelism: 4

---

## GET /api/auth/setup-status

Check if the system has been set up (at least one user exists).

**Auth level**: Public (no authentication required)

### Response (200)

```json
{
  "success": true,
  "data": {
    "has_users": true
  }
}
```

| Field       | Type | Description |
|-------------|------|-------------|
| `has_users` | bool | `true` if at least one user exists |

### cURL

```bash
curl -k https://localhost:8443/api/auth/setup-status
```

---

## GET /api/auth/license

Get the current license information.

**Auth level**: Protected (any authenticated user)

### Response (200)

```json
{
  "success": true,
  "data": {
    "is_trial": true,
    "tier": "Trial"
  }
}
```

| Field      | Type   | Description |
|------------|--------|-------------|
| `is_trial` | bool   | `true` if the license is a trial |
| `tier`     | string | License tier: `"Trial"`, `"Base"`, `"Reports"`, `"Advanced"`, `"Api"` |

### cURL

```bash
curl -k https://localhost:8443/api/auth/license \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```
