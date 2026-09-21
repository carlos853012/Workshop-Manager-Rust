# Sales

Base URL: `https://localhost:8443/api/sales`

All endpoints require authentication.

---

## GET /api/sales

List completed sales with pagination.

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
        "subtotal": 30000,
        "discount_amount": 3000,
        "taxable_amount": 22689,
        "tax_amount": 4311,
        "total": 27000,
        "payment_method": "cash",
        "status": "completed",
        "created_at": "2025-01-15T14:30:00Z"
      }
    ],
    "total": 85,
    "page": 1,
    "per_page": 20
  }
}
```

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/sales?page=1&per_page=20" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## POST /api/sales

Create a new sale with line items. Stock is deducted atomically within a database transaction.

**Auth level**: Protected (Admin or Seller)

### Request Body

```json
{
  "customer_name": "Juan Perez",
  "customer_email": "juan@email.com",
  "customer_phone": "+56912345678",
  "payment_method": "cash",
  "discount_amount": 10,
  "items": [
    {
      "product_id": "550e8400-e29b-41d4-a716-446655440000",
      "quantity": 2,
      "unit_price": 15000,
      "discount": 5
    },
    {
      "product_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "quantity": 1,
      "unit_price": 25000,
      "discount": null
    }
  ]
}
```

| Field             | Type     | Required | Constraints                              |
|-------------------|----------|----------|------------------------------------------|
| `customer_name`   | string   | No       | Free text                                |
| `customer_email`  | string   | No       | Must contain `@` and `.` if provided     |
| `customer_phone`  | string   | No       | Max 20 characters                        |
| `payment_method`  | enum     | Yes      | `cash`, `card`, or `transfer`            |
| `discount_amount` | decimal  | No       `0..=100` (percentage discount on total)|
| `items`           | array    | Yes      | At least 1 item required                 |

#### Item Object

| Field        | Type    | Required | Constraints                     |
|--------------|---------|----------|---------------------------------|
| `product_id` | UUID    | Yes      | Must exist and be active        |
| `quantity`   | i32     | Yes      | `> 0`                           |
| `unit_price` | decimal | Yes      | `>= 0`                          |
| `discount`   | decimal | No       | `0..=100` (percentage per item) |

### Response (200)

```json
{
  "success": true,
  "data": {
    "id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
    "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "customer_name": "Juan Perez",
    "customer_email": "juan@email.com",
    "customer_phone": "+56912345678",
    "subtotal": 55000,
    "discount_amount": 5500,
    "taxable_amount": 41681,
    "tax_amount": 7919,
    "total": 49500,
    "payment_method": "cash",
    "status": "completed",
    "created_at": "2025-01-15T14:30:00Z",
    "items": [
      {
        "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "sale_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
        "product_id": "550e8400-e29b-41d4-a716-446655440000",
        "product_name": "Filtro de Aceite Honda CB500",
        "quantity": 2,
        "unit_price": 15000,
        "total": 28500
      },
      {
        "id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
        "sale_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
        "product_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "product_name": "Cadena 520H",
        "quantity": 1,
        "unit_price": 25000,
        "total": 25000
      }
    ]
  }
}
```

### Error Responses

| Status | Error            | Condition                                 |
|--------|------------------|-------------------------------------------|
| 403    | Forbidden        | User is not admin or seller               |
| 404    | Not Found        | Product ID does not exist                 |
| 409    | Conflict         | Insufficient stock for a product          |
| 422    | Validation error | Invalid fields (bad email, zero quantity, etc.) |

### Business Logic

1. All items are sorted by `product_id` to prevent deadlocks (consistent lock ordering)
2. Each product is locked with `SELECT ... FOR UPDATE` within the transaction
3. Stock is checked and deducted atomically
4. IVA (19% Chilean tax) is extracted from the total (prices include IVA)
5. All amounts are rounded to the nearest CLP multiple of 10
6. The sale status is always `completed` upon creation
7. An audit log entry is created

### IVA Calculation

Chilean prices include IVA. The system extracts the base amount and tax:

- `taxable_amount = round_to_ten(total / 1.19)`
- `tax_amount = total - taxable_amount`
- Rounding follows Chilean law (nearest multiple of 10)

### cURL

```bash
curl -k -X POST https://localhost:8443/api/sales \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  -H "Content-Type: application/json" \
  -d '{
    "customer_name": "Juan Perez",
    "payment_method": "cash",
    "items": [
      {
        "product_id": "550e8400-e29b-41d4-a716-446655440000",
        "quantity": 2,
        "unit_price": 15000
      }
    ]
  }'
```

---

## GET /api/sales/:id

Get a sale with its line items.

**Auth level**: Protected

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | Sale ID     |

### Response (200)

Returns the sale object with an `items` array (same structure as POST response).

### Error Responses

| Status | Error      | Condition                    |
|--------|------------|------------------------------|
| 404    | Not Found  | Sale not found               |

### cURL

```bash
curl -k -X GET https://localhost:8443/api/sales/7c9e6679-7425-40de-944b-e07fc1f90ae7 \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## Payment Methods

| Value      | Description                |
|------------|----------------------------|
| `cash`     | Cash payment               |
| `card`     | Credit/debit card          |
| `transfer` | Bank transfer              |

---

## Amount Fields

| Field             | Description                                      |
|-------------------|--------------------------------------------------|
| `subtotal`        | Sum of all item subtotals before discounts       |
| `discount_amount` | Total discount applied (subtotal × discount %)   |
| `taxable_amount`  | Base amount excluding IVA (subtotal - discount, rounded) |
| `tax_amount`      | IVA amount (19% of taxable_amount)               |
| `total`           | Final amount: subtotal - discount_amount         |

---

## POST /api/sales/:id/cancel

Cancel an existing sale and restore product stock.

**Auth level**: Protected (Admin or Seller only)

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id`      | UUID | The sale to cancel |

### Response (200)

Returns the sale with `status` changed to `"cancelled"`.

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "workshop_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "customer_name": "Juan Perez",
    "customer_email": "juan@email.com",
    "customer_phone": "+56912345678",
    "subtotal": 30000,
    "discount_amount": 3000,
    "taxable_amount": 22689,
    "tax_amount": 4311,
    "total": 27000,
    "payment_method": "cash",
    "status": "cancelled",
    "created_at": "2025-01-15T14:30:00Z"
  }
}
```

### Status Codes

| Status | Condition |
|--------|-----------|
| 200 | Sale successfully cancelled |
| 403 | User is not Admin or Seller |
| 404 | Sale not found or different workshop |
| 409 | Sale is already cancelled |

### Business Logic

1. Runs inside a PostgreSQL transaction with `FOR UPDATE` lock
2. Restores stock for each line item (`products.stock += quantity`)
3. Sets `sales.status = 'cancelled'`
4. Records an audit log entry (action: `"cancel"`)

### cURL

```bash
curl -k -X POST https://localhost:8443/api/sales/550e8400-e29b-41d4-a716-446655440000/cancel \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```
