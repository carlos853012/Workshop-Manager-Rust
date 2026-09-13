# Dashboard

The Dashboard provides a real-time overview of your workshop's performance.

## Accessing the Dashboard

Click **Dashboard** in the sidebar, or it appears automatically after login.

<!-- Screenshot: Full dashboard view -->

## KPI Cards

The Dashboard displays 8 Key Performance Indicator cards in a grid:

### Product Metrics

| Card | Description |
|------|-------------|
| **Total Products** | Total number of active products in inventory (excluding soft-deleted) |
| **Low Stock** | Products where current stock is at or below the minimum stock threshold |

### Sales Metrics

| Card | Description |
|------|-------------|
| **Total Sales** | Number of completed sales transactions |
| **Revenue** | Total revenue in Chilean Pesos (CLP), formatted as `$X.XXX` |
| **Average Sale** | Average sale amount across all completed sales |

### Repair Metrics

| Card | Description |
|------|-------------|
| **Pending Repairs** | Repairs with status `Pending` - awaiting assignment or work |
| **Completed Repairs** | Repairs with status `Completed` - finished and delivered |
| **In-Progress Repairs** | Repairs currently being worked on |

### Business Metrics

| Card | Description |
|------|-------------|
| **Total Suppliers** | Number of registered suppliers |
| **Total Customers** | Number of unique customers (based on email) |

## Revenue Metrics

Revenue is displayed in Chilean Pesos (CLP) using the `format_clp` function:

- No decimal places (CLP is integer currency)
- Thousands separator with dots: `$1.500.000`
- Prefix `$`: `$150.000`

The revenue figure represents the sum of all completed sale totals.

## Sales Statistics

| Metric | Source |
|--------|--------|
| Total Sales | Count of all completed sales |
| Revenue | Sum of `total` field from all sales |
| Average Sale | Revenue / Total Sales |

All amounts exclude cancelled or refunded sales.

## Inventory Alerts

The **Low Stock** card highlights products that need restocking. A product appears in this count when:

```
current_stock <= min_stock
```

Set `min_stock` when creating or editing a product to enable low stock alerts.

## Repair Status Overview

Repairs are categorized by status:

| Status | Description |
|--------|-------------|
| Pending | Awaiting assignment or work to begin |
| In Progress | Currently being worked on |
| Completed | Work finished, ready for pickup |
| Cancelled | Repair was cancelled |

The Dashboard shows counts for Pending, Completed, and In-Progress repairs.

## Refreshing Data

The Dashboard data loads automatically when:

1. You first navigate to the page
2. You return from another section

To manually refresh, navigate away from the Dashboard and return, or log out and log back in.

<!-- Screenshot: Dashboard with highlighted KPI cards -->
