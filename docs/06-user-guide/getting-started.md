# Getting Started

First-time setup guide for WorkshopManager.

## Launching the Application

WorkshopManager consists of two programs:

1. **Server** (`inventory-server`) - Must be running first
2. **Viewer** (`inventory-viewer`) - Desktop UI that connects to the server

### Starting the Server

```powershell
cd target\release
.\inventory-server.exe
```

Wait until you see:
```
INFO Server listening on https://127.0.0.1:8443
```

### Starting the Viewer

In a second terminal:

```powershell
cd target\release
.\inventory-viewer.exe
```

The desktop application will open.

<!-- Screenshot: WorkshopManager login screen -->

## Initial Setup Wizard

When the server is fresh (no workshop configured), the viewer automatically shows the setup wizard.

### Step 1: Workshop Profile

Enter your workshop information:

| Field | Description | Required |
|-------|-------------|----------|
| Workshop Name | Business name (e.g., "Taller Moto Express") | Yes |
| Address | Street address | Yes |
| City | City name | Yes |

<!-- Screenshot: Workshop profile form -->

### Step 2: Admin Account

Create the administrator account:

| Field | Description | Required |
|-------|-------------|----------|
| Full Name | Display name for the admin | Yes |
| Email | Login email address | Yes |
| Password | Minimum 8 characters | Yes |

<!-- Screenshot: Admin account form -->

> The admin account has full access to all features including user management.

### Step 3: Confirmation

The wizard confirms the workshop and admin account were created successfully. Click **Continue** to proceed to login.

<!-- Screenshot: Setup complete confirmation -->

## First Login

1. Enter the email and password created during setup
2. Click **Login**
3. You will see the Dashboard

<!-- Screenshot: Login screen with credentials filled -->

## Dashboard Overview

After your first login, you land on the **Dashboard** with KPI cards:

| Card | Description |
|------|-------------|
| Total Products | Number of products in inventory |
| Total Sales | Number of completed sales |
| Pending Repairs | Repairs awaiting completion |
| Total Suppliers | Registered suppliers |
| Revenue | Total revenue (CLP) |
| Average Sale | Average sale amount |
| Total Customers | Unique customers |
| In-Progress Repairs | Currently active repairs |

<!-- Screenshot: Dashboard with KPI cards -->

## Navigation

Use the left sidebar to navigate between sections:

| Section | Icon | Description |
|---------|------|-------------|
| Dashboard | Home | KPI overview |
| Products | Box | Inventory management |
| POS | Cart | Point of Sale |
| Repairs | Wrench | Repair orders |
| Suppliers | Truck | Supplier management |
| Reports | Chart | Client reports |
| Service Certificates | Document | Generate service certificates |
| Users | People | User management (admin only) |
| Device Keys | Key | Device authentication (admin only) |

<!-- Screenshot: Sidebar navigation -->

## Next Steps

- [Inventory](inventory.md) - Add your first products
- [POS](pos.md) - Make your first sale
- [Repairs](repairs.md) - Create a repair order
