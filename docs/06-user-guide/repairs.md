# Repair Management

Track motorcycle repairs from intake to delivery.

## Viewing Repairs

Navigate to **Repairs** in the sidebar.

<!-- Screenshot: Repairs page with table -->

### Repair Table

The repair list displays as a data table with columns:

| Column | Description |
|--------|-------------|
| Customer | Customer name |
| Vehicle | Vehicle description (e.g., "Honda CG 150") |
| License Plate | Chilean license plate |
| Priority | High / Medium / Low |
| Status | Pending / In Progress / Completed / Cancelled |
| Created | Date the repair was created |

### Pagination

Repairs are paginated at 10 per page. Use the pagination controls to navigate.

## Creating a Repair Order

1. Click **+ New Repair** button
2. Fill in the repair details
3. Click **Save**

<!-- Screenshot: Create repair modal -->

### Repair Fields

| Field | Required | Description |
|-------|----------|-------------|
| Customer Name | No | Client's name |
| Customer Email | No | Client's email |
| Customer Phone | No | Client's phone number |
| Vehicle | No | Vehicle description (e.g., "Yamaha FZ 250") |
| License Plate | No | Chilean license plate |
| Description | No | Initial description of the problem |
| Priority | Yes | High / Medium / Low |
| Estimated Cost | No | Initial cost estimate (CLP) |
| Estimated Delivery | No | Expected completion date |

### Priority Levels

| Priority | Description | Use Case |
|----------|-------------|----------|
| High | Urgent repair | Safety issues, commercial vehicles |
| Medium | Normal repair | Standard maintenance, part replacements |
| Low | Non-urgent | Cosmetic repairs, optional upgrades |

## License Plate Validation

WorkshopManager validates Chilean license plate formats:

| Format | Example | Description |
|--------|---------|-------------|
| XXXX-9999 | `ABCD-1234` | Old format (4 letters + 4 digits) |
| XX-XX-99 | `BB-KK-12` | New format (2-2-2 alphanumeric) |

> License plate validation is optional. You can leave the field empty if the vehicle doesn't have a plate or if you prefer not to enter it.

## Viewing Repair Details

1. Click the **View** button (eye icon) on a repair row
2. A detail modal opens showing:
   - Customer and vehicle information
   - Current status and priority
   - Diagnosis
   - Parts used (linked to inventory)
   - Cost breakdown
   - Status history/timeline

<!-- Screenshot: Repair detail modal -->

## Updating Repair Status

From the repair detail view:

1. Click **Update Status**
2. Select the new status:
   - **Pending** → **In Progress**: Work has started
   - **In Progress** → **Completed**: Work is finished
   - **Pending** → **Cancelled**: Repair was cancelled
3. Add a description/note (optional)
4. Confirm

Status changes are logged with timestamps for the repair history.

## Adding Diagnosis

1. Open the repair detail view
2. Click **Add Diagnosis**
3. Enter the diagnostic notes
4. Save

Diagnosis notes help track what was found during inspection.

## Adding Parts (Linked to Inventory)

Parts used in a repair can be linked to inventory products:

1. Open the repair detail view
2. Click **Add Part**
3. Fill in:
   - **Name**: Part name (auto-filled if linking to inventory)
   - **Quantity**: Number of units used
   - **Unit Cost**: Cost per unit (CLP)
   - **Product**: Select from inventory (optional)
4. Confirm

### Linking to Inventory

When you select a product from inventory:
- The part name is auto-filled from the product
- The unit cost is auto-filled from the product cost
- Stock is reduced automatically when the part is added

### Adding Custom Parts

If the part is not in inventory, leave the Product field empty and enter the name and cost manually. This tracks the cost without affecting inventory.

## Removing Parts

1. Open the repair detail view
2. Find the part in the parts list
3. Click the **Remove** button
4. Confirm

> Removing a part that was linked to inventory does NOT restore stock automatically.

## Repair Cost Tracking

The repair tracks multiple cost types:

| Cost Type | Description |
|-----------|-------------|
| Estimated Cost | Initial estimate provided to customer |
| Labor Cost | Cost of labor hours |
| Parts Cost | Total cost of parts used (auto-calculated) |
| Final Cost | Total cost charged to customer |

### Cost Breakdown

The detail view shows:
```
Parts:      $45.000
Labor:      $30.000
─────────────────────
Total:      $75.000
```

## Repair History/Updates

Every status change and update is logged in the repair timeline:

| Entry | Description |
|-------|-------------|
| Status Change | "Status changed from Pending to In Progress" |
| Diagnosis | Diagnostic notes added |
| Part Added | Part added to repair |
| Cost Update | Cost estimate or final cost updated |

The timeline is displayed in chronological order in the repair detail view.
