# Administration

Manage users and device keys (admin only).

## User Management

Navigate to **Users** in the sidebar (requires Admin role).

<!-- Screenshot: Users page with table -->

### User Table

| Column | Description |
|--------|-------------|
| Email | Login email address |
| Name | Display name |
| Role | Admin / Mechanic / Seller |
| Status | Active / Inactive |
| Created | Account creation date |

### User Roles

| Role | Permissions |
|------|-------------|
| **Admin** | Full access: all features + user management + device keys |
| **Mechanic** | Products, POS, Repairs, Suppliers, Reports, Service Certificates |
| **Seller** | Products, POS, Reports, Service Certificates |

## Creating Users

1. Click **+ New User** button
2. Fill in the user details
3. Click **Save**

<!-- Screenshot: Create user modal -->

### User Fields

| Field | Required | Description |
|-------|----------|-------------|
| Email | Yes | Login email (must be unique) |
| Display Name | No | Full name for display |
| Password | Yes | Minimum 8 characters |
| Role | Yes | Admin / Mechanic / Seller |

### Role Selection Guide

| Use Case | Recommended Role |
|----------|------------------|
| Workshop owner/manager | Admin |
| Mechanic performing repairs | Mechanic |
| Counter staff making sales | Seller |
| Mixed responsibilities | Admin (gives full access) |

## Editing Users

1. Click the **Edit** button (pencil icon) on a user row
2. Modify the fields:
   - Display Name
   - Role
   - Status
3. Click **Save**

> You cannot change a user's email or password from the edit form. The admin must reset credentials manually if needed.

## Deactivating Users

1. Click the **Edit** button on a user row
2. Change **Status** to "Inactive"
3. Click **Save**

Inactive users:
- Cannot log in
- Are preserved in the database
- Can be reactivated by setting status back to "Active"

> You cannot deactivate your own admin account.

## Device Key Management

Navigate to **Device Keys** in the sidebar (requires Admin role).

<!-- Screenshot: Device Keys page -->

### What are Device Keys?

Device keys provide an additional layer of authentication. When `require_device_key = true` in `server.toml`, each client workstation must present a valid device key to access the API.

### Device Key List

| Column | Description |
|--------|-------------|
| Key | Partial key value (masked for security) |
| Bound IP | IP address the key was first used from |
| Active | Whether the key is currently valid |
| Created | When the key was generated |
| Last Seen | Last time the key was used |

## Generating Device Keys

1. Click **Generate Key** button
2. A new key is created and displayed
3. **Copy the key immediately** - it is shown only once
4. Configure the key in the client's `viewer.toml`:

```toml
[server]
device_key = "the-generated-key-here"
```

<!-- Screenshot: New key generated modal -->

### Security Note

Device keys are shown in full only at generation time. After that, only the masked version is visible in the list.

## Revoking Device Keys

1. Click the **Revoke** button (trash icon) on a key row
2. Confirm the revocation

Revoked keys:
- Immediately lose access
- Cannot be reactivated
- Are removed from the list

## Unbinding Device Keys

Device keys can be unbound from their original IP address:

1. A key is initially bound to the IP address it was first used from
2. If the workstation IP changes, the key needs to be unbound
3. Regenerate a new key instead for security

> **Best practice:** When a workstation's IP changes, revoke the old key and generate a new one rather than unbinding.

## Best Practices

1. **Principle of least privilege**: Assign the lowest role necessary
2. **Regular audits**: Review user list periodically
3. **Deactivate, don't delete**: Use deactivation for temporary access changes
4. **Device keys for LAN**: Enable device keys when the server is accessible over LAN
5. **Unique accounts**: One user account per person
6. **Strong passwords**: Enforce minimum 8 characters
