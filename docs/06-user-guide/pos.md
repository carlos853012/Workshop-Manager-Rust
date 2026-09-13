# Point of Sale (POS)

Process sales quickly with the barcode-driven POS interface.

## Opening the POS

Navigate to **POS** in the sidebar.

<!-- Screenshot: POS interface with empty cart -->

The POS screen is divided into two areas:

- **Left**: Barcode input and cart items
- **Right**: Cart summary and payment controls

## Adding Items to Cart

### Method 1: Barcode Scanner

1. Click the barcode input field (it auto-focuses when the POS opens)
2. Scan a barcode with your barcode scanner
3. The product is added to the cart automatically
4. If the product is already in the cart, the quantity increases by 1

### Method 2: Manual Entry

1. Click the barcode input field
2. Type the barcode number manually
3. Press **Enter**

### Product Not Found

If the barcode is not recognized, the error message "Producto no encontrado" appears. Verify the barcode or check that the product exists in inventory.

## Adjusting Quantities

Each cart item shows:

| Control | Description |
|---------|-------------|
| **-** button | Decrease quantity by 1 |
| **+** button | Increase quantity by 1 |
| Quantity display | Current quantity for this item |

If quantity reaches 0, the item is removed from the cart.

## Removing Items

Click the **X** button on a cart item to remove it completely from the cart.

## Customer Information

Optionally fill in customer details before completing the sale:

| Field | Description |
|-------|-------------|
| Customer Name | Client's name (optional) |
| Payment Method | Cash, Card, or Transfer |

Customer information is saved with the sale for reporting purposes.

## Payment Methods

Select one of three payment methods:

| Method | Icon | Description |
|--------|------|-------------|
| **Cash** | Cash | Physical cash payment |
| **Card** | Credit Card | Credit/debit card payment |
| **Transfer** | Transfer | Bank transfer |

## Discounts

Apply a discount to the entire sale:

1. Enter the discount amount in the **Discount** field (CLP)
2. The total updates automatically
3. Discount is applied before IVA calculation

> Discounts are applied to the sale total, not individual items.

## Confirming a Sale

1. Verify all items and quantities in the cart
2. Select the payment method
3. Optionally enter customer information and discount
4. Click **Confirm Sale** (or press the confirm button)

### After Confirmation

- The sale is recorded in the database
- Stock is automatically reduced for each item sold
- A success message appears
- The cart is cleared for the next sale

### Error Handling

If a sale fails (e.g., insufficient stock), an error message explains the issue. Fix the problem and try again.

## IVA Calculation

WorkshopManager follows Chilean tax law:

- **IVA rate**: 19%
- **Prices include IVA**: Product prices are entered with IVA included
- **IVA extraction**: The system calculates the base amount and IVA portion

### How IVA Works

For a product priced at `$11.900`:
- Base price: `$10.000`
- IVA (19%): `$1.900`
- Total: `$11.900`

The POS displays an IVA breakdown for transparency:

| Line | Amount |
|------|--------|
| Subtotal (base) | $10.000 |
| IVA (19%) | $1.900 |
| Discount | -$0 |
| **Total** | **$11.900** |

### Rounding

Chilean law requires rounding to the nearest multiple of 10:
- `$15.678` rounds to `$15.680`
- `$15.673` rounds to `$15.670`
- `$999` rounds to `$1.000`

All amounts are automatically rounded.

## Sale History

Completed sales are visible in the **Sales** section. Each sale record includes:

- Sale ID
- Customer information (if provided)
- Items purchased
- Total amount
- Payment method
- Date and time

<!-- Screenshot: Completed sale confirmation -->
