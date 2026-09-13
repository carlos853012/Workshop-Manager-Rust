# Maintenance Procedures — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Operations engineers, system administrators

---

## 1. Routine Maintenance Tasks

### 1.1 Daily Tasks

| Task | Description | Time Required | Automation |
|------|-------------|---------------|------------|
| Health check | Verify service is running | 1 minute | Manual |
| Log review | Check for errors or warnings | 5 minutes | Manual |
| Backup verification | Confirm backup completed | 2 minutes | Manual |
| Disk space check | Monitor storage usage | 1 minute | Manual |

### 1.2 Weekly Tasks

| Task | Description | Time Required | Automation |
|------|-------------|---------------|------------|
| Database maintenance | Run VACUUM and ANALYZE | 15 minutes | Manual |
| Log rotation | Archive old logs | 5 minutes | Script |
| Security review | Check audit logs | 10 minutes | Manual |
| Performance review | Check response times | 5 minutes | Manual |

### 1.3 Monthly Tasks

| Task | Description | Time Required | Automation |
|------|-------------|---------------|------------|
| Dependency updates | Check for updates | 30 minutes | Manual |
| Security patches | Apply security updates | 1 hour | Manual |
| Backup test | Test restore process | 30 minutes | Manual |
| Capacity review | Assess resource usage | 15 minutes | Manual |

---

## 2. Database Maintenance

### 2.1 VACUUM

Reclaims storage and updates statistics:

```sql
-- Manual VACUUM
VACUUM ANALYZE;

-- Check vacuum status
SELECT 
    relname,
    n_live_tup,
    n_dead_tup,
    last_vacuum,
    last_autovacuum
FROM pg_stat_user_tables;
```

### 2.2 ANALYZE

Updates table statistics for query optimizer:

```sql
-- Manual ANALYZE
ANALYZE;

-- Check statistics
SELECT 
    relname,
    last_analyze,
    last_autoanalyze
FROM pg_stat_user_tables;
```

### 2.3 Index Maintenance

```sql
-- Check index usage
SELECT 
    indexrelname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes;

-- Rebuild unused indexes
REINDEX INDEX idx_users_email;
```

### 2.4 Disk Space Management

```sql
-- Check database size
SELECT pg_size_pretty(pg_database_size('workshop_manager'));

-- Check table sizes
SELECT 
    relname AS table_name,
    pg_size_pretty(pg_total_relation_size(relid)) AS total_size,
    pg_size_pretty(pg_relation_size(relid)) AS table_size,
    pg_size_pretty(pg_indexes_size(relid)) AS index_size
FROM pg_catalog.pg_statio_user_tables
ORDER BY pg_total_relation_size(relid) DESC;
```

---

## 3. Backup Verification

### 3.1 Backup Schedule

| Backup Type | Frequency | Retention | Location |
|-------------|-----------|-----------|----------|
| Full backup | Every 24 hours | 7 days | `$LOCALAPPDATA/WorkshopManager/data/backups/` |
| WAL archives | Continuous | 7 days | Same location |
| Manual backup | As needed | Indefinite | User-specified |

### 3.2 Verification Process

1. **Check backup exists**
   ```bash
   ls -la $LOCALAPPDATA/WorkshopManager/data/backups/
   ```

2. **Verify backup integrity**
   ```bash
   # Check file is not corrupted
   gunzip -t backup_YYYYMMDD_HHMMSS.sql.gz
   ```

3. **Test restore** (monthly)
   ```bash
   # Restore to test database
   gunzip -c backup_YYYYMMDD_HHMMSS.sql.gz | psql -U postgres workshop_manager_test
   ```

4. **Verify data**
   ```sql
   -- Check row counts
   SELECT 
       relname,
       n_live_tup
   FROM pg_stat_user_tables
   ORDER BY relname;
   ```

### 3.3 Backup Monitoring

| Metric | Threshold | Alert |
|--------|-----------|-------|
| Backup age | > 24 hours | Critical |
| Backup size | < 1KB | Warning |
| Restore time | > 10 minutes | Warning |
| Disk space | < 1GB free | Critical |

---

## 4. Certificate Renewal

### 4.1 TLS Certificates

WorkshopManager uses self-signed certificates generated with `rcgen`.

| Certificate | Location | Renewal |
|-------------|----------|---------|
| Server cert | `$LOCALAPPDATA/WorkshopManager/data/cert.pem` | On demand |
| Private key | `$LOCALAPPDATA/WorkshopManager/data/key.pem` | On demand |

### 4.2 Renewal Process

1. **Backup existing certificates**
   ```bash
   cp cert.pem cert.pem.backup
   cp key.pem key.pem.backup
   ```

2. **Generate new certificates**
   - Restart server (auto-generates if missing)
   - Or use `rcgen` tool manually

3. **Update clients**
   - Distribute new CA certificate
   - Update client trust stores

4. **Verify**
   ```bash
   curl -k https://localhost:8443/health
   ```

### 4.3 Certificate Monitoring

| Check | Frequency | Method |
|-------|-----------|--------|
| Expiration | Weekly | `openssl x509 -enddate -noout -in cert.pem` |
| Validity | Daily | Health check endpoint |
| Chain | On renewal | Visual inspection |

---

## 5. Dependency Updates

### 5.1 Update Process

1. **Check for updates**
   ```bash
   cargo outdated --workspace
   ```

2. **Review changelogs**
   - Check for breaking changes
   - Review security patches
   - Assess compatibility

3. **Update dependencies**
   ```bash
   # Update specific dependency
   cargo update -p dependency_name

   # Update all
   cargo update
   ```

4. **Test**
   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   ```

5. **Commit**
   ```bash
   git add Cargo.lock
   git commit -m "chore: update dependencies"
   ```

### 5.2 Update Categories

| Category | Frequency | Risk | Testing |
|----------|-----------|------|---------|
| Security patches | Immediate | Low | Full test suite |
| Minor updates | Monthly | Low | Full test suite |
| Major updates | Quarterly | Medium | Extended testing |
| Rust version | As needed | Medium | Full test suite |

---

## 6. Security Patches

### 6.1 Security Update Process

1. **Monitor advisories**
   - Rust security advisories
   - Dependency vulnerabilities
   - CVE databases

2. **Assess impact**
   - Is WorkshopManager affected?
   - What is the risk level?
   - Is a fix available?

3. **Apply patch**
   ```bash
   # Check for vulnerabilities
   cargo audit

   # Apply security updates
   cargo update -p vulnerable_crate
   ```

4. **Test and deploy**
   ```bash
   cargo build --workspace
   cargo test --workspace
   ```

### 6.2 Vulnerability Response

| Severity | Response Time | Action |
|----------|---------------|--------|
| Critical | 24 hours | Emergency patch |
| High | 1 week | Scheduled update |
| Medium | 1 month | Regular update |
| Low | Next release | Regular update |

---

## 7. Upgrade Procedures

### 7.1 Version Upgrade Process

1. **Backup**
   ```bash
   # Create backup before upgrade
   # (Automatic backup runs every 24h)
   ```

2. **Download new version**
   - Download latest release
   - Verify checksum

3. **Stop service**
   ```bash
   # Stop current server
   ```

4. **Run migrations**
   ```bash
   # Migrations run automatically on startup
   ```

5. **Start new version**
   ```bash
   # Start new server
   ```

6. **Verify**
   ```bash
   curl -k https://localhost:8443/health
   ```

### 7.2 Database Migrations

| Migration | Version | Description |
|-----------|---------|-------------|
| 001_initial | 0.1.0 | Initial schema |
| 002_add_workshop_id | 0.1.0 | Multi-tenant support |
| 003_add_device_keys | 0.1.0 | Device key management |
| 004_add_audit_log | 0.1.0 | Audit trail |
| 005_add_backups | 0.1.0 | Backup scheduler |
| 006_add_rate_limiting | 0.1.0 | Rate limiter |
| 007_add_tls | 0.1.0 | TLS certificates |
| 008_add_features | 0.1.0 | Feature flags |

### 7.3 Rollback Procedure

If upgrade fails:

1. **Stop new version**
   ```bash
   # Stop server
   ```

2. **Restore backup**
   ```bash
   # Restore database from backup
   gunzip -c backup_YYYYMMDD_HHMMSS.sql.gz | psql -U postgres workshop_manager
   ```

3. **Start old version**
   ```bash
   # Start previous server version
   ```

4. **Verify**
   ```bash
   curl -k https://localhost:8443/health
   ```

5. **Investigate**
   - Review logs
   - Identify issue
   - Plan fix

---

## 8. Maintenance Schedule

### 8.1 Weekly Schedule

| Day | Task | Time |
|-----|------|------|
| Monday | Database maintenance | 09:00 |
| Tuesday | Log rotation | 09:00 |
| Wednesday | Security review | 09:00 |
| Thursday | Performance review | 09:00 |
| Friday | Backup verification | 09:00 |

### 8.2 Monthly Schedule

| Week | Task | Time |
|------|------|------|
| 1st | Dependency check | Monday 09:00 |
| 2nd | Security patch review | Monday 09:00 |
| 3rd | Backup restore test | Monday 09:00 |
| 4th | Capacity review | Monday 09:00 |

---

## 9. Maintenance Logs

### 9.1 Log Format

```
[DATE] [TASK] [STATUS] [DURATION] [NOTES]
```

### 9.2 Example Entries

```
[2026-09-13 09:00] [VACUUM] [SUCCESS] [5m] [Reclaimed 100MB]
[2026-09-13 09:15] [LOG ROTATION] [SUCCESS] [2m] [Archived 50MB]
[2026-09-13 09:30] [SECURITY REVIEW] [SUCCESS] [10m] [No issues found]
[2026-09-13 09:45] [BACKUP VERIFY] [SUCCESS] [3m] [Backup intact]
```

---

## 10. Cross-References

| Document | Description |
|----------|-------------|
| [Monitoring](./monitoring.md) | Observability and alerting |
| [Incident Response](./incident-response.md) | Incident playbook |
| [Backup Recovery](../05-deployment/backup-recovery.md) | Backup procedures |
| [Installation](../05-deployment/installation.md) | Server setup |
| [Configuration](../05-deployment/configuration.md) | Server configuration |
