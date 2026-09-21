# Reports & Analytics

Base URL: `https://localhost:8443/api`

All endpoints require authentication.

---

## GET /api/analytics/dashboard

Get dashboard KPIs (Key Performance Indicators).

**Auth level**: Protected

### Response (200)

```json
{
  "success": true,
  "data": {
    "total_products": 150,
    "low_stock": 12,
    "total_sales": 85,
    "total_revenue": 4250000,
    "pending_repairs": 8,
    "completed_repairs": 45,
    "average_sale": 50000
  }
}
```

| Field              | Type    | Description                                    |
|--------------------|---------|------------------------------------------------|
| `total_products`   | i64     | Count of active products                       |
| `low_stock`        | i64     | Products where `stock <= min_stock`            |
| `total_sales`      | i64     | Count of completed sales                       |
| `total_revenue`    | decimal | Sum of all completed sale totals (CLP)         |
| `pending_repairs`  | i64     | Repairs with status `pending`                  |
| `completed_repairs`| i64     | Repairs with status `completed`                |
| `average_sale`     | decimal | `total_revenue / total_sales` (or 0 if none)  |

### cURL

```bash
curl -k -X GET https://localhost:8443/api/analytics/dashboard \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## GET /api/analytics/kpis

Get extended KPIs (suppliers, customers, in-progress/cancelled repairs).

**Auth level**: Protected

### Response (200)

```json
{
  "success": true,
  "data": {
    "total_suppliers": 12,
    "total_customers": 67,
    "in_progress_repairs": 5,
    "cancelled_repairs": 3
  }
}
```

| Field                 | Type | Description                                        |
|-----------------------|------|----------------------------------------------------|
| `total_suppliers`     | i64  | Count of active suppliers                          |
| `total_customers`     | i64  | Distinct customers (by email) in completed sales   |
| `in_progress_repairs` | i64  | Repairs with status `in_progress`                  |
| `cancelled_repairs`   | i64  | Repairs with status `cancelled`                    |

### cURL

```bash
curl -k -X GET https://localhost:8443/api/analytics/kpis \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## GET /api/reports/clients

Get aggregated client report. Groups sales by customer email.

**Auth level**: Protected

### Response (200)

```json
{
  "success": true,
  "data": [
    {
      "customer_email": "juan@email.com",
      "customer_name": "Juan Perez",
      "total_purchases": 5,
      "total_spent": 275000,
      "last_purchase": "2025-01-15T14:30:00Z"
    },
    {
      "customer_email": "maria@email.com",
      "customer_name": "Maria Lopez",
      "total_purchases": 3,
      "total_spent": 150000,
      "last_purchase": "2025-01-14T10:00:00Z"
    }
  ]
}
```

| Field             | Type     | Description                              |
|-------------------|----------|------------------------------------------|
| `customer_email`  | string   | Customer email (grouping key)            |
| `customer_name`   | string   | Most recent name used                    |
| `total_purchases` | i64      | Number of completed sales                |
| `total_spent`     | decimal  | Sum of sale totals                       |
| `last_purchase`   | datetime | Most recent sale timestamp               |

Results are ordered by `total_spent` descending.

### cURL

```bash
curl -k -X GET https://localhost:8443/api/reports/clients \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## GET /api/reports/client-history?email=

Get complete history for a specific client (sales + repairs).

**Auth level**: Protected

### Query Parameters

| Parameter | Type   | Required | Description  |
|-----------|--------|----------|--------------|
| `email`   | string | Yes      | Client email |

### Response (200)

```json
{
  "success": true,
  "data": {
    "email": "juan@email.com",
    "name": "Juan Perez",
    "sales": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "total": 55000,
        "payment_method": "cash",
        "created_at": "2025-01-15T14:30:00Z"
      }
    ],
    "repairs": [
      {
        "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "description": "Cambio de aceite",
        "status": "completed",
        "total": 45000,
        "created_at": "2025-01-10T09:00:00Z"
      }
    ],
    "total_spent": 55000,
    "total_repairs": 1
  }
}
```

| Field          | Type     | Description                              |
|----------------|----------|------------------------------------------|
| `email`        | string   | Client email                             |
| `name`         | string   | Most recent customer name from sales     |
| `sales`        | array    | All completed sales for this email       |
| `repairs`      | array    | All repairs for this email               |
| `total_spent`  | decimal  | Sum of all sale totals                   |
| `total_repairs`| i64      | Count of repairs                         |

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/reports/client-history?email=juan@email.com" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## GET /api/reports/client-search?query=

Search clients by name, email, or license plate. Searches across repairs.

**Auth level**: Protected

### Query Parameters

| Parameter | Type   | Required | Description                     |
|-----------|--------|----------|---------------------------------|
| `query`   | string | Yes      | Search term (ILIKE match)       |

### Response (200)

```json
{
  "success": true,
  "data": [
    {
      "customer_name": "Juan Perez",
      "customer_email": "juan@email.com",
      "customer_phone": "+56912345678",
      "vehicles": [
        {
          "license_plate": "BJH61",
          "vehicle": "Honda CB500X 2023",
          "total_repairs": 3,
          "last_repair_date": "2025-01-15T10:00:00Z"
        }
      ]
    }
  ]
}
```

| Field             | Type   | Description                              |
|-------------------|--------|------------------------------------------|
| `customer_name`   | string | Customer name                            |
| `customer_email`  | string | Customer email                           |
| `customer_phone`  | string | Customer phone                           |
| `vehicles`        | array  | Vehicles associated with this customer   |

### Search Behavior

- Matches against `license_plate`, `customer_name`, and `customer_email` in repairs
- Case-insensitive (ILIKE)
- Excludes deleted repairs

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/reports/client-search?query=juan" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## GET /api/reports/client-certificate

Generate a service certificate (JSON). Requires at least one of `email`, `name`, or `plate`.

**Auth level**: Protected

### Query Parameters

| Parameter | Type   | Required | Description              |
|-----------|--------|----------|--------------------------|
| `email`   | string | No       | Client email             |
| `name`    | string | No       | Client name              |
| `plate`   | string | No       | License plate            |

### Response (200)

```json
{
  "success": true,
  "data": {
    "workshop": {
      "name": "Taller Moto Chile",
      "address": "Av. Libertador 1234",
      "city": "Santiago"
    },
    "client": {
      "name": "Juan Perez",
      "email": "juan@email.com",
      "phone": "+56912345678"
    },
    "vehicle": {
      "description": "Honda CB500X 2023",
      "license_plate": "BJH61"
    },
    "services": [
      {
        "date": "2025-01-15T10:00:00Z",
        "description": "Cambio de aceite y filtro",
        "diagnosis": "Aceite degradado",
        "status": "completed"
      }
    ],
    "parts_used": [
      {
        "name": "Filtro de Aceite Honda",
        "quantity": 1,
        "product_brand": "Honda",
        "product_model": "CB500",
        "product_sku": "OIL-HONDA-CB500"
      }
    ],
    "generated_at": "2025-01-15T15:00:00Z"
  }
}
```

### Query Priority

1. `email` + `plate` — matches both
2. `email` only — all repairs for that email
3. `name` + `plate` — matches both
4. `name` only — all repairs for that name
5. `plate` only — all repairs with that plate
6. None provided — returns empty (error)

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/reports/client-certificate?email=juan@email.com&plate=BJH61" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## GET /api/reports/client-certificate.pdf

Download a service certificate as a PDF file. Same query parameters as the JSON endpoint.

**Auth level**: Protected

### Query Parameters

Same as `/api/reports/client-certificate`.

### Response

- **Content-Type**: `application/pdf`
- **Content-Disposition**: `attachment; filename="certificado_BJH61.pdf"`

The filename uses the license plate, or `servicio` if no plate is available.

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/reports/client-certificate.pdf?email=juan@email.com&plate=BJH61" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..." \
  --output certificado.pdf
```

---

## GET /api/analytics/revenue

Get revenue time-series data (sales + repairs) grouped by day, week, or month.

**Auth level**: Protected

### Query Parameters

| Parameter    | Type   | Default       | Constraints        |
|--------------|--------|---------------|--------------------|
| `start_date` | string | 180 days ago  | Format: YYYY-MM-DD |
| `end_date`   | string | Today         | Format: YYYY-MM-DD |

### Response (200)

```json
{
  "success": true,
  "data": {
    "data": [
      {
        "period": "2025-01-15",
        "sales": 250000,
        "repairs": 75000
      },
      {
        "period": "2025-01-16",
        "sales": 180000,
        "repairs": 0
      }
    ],
    "grouping": "day"
  }
}
```

| Field            | Type   | Description |
|------------------|--------|-------------|
| `data`           | array  | Time-series data points |
| `data[].period`  | string | Period label (format depends on grouping) |
| `data[].sales`   | decimal | Total sales revenue for that period |
| `data[].repairs` | decimal | Total repair labor cost for that period |
| `grouping`       | string | `"day"`, `"week"`, or `"month"` |

Grouping is auto-selected based on date range:
- `day`: range ≤ 31 days
- `week`: range 32–365 days
- `month`: range > 365 days

### cURL

```bash
curl -k -X GET "https://localhost:8443/api/analytics/revenue?start_date=2025-01-01&end_date=2025-01-31" \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```

---

## GET /api/analytics/top-products

Get the top 5 products by quantity sold.

**Auth level**: Protected

### Response (200)

```json
{
  "success": true,
  "data": {
    "data": [
      {
        "product_id": "550e8400-e29b-41d4-a716-446655440000",
        "product_name": "Filtro de Aceite Honda CB500",
        "total_quantity": 142,
        "total_revenue": 2130000
      },
      {
        "product_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "product_name": "Cadena 520H",
        "total_quantity": 87,
        "total_revenue": 2175000
      }
    ]
  }
}
```

| Field               | Type    | Description |
|---------------------|---------|-------------|
| `data`              | array   | Top 5 products by quantity sold |
| `data[].product_id` | UUID    | Product identifier |
| `data[].product_name` | string | Product name |
| `data[].total_quantity` | i64  | Total units sold (completed sales only) |
| `data[].total_revenue` | decimal | Total revenue (quantity × unit_price) |

### cURL

```bash
curl -k -X GET https://localhost:8443/api/analytics/top-products \
  -H "Authorization: Bearer eyJ..." \
  -H "X-WorkshopManager-Key: your-api-key" \
  -H "X-WorkshopManager-Device-Key: wm_..."
```
