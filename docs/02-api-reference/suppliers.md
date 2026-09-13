# Suppliers

Base URL: `https://localhost:8443/api/suppliers`

All endpoints require authentication.

---

## GET /api/suppliers

List active suppliers with pagination, ordered by name.

**Auth level**: Protected

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
        "name": "Distribuidora Honda Chile",
        "contact_person": "Pedro Gonzalez",
        "email": "ventas@honda-cl.cl",
        "phone": "+56223456789",
        "address": "Av. Industrial 5678, Santiago",
        "tax_id": "76.123.456-7",
        "payment_terms": "Net 30",
        "status": "active",
        "created_at": "2025-01-15T10:30:00Z",
        "updated_at": "2025-01-15T10:30:00Z"
      }
    ],
    "total": 12,
    "page": 1,
    "per_page": 20
  }
}
```

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/suppliers?page=1&per_page=20" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## POST /api/suppliers

Create a new supplier.

**Auth level**: Protected

### Request Body

```json
{
  "name": "Distribuidora Honda Chile",
  "contact_person": "Pedro Gonzalez",
  "email": "ventas@honda-cl.cl",
  "phone": "+56223456789",
  "address": "Av. Industrial 5678, Santiago",
  "tax_id": "76.123.456-7",
  "payment_terms": "Net 30"
}
```

| Field            | Type   | Required | Constraints                          |
|------------------|--------|----------|--------------------------------------|
| `name`           | string | Yes      | 1–200 characters, non-empty         |
| `contact_person` | string | No       | Free text                            |
| `email`          | string | No       | Valid format if non-empty            |
| `phone`          | string | No       | Max 20 characters                    |
| `address`        | string | No       | Free text                            |
| `tax_id`         | string | No       | Max 50 characters                    |
| `payment_terms`  | string | No       | Free text                            |

### Response (200)

Returns the created supplier with generated `id` and timestamps.

### Error Responses

| Status | Error            | Condition                       |
|--------|------------------|----------------------------------|
| 422    | Validation error | Invalid name, email, or fields   |

### cURL

```bash
curl -k -X POST https://localhost:8443/api/suppliers \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Distribuidora Honda Chile",
    "contact_person": "Pedro Gonzalez",
    "email": "ventas@honda-cl.cl"
  }'
```

---

## GET /api/suppliers/:id

Get a single supplier by ID.

**Auth level**: Protected

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | Supplier ID |

### Response (200)

Returns the supplier object.

### Error Responses

| Status | Error      | Condition                       |
|--------|------------|----------------------------------|
| 404    | Not Found  | Supplier not found or deleted    |

### cURL

```bash
curl -k -X GET https://localhost:8443/api/suppliers/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## PUT /api/suppliers/:id

Update a supplier.

**Auth level**: Protected

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | Supplier ID |

### Request Body

Same as `POST /api/suppliers` (all fields required).

### Response (200)

Returns the updated supplier. `created_at` is preserved; only `updated_at` changes.

### Error Responses

| Status | Error            | Condition                       |
|--------|------------------|----------------------------------|
| 404    | Not Found        | Supplier not found               |
| 422    | Validation error | Invalid field values             |

### Business Rules

- Audit log records both old and new values
- `status` field is preserved (cannot be changed via this endpoint)

### cURL

```bash
curl -k -X PUT https://localhost:8443/api/suppliers/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Distribuidora Honda Chile",
    "contact_person": "Pedro Gonzalez",
    "email": "ventas@honda-cl.cl",
    "phone": "+56223456789",
    "address": "Av. Industrial 5678, Santiago",
    "tax_id": "76.123.456-7",
    "payment_terms": "Net 30"
  }'
```
