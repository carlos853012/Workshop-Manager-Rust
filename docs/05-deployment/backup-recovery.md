# Backup & Recovery

Backup strategy, procedures, and recovery instructions for WorkshopManager.

## Automatic Backup Schedule

The server runs an automatic backup scheduler that:

1. Creates a backup **immediately on startup**
2. Creates a backup **every 24 hours**
3. Prunes old backups, keeping the **7 most recent**

The scheduler runs as a background task within the server process.

## Backup Location and Format

### Location

| Platform | Path |
|----------|------|
| Windows | `%LOCALAPPDATA%\WorkshopManager\data\backups\` |
| Linux | `~/.local/share/WorkshopManager/data/backups/` |

### File Format

Backups use a three-layer format:

```
workshop_manager_backup_YYYYMMDD_HHMMSS.sql.gz.enc
```

| Layer | Format | Description |
|-------|--------|-------------|
| 1 | SQL dump | `pg_dump` output with `--clean --if-exists` |
| 2 | gzip | Standard gzip compression |
| 3 | AES-256-GCM | Encrypted with the server's `.crypto_key` |

Example: `workshop_manager_backup_20260913_143022.sql.gz.enc`

### Retention Policy

- **7 most recent backups** are kept
- Older backups are automatically deleted after each successful backup
- Pruning occurs immediately after each backup

## Manual Backup Procedures

### Creating a Manual Backup

There is no built-in CLI command for manual backups. To create one manually, stop the server and copy the database files, or use `pg_dump` directly.

**Using the embedded pg_dump:**

```powershell
# Windows (from the server data directory)
$env:PGPASSWORD = ""  # Embedded PostgreSQL has no password
& "$env:LOCALAPPDATA\WorkshopManager\data\pg_dump\pg_dump.exe" --dbname "host=127.0.0.1 port=5433 dbname=workshop_manager" --clean --if-exists > backup.sql
```

```bash
# Linux
~/.local/share/WorkshopManager/data/pg_dump/pg_dump \
  --dbname "host=127.0.0.1 port=5433 dbname=workshop_manager" \
  --clean --if-exists > backup.sql
```

### Backing Up Manually (Copy Data Directory)

The simplest approach: copy the entire data directory while the server is stopped.

```powershell
# Stop the server first, then:
Copy-Item -Recurse "$env:LOCALAPPDATA\WorkshopManager\data" -Destination "C:\backups\workshop-$(Get-Date -Format yyyyMMdd)"
```

### Compressing a Backup

```powershell
# Using PowerShell
Compress-Archive -Path "backup.sql" -DestinationPath "backup.zip"
```

## Recovery Procedures

### Recovering from an Automatic Backup

Automatic backups are encrypted with the server's `.crypto_key`. To restore:

1. **Stop the server** if running
2. **Ensure the `.crypto_key` exists** in the data directory (same key used during backup)
3. **Decrypt and decompress** the backup:

```powershell
# The backup files are encrypted - you must use the same crypto key
# For encrypted backups (.enc), use the server's decryption logic
# Or restore from a raw SQL dump if you created one manually
```

4. **Restore to PostgreSQL:**

```powershell
& "$env:LOCALAPPDATA\WorkshopManager\data\pg_dump\pg_restore.exe" \
  --dbname "host=127.0.0.1 port=5433 dbname=workshop_manager" \
  --clean --if-exists backup.sql
```

5. **Start the server** - migrations will run automatically

### Recovering from Data Directory Copy

1. **Stop the server**
2. **Replace the data directory** with the backup copy
3. **Start the server** - migrations will run automatically

### Full Disaster Recovery

1. Install Rust and build the project (see [installation.md](installation.md))
2. Copy `config/` directory with your configuration
3. Restore the data directory from backup
4. Start the server

## Backup Verification

### Checking Backup Files

```powershell
# List backups sorted by date
Get-ChildItem "$env:LOCALAPPDATA\WorkshopManager\data\backups\*.enc" |
  Sort-Object LastWriteTime -Descending |
  Select-Object Name, Length, LastWriteTime
```

### Verifying Backup Integrity

Backups are encrypted AES-256-GCM. To verify a backup is readable:

1. Ensure the `.crypto_key` exists in the data directory
2. Attempt to decrypt using the server's crypto module
3. Check that the decrypted content is valid SQL

### Automated Verification

The server logs backup operations:

```
INFO Encrypted backup created: .../backups/workshop_manager_backup_20260913_143022.sql.gz.enc
INFO Old backup removed: .../backups/workshop_manager_backup_20260906_143022.sql.gz.enc
```

Failed backups are logged as errors:

```
ERROR Backup failed: ...
ERROR Backup pruning failed: ...
```

## Best Practices

1. **Test recovery** periodically by restoring a backup to a test environment
2. **Copy backups off-site** - the automatic backups stay on the same machine
3. **Monitor disk space** - each backup is ~1-10 MB compressed
4. **Keep the `.crypto_key` safe** - without it, encrypted backups cannot be restored
5. **Document your backup procedure** for your team
