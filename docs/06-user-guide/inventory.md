# Inventory Management

Manage your product catalog: add, edit, search, and track stock levels.

## Viewing Products

Navigate to **Products** in the sidebar.

<!-- Screenshot: Products page with table -->

### Product Table

The product list displays as a data table with columns:

| Column | Description |
|--------|-------------|
| Name | Product name |
| Category | Product category (optional) |
| Brand | Brand/manufacturer (optional) |
| Price | Selling price in CLP |
| Stock | Current quantity in stock |
| Min Stock | Minimum stock threshold |
| Status | Active or Deleted |

### Pagination

- Products are displayed **10 per page**
- Use the pagination controls at the bottom of the table to navigate
- The total count is shown (e.g., "Showing 1-10 of 45")

### Searching Products

Use the search bar to filter products by name, category, brand, or model. Results update as you type.

You can also search by **barcode** using the barcode search field - this is the same field used in the POS.

## Creating a Product

1. Click the **+ New Product** button (top right of the Products page)
2. Fill in the product details
3. Click **Save**

### Product Fields

| Field | Required | Description |
|-------|----------|-------------|
| Name | Yes | Product name (e.g., "Filtro de aceite Honda CG") |
| Description | No | Detailed description |
| Category | No | Product category (e.g., "Filtros", "Aceites", "Pastillas") |
| Brand | No | Manufacturer brand (e.g., "Honda", "Yamaha") |
| Model | No | Compatible model |
| SKU | No | Internal stock keeping unit |
| Barcode | No | EAN-13 barcode (auto-generated if empty) |
| Price | Yes | Selling price in CLP (includes IVA) |
| Cost | Yes | Purchase cost in CLP |
| Stock | Yes | Initial stock quantity |
| Min Stock | Yes | Minimum stock threshold for alerts |
| Location | No | Physical storage location (e.g., "Estante A-3") |
| Supplier | No | Linked supplier (from supplier list) |

<!-- Screenshot: Create product modal -->

### Pricing

- **Price**: The selling price shown to customers. In Chile, this includes 19% IVA.
- **Cost**: Your purchase cost. Used for profit margin calculations.
- The system tracks margins automatically.

### Stock

- **Stock**: Current quantity on hand
- **Min Stock**: When stock falls to or below this level, the product appears in low stock alerts on the Dashboard

## Barcode Generation (EAN-13)

If you leave the **Barcode** field empty when creating a product, WorkshopManager auto-generates an EAN-13 barcode:

- Format: 6-digit workshop prefix + 6-digit item number + 1 check digit
- The workshop prefix is auto-generated from your workshop ID
- Barcodes are unique per product

You can also enter your own barcode manually (must be valid EAN-13 format).

## Editing a Product

1. Click the **Edit** button (pencil icon) on a product row
2. Modify the fields
3. Click **Save**

> Editing a product does not change its barcode. To change a barcode, delete and recreate the product.

## Deleting a Product

1. Click the **Delete** button (trash icon) on a product row
2. Confirm the deletion in the dialog

Deletion is **soft-delete**: the product is marked as `status = "deleted"` and disappears from the list, but the record is preserved in the database for historical integrity.

## Stock Management

### Adjusting Stock

1. Click the **Stock** button on a product row
2. Enter the quantity adjustment (positive to add, negative to subtract)
3. Confirm

### Stock Tracking

Stock is automatically adjusted when:

- A sale is completed (stock decreases by quantity sold)
- A repair uses parts linked to inventory (stock decreases)
- You manually adjust stock

## Low Stock Alerts

Products at or below their minimum stock threshold appear on:

- **Dashboard**: Low Stock KPI card
- **Products page**: Visual indicator on the stock column

To configure alerts, set the **Min Stock** field appropriately when creating or editing products.

## Supplier Linking

Each product can be linked to one supplier:

1. Create suppliers first (see [Suppliers](suppliers.md))
2. When creating/editing a product, select a supplier from the dropdown
3. The supplier is shown in the product details

Supplier linking helps track which supplier provides which products for reorder purposes.
