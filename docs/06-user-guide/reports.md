# Reports & Analytics

View client spending reports and business analytics.

## Accessing Reports

Navigate to **Reports** in the sidebar.

<!-- Screenshot: Reports page with client list -->

## Client Report

The Reports page shows a list of all clients who have made purchases or repairs.

### Client List

| Column | Description |
|--------|-------------|
| Email | Client's email address (unique identifier) |
| Name | Client's display name |
| Total Purchases | Number of sales transactions |
| Total Spent | Total amount spent (CLP) |
| Last Purchase | Date of most recent purchase |

### Sorting

Clients are listed by total spending (highest first) by default.

## Client History (Detailed View)

1. Click on a client row in the Reports list
2. A detailed view opens showing the client's full history

<!-- Screenshot: Client history detail view -->

### History Overview

The top section shows summary cards:

| Card | Description |
|------|-------------|
| Total Spent | Total amount across all transactions |
| Total Sales | Number of sales transactions |
| Total Repairs | Number of repair orders |
| Last Activity | Date of most recent transaction |

### Sales History

A list of all sales made by this client:

| Field | Description |
|-------|-------------|
| Sale ID | Unique transaction identifier |
| Total | Sale amount (CLP) |
| Payment Method | Cash, Card, or Transfer |
| Date | Date and time of sale |

### Repair History

A list of all repairs for this client:

| Field | Description |
|-------|-------------|
| Repair ID | Unique repair identifier |
| Description | Repair description |
| Status | Current repair status |
| Total Cost | Final repair cost (if completed) |
| Date | Date the repair was created |

## Client Search

Use the search bar to filter clients by:

- Name
- Email

Search is case-insensitive and matches partial strings.

## Dashboard Analytics

The Dashboard (see [Dashboard](dashboard.md)) provides real-time analytics:

| Metric | Description |
|--------|-------------|
| Total Revenue | Sum of all completed sales |
| Average Sale | Mean sale amount |
| Total Products | Active product count |
| Low Stock | Products needing restock |
| Pending Repairs | Awaiting work |
| Completed Repairs | Finished repairs |
| Total Suppliers | Registered suppliers |
| Total Customers | Unique clients |

## KPIs Explanation

### Revenue KPIs

- **Revenue**: Total CLP revenue from all completed sales
- **Average Sale**: Revenue divided by number of sales
- **Total Sales**: Count of completed sale transactions

### Inventory KPIs

- **Total Products**: Products with `status = "active"`
- **Low Stock**: Products where `stock <= min_stock`

### Repair KPIs

- **Pending**: Repairs with `status = "pending"`
- **In Progress**: Repairs with `status = "in_progress"`
- **Completed**: Repairs with `status = "completed"`

### Business KPIs

- **Total Suppliers**: Registered supplier count
- **Total Customers**: Unique customer count (based on email across sales and repairs)
