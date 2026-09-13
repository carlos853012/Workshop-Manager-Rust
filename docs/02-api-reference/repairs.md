# Repairs

Base URL: `https://localhost:8443/api/repairs`

All endpoints require authentication.

---

## GET /api/repairs

List repairs with pagination. Excludes soft-deleted repairs.

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
        "customer_name": "Juan Perez",
        "customer_email": "juan@email.com",
        "customer_phone": "+56912345678",
        "vehicle": "Honda CB500X 2023",
        "license_plate": "BJH61",
        "description": "Cambio de aceite y filtro",
        "diagnosis": "Aceite degradado, filtro obstruido",
        "technician_id": null,
        "estimated_delivery": "2025-01-20",
        "priority": "medium",
        "status": "pending",
        "estimated_cost": 45000,
        "final_cost": null,
        "labor_cost": null,
        "created_at": "2025-01-15T10:30:00Z",
        "updated_at": "2025-01-15T10:30:00Z"
      }
    ],
    "total": 42,
    "page": 1,
    "per_page": 20
  }
}
```

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/repairs?page=1&per_page=20" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## POST /api/repairs

Create a new repair. Automatically creates an initial `repair_update` record.

**Auth level**: Protected

### Request Body

```json
{
  "customer_name": "Juan Perez",
  "customer_email": "juan@email.com",
  "customer_phone": "+56912345678",
  "vehicle": "Honda CB500X 2023",
  "license_plate": "BJH61",
  "description": "Cambio de aceite y filtro",
  "priority": "medium",
  "estimated_cost": 45000,
  "estimated_delivery": "2025-01-20"
}
```

| Field                | Type     | Required | Constraints                              |
|----------------------|----------|----------|------------------------------------------|
| `customer_name`      | string   | No       | Free text                                |
| `customer_email`     | string   | No       | Must contain `@` and `.` if provided     |
| `customer_phone`     | string   | No       | Max 20 characters                        |
| `vehicle`            | string   | No       | Free text                                |
| `license_plate`      | string   | No       | Max 50 chars; validated if provided      |
| `description`        | string   | No       | Free text                                |
| `priority`           | enum     | Yes      | `high`, `medium`, or `low`               |
| `estimated_cost`     | decimal  | No       | `>= 0`                                   |
| `estimated_delivery` | date     | No       | ISO 8601 date (YYYY-MM-DD)              |

### Response (200)

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "customer_name": "Juan Perez",
    "customer_email": "juan@email.com",
    "customer_phone": "+56912345678",
    "vehicle": "Honda CB500X 2023",
    "license_plate": "BJH61",
    "description": "Cambio de aceite y filtro",
    "diagnosis": null,
    "technician_id": null,
    "estimated_delivery": "2025-01-20",
    "priority": "medium",
    "status": "pending",
    "estimated_cost": 45000,
    "final_cost": null,
    "labor_cost": null,
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T10:30:00Z",
    "updates": [
      {
        "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "repair_id": "550e8400-e29b-41d4-a716-446655440000",
        "status": "pending",
        "description": "Repair created",
        "created_by": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
        "created_at": "2025-01-15T10:30:00Z"
      }
    ]
  }
}
```

### Error Responses

| Status | Error            | Condition                              |
|--------|------------------|----------------------------------------|
| 422    | Validation error | Invalid email, license plate, or fields|

### License Plate Validation

Chilean license plate formats are validated:

| Format        | Example   | Description                  |
|---------------|-----------|------------------------------|
| `BBBB NN`     | `BJH61`   | Motorcycle (3 letters + 2 digits) |
| `BB BB NN`    | `BCDF12`  | New format (4 letters + 2 digits) |
| `BB NNNN`     | `AR1240`  | Old format (2 letters + 4 digits) |

### Business Rules

- Initial status is always `pending`
- An initial `repair_update` is created automatically
- Audit log entry is created

### cURL

```bash
curl -k -X POST https://localhost:8443/api/repairs \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  -H "Content-Type: application/json" \
  -d '{
    "customer_name": "Juan Perez",
    "vehicle": "Honda CB500X 2023",
    "license_plate": "BJH61",
    "description": "Cambio de aceite y filtro",
    "priority": "medium"
  }'
```

---

## GET /api/repairs/:id

Get a repair with its full update history.

**Auth level**: Protected

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | Repair ID   |

### Response (200)

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "customer_name": "Juan Perez",
    "customer_email": "juan@email.com",
    "customer_phone": "+56912345678",
    "vehicle": "Honda CB500X 2023",
    "license_plate": "BJH61",
    "description": "Cambio de aceite y filtro",
    "diagnosis": "Aceite degradado",
    "technician_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
    "estimated_delivery": "2025-01-20",
    "priority": "medium",
    "status": "in_progress",
    "estimated_cost": 45000,
    "final_cost": null,
    "labor_cost": 15000,
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T14:00:00Z",
    "updates": [
      {
        "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "repair_id": "550e8400-e29b-41d4-a716-446655440000",
        "status": "pending",
        "description": "Repair created",
        "created_by": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
        "created_at": "2025-01-15T10:30:00Z"
      },
      {
        "id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
        "repair_id": "550e8400-e29b-41d4-a716-446655440000",
        "status": "in_progress",
        "description": "Status changed from pending to in_progress",
        "created_by": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
        "created_at": "2025-01-15T14:00:00Z"
      }
    ]
  }
}
```

### Error Responses

| Status | Error      | Condition                    |
|--------|------------|------------------------------|
| 404    | Not Found  | Repair not found or deleted  |

### cURL

```bash
curl -k -X GET https://localhost:8443/api/repairs/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## PUT /api/repairs/:id

Update a repair. Changing the status automatically creates a `repair_update` record.

**Auth level**: Protected

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | Repair ID   |

### Request Body

```json
{
  "status": "in_progress",
  "diagnosis": "Aceite degradado, filtro obstruido",
  "technician_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "estimated_cost": 50000,
  "final_cost": null,
  "labor_cost": 15000,
  "estimated_delivery": "2025-01-22"
}
```

| Field                | Type     | Required | Constraints                              |
|----------------------|----------|----------|------------------------------------------|
| `status`             | enum     | No       | `pending`, `in_progress`, `completed`, `cancelled` |
| `diagnosis`          | string   | No       | Free text                                |
| `technician_id`      | UUID     | No       | Must be a valid user ID                  |
| `estimated_cost`     | decimal  | No       | `>= 0`                                   |
| `final_cost`         | decimal  | No       | `>= 0`                                   |
| `labor_cost`         | decimal  | No       | `>= 0`                                   |
| `estimated_delivery` | date     | No       | ISO 8601 date                            |

### Response (200)

Returns the updated repair with its full update history.

### Error Responses

| Status | Error      | Condition                    |
|--------|------------|------------------------------|
| 404    | Not Found  | Repair not found or deleted  |

### Business Rules

- Only changed fields are updated (null fields preserve existing values)
- Status changes automatically create a `repair_update` entry with description: `"Status changed from X to Y"`
- Non-status field changes do not create update entries
- Audit log records both old and new values

### Repair Statuses

| Value          | Description                    |
|----------------|--------------------------------|
| `pending`      | Awaiting diagnosis/work        |
| `in_progress`  | Currently being worked on      |
| `completed`    | Work finished                  |
| `cancelled`    | Repair cancelled               |

### Priority Levels

| Value    | Description |
|----------|-------------|
| `high`   | Urgent      |
| `medium` | Normal      |
| `low`    | Low priority|

### cURL

```bash
curl -k -X PUT https://localhost:8443/api/repairs/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  -H "Content-Type: application/json" \
  -d '{
    "status": "in_progress",
    "diagnosis": "Aceite degradado",
    "technician_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7"
  }'
```

---

## GET /api/repairs/:id/parts

List parts used in a repair.

**Auth level**: Protected

### Path Parameters

| Parameter  | Type | Description |
|------------|------|-------------|
| `id`       | UUID | Repair ID   |

### Response (200)

```json
{
  "success": true,
  "data": [
    {
      "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "repair_id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Filtro de Aceite Honda",
      "quantity": 1,
      "unit_cost": 8000,
      "total_cost": 8000,
      "product_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "created_at": "2025-01-15T11:00:00Z"
    }
  ]
}
```

### cURL

```bash
curl -k -X GET https://localhost:8443/api/repairs/550e8400-e29b-41d4-a716-446655440000/parts \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## POST /api/repairs/:id/parts

Add a part to a repair. If `product_id` is provided, stock is deducted from that product.

**Auth level**: Protected

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | Repair ID   |

### Request Body

```json
{
  "name": "Filtro de Aceite Honda",
  "quantity": 1,
  "unit_cost": 8000,
  "product_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
}
```

| Field        | Type     | Required | Constraints                          |
|--------------|----------|----------|--------------------------------------|
| `name`       | string   | Yes      | Non-empty                            |
| `quantity`   | decimal  | Yes      | `> 0`                                |
| `unit_cost`  | decimal  | No       | `>= 0`                               |
| `product_id` | UUID     | No       | Must exist with sufficient stock     |

### Response (200)

```json
{
  "success": true,
  "data": {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "repair_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Filtro de Aceite Honda",
    "quantity": 1,
    "unit_cost": 8000,
    "total_cost": 8000,
    "product_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "created_at": "2025-01-15T11:00:00Z"
  }
}
```

### Error Responses

| Status | Error            | Condition                       |
|--------|------------------|----------------------------------|
| 404    | Not Found        | Repair not found                 |
| 422    | Validation error | Insufficient stock or bad input  |

### Business Rules

- `total_cost` is computed as `unit_cost × quantity`
- If `product_id` is provided, stock is deducted atomically
- If stock is insufficient, the operation fails with a validation error

### cURL

```bash
curl -k -X POST https://localhost:8443/api/repairs/550e8400-e29b-41d4-a716-446655440000/parts \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Filtro de Aceite Honda",
    "quantity": 1,
    "unit_cost": 8000,
    "product_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
  }'
```

---

## DELETE /api/repairs/:id/parts/:part_id

Remove a part from a repair. If the part was linked to a product, stock is restored.

**Auth level**: Protected

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | Repair ID   |
| `part_id` | UUID | Part ID     |

### Response (200)

```json
{
  "success": true,
  "data": null
}
```

### Error Responses

| Status | Error      | Condition                    |
|--------|------------|------------------------------|
| 404    | Not Found  | Repair or part not found     |

### Business Rules

- The part is physically deleted from `repair_parts`
- If the part had a `product_id`, the product's stock is restored by the part's `quantity`
- Audit log entry is created

### cURL

```bash
curl -k -X DELETE https://localhost:8443/api/repairs/550e8400-e29b-41d4-a716-446655440000/parts/a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```
