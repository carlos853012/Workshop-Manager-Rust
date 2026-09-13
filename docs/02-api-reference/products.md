# Products

Base URL: `https://localhost:8443/api/products`

All endpoints require authentication unless noted.

---

## GET /api/products

List products with pagination. Only returns active products for the authenticated user's workshop.

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
        "name": "Filtro de Aceite Honda CB500",
        "description": "Filtro de aceite original Honda",
        "category": "Filtros",
        "brand": "Honda",
        "model": "CB500",
        "sku": "OIL-HONDA-CB500",
        "barcode": "7801234567890",
        "price": 15000,
        "cost": 8000,
        "stock": 25,
        "min_stock": 5,
        "location": "Estante A-3",
        "supplier_id": null,
        "status": "active",
        "created_at": "2025-01-15T10:30:00Z",
        "updated_at": "2025-01-15T10:30:00Z"
      }
    ],
    "total": 150,
    "page": 1,
    "per_page": 20
  }
}
```

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/products?page=1&per_page=20" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## POST /api/products

Create a new product. If `barcode` is omitted or empty, an EAN-13 barcode is auto-generated using the workshop's prefix.

**Auth level**: Protected (Admin or Seller)

### Request Body

```json
{
  "name": "Filtro de Aceite Honda CB500",
  "description": "Filtro de aceite original Honda",
  "category": "Filtros",
  "brand": "Honda",
  "model": "CB500",
  "sku": "OIL-HONDA-CB500",
  "barcode": null,
  "price": 15000,
  "cost": 8000,
  "stock": 25,
  "min_stock": 5,
  "location": "Estante A-3",
  "supplier_id": null
}
```

| Field          | Type     | Required | Constraints                          |
|----------------|----------|----------|--------------------------------------|
| `name`         | string   | Yes      | 1–200 characters, non-empty         |
| `description`  | string   | No       | Free text                            |
| `category`     | string   | No       | Free text                            |
| `brand`        | string   | No       | Free text                            |
| `model`        | string   | No       | Free text                            |
| `sku`          | string   | No       | Max 100 characters                   |
| `barcode`      | string   | No       | EAN-13; auto-generated if omitted   |
| `price`        | decimal  | Yes      | `>= 0` (CLP)                         |
| `cost`         | decimal  | Yes      | `>= 0` (CLP)                         |
| `stock`        | i32      | Yes      | `>= 0`                               |
| `min_stock`    | i32      | Yes      | `>= 0`                               |
| `location`     | string   | No       | Free text                            |
| `supplier_id`  | UUID     | No       | Must exist in suppliers if provided  |

### Response (200)

Returns the created product with generated `id`, `barcode` (if auto-generated), and timestamps.

### Error Responses

| Status | Error            | Condition                       |
|--------|------------------|----------------------------------|
| 403    | Forbidden        | User is not admin or seller      |
| 422    | Validation error | Invalid field values             |

### Business Rules

- Barcode is auto-generated as EAN-13 if not provided
- All monetary values are in CLP (Chilean Pesos)
- An audit log entry is created for every product creation

### cURL

```bash
curl -k -X POST https://localhost:8443/api/products \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Filtro de Aceite Honda CB500",
    "category": "Filtros",
    "brand": "Honda",
    "price": 15000,
    "cost": 8000,
    "stock": 25,
    "min_stock": 5
  }'
```

---

## GET /api/products/:id

Get a single product by ID.

**Auth level**: Protected

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | Product ID  |

### Response (200)

Returns the product object.

### Error Responses

| Status | Error      | Condition                         |
|--------|------------|------------------------------------|
| 404    | Not Found  | Product not found or deleted       |

### cURL

```bash
curl -k -X GET https://localhost:8443/api/products/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## PUT /api/products/:id

Update a product. The barcode is preserved from the original record (cannot be changed via update).

**Auth level**: Protected (Admin or Seller)

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | Product ID  |

### Request Body

Same as `POST /api/products` (all fields required).

### Response (200)

Returns the updated product. The `barcode` field reflects the original value.

### Error Responses

| Status | Error            | Condition                       |
|--------|------------------|----------------------------------|
| 403    | Forbidden        | User is not admin or seller      |
| 404    | Not Found        | Product not found                |
| 422    | Validation error | Invalid field values             |

### Business Rules

- Barcode cannot be changed after creation
- Audit log records both old and new values
- `created_at` is preserved; only `updated_at` changes

### cURL

```bash
curl -k -X PUT https://localhost:8443/api/products/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Filtro de Aceite Honda CB500",
    "price": 18000,
    "cost": 9000,
    "stock": 20,
    "min_stock": 5
  }'
```

---

## DELETE /api/products/:id

Soft-delete a product (sets `status` to `deleted`).

**Auth level**: Protected (Admin only)

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | Product ID  |

### Response (200)

```json
{
  "success": true,
  "data": null
}
```

### Error Responses

| Status | Error      | Condition                        |
|--------|------------|----------------------------------|
| 403    | Forbidden  | User is not admin                |
| 404    | Not Found  | Product not found or already deleted |

### Business Rules

- Products are soft-deleted (`status` = `deleted`), not physically removed
- Deleted products are excluded from all list and lookup queries
- Audit log records the deletion

### cURL

```bash
curl -k -X DELETE https://localhost:8443/api/products/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## GET /api/products/lookup?barcode=

Look up a product by barcode for POS (Point of Sale) use. Returns a simplified product response.

**Auth level**: Protected

### Query Parameters

| Parameter | Type   | Required | Description |
|-----------|--------|----------|-------------|
| `barcode` | string | Yes      | Barcode to look up (trimmed) |

### Response (200)

```json
{
  "success": true,
  "data": {
    "product_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Filtro de Aceite Honda CB500",
    "price": 15000,
    "cost": 8000,
    "stock": 25,
    "min_stock": 5,
    "barcode": "7801234567890",
    "sku": "OIL-HONDA-CB500",
    "supplier_id": null
  }
}
```

### Error Responses

| Status | Error           | Condition              |
|--------|-----------------|------------------------|
| 404    | Not Found       | No product with that barcode |
| 422    | Validation error| Empty barcode parameter |

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/products/lookup?barcode=7801234567890" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```
