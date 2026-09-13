# Monitoring & Observability — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Operations engineers, system administrators

---

## 1. Health Check Endpoint

### 1.1 Endpoint

```
GET /health
```

**Response:**
```
OK
```

**Status Code:** `200 OK`

### 1.2 Implementation

```rust
// crates/inventory-server/src/main.rs
async fn health_check() -> &'static str {
    "OK"
}
```

### 1.3 Usage

```bash
# Basic health check
curl https://localhost:8443/health

# With authentication (not required for health endpoint)
curl -H "X-WorkshopManager-Key: your-api-key" https://localhost:8443/health
```

---

## 2. Logging Strategy

### 2.1 Framework

WorkshopManager uses `tracing-subscriber` for structured logging.

### 2.2 Log Levels

| Level | Usage | When to Use |
|-------|-------|-------------|
| `ERROR` | System failures | Database connection lost, critical errors |
| `WARN` | Unexpected conditions | Rate limit exceeded, deprecated usage |
| `INFO` | Business events | User login, sale created, backup completed |
| `DEBUG` | Development debugging | SQL queries, request/response details |
| `TRACE` | Verbose debugging | Full request/response bodies |

### 2.3 Log Configuration

```rust
// crates/inventory-server/src/main.rs
tracing_subscriber::fmt()
    .with_max_level(Level::INFO)
    .with_target(false)
    .with_thread_ids(true)
    .init();
```

### 2.4 Log Format

```
2026-09-13T10:30:45.123456Z  INFO workshop_manager::routes::auth: User logged in user_id=550e8400-e29b-41d4-a716-446655440000 email=admin@workshop.cl
```

---

## 3. Metrics to Monitor

### 3.1 System Metrics

| Metric | Description | Threshold | Alert |
|--------|-------------|-----------|-------|
| CPU usage | Server CPU utilization | > 80% sustained | Warning |
| Memory usage | RSS memory consumption | > 500MB | Warning |
| Disk usage | Database disk space | > 80% | Critical |
| File descriptors | Open file handles | > 1000 | Warning |

### 3.2 Application Metrics

| Metric | Description | Threshold | Alert |
|--------|-------------|-----------|-------|
| Request rate | Requests per second | > 100 | Info |
| Response time | P95 response latency | > 1s | Warning |
| Error rate | 5xx responses per minute | > 5 | Warning |
| Active connections | WebSocket connections | > 50 | Info |

### 3.3 Database Metrics

| Metric | Description | Threshold | Alert |
|--------|-------------|-----------|-------|
| Connection pool | Active database connections | > 80% pool size | Warning |
| Query duration | Slow query threshold | > 1s | Warning |
| Transaction rate | Transactions per second | Baseline | Info |
| Cache hit ratio | Buffer cache efficiency | < 90% | Warning |

### 3.4 Security Metrics

| Metric | Description | Threshold | Alert |
|--------|-------------|-----------|-------|
| Failed logins | Failed login attempts | > 10/min | Warning |
| Rate limit hits | Rate limit exceeded events | > 5/min | Warning |
| Auth failures | Authentication errors | > 20/min | Critical |
| Audit log size | Audit table growth | > 1GB | Warning |

---

## 4. Alert Thresholds

### 4.1 Alert Levels

| Level | Description | Action |
|-------|-------------|--------|
| **Critical** | System down or data loss risk | Immediate response |
| **Warning** | Degraded performance or capacity | Investigate within 1 hour |
| **Info** | Notable events | Log and monitor |

### 4.2 Alert Rules

| Rule | Condition | Level | Action |
|------|-----------|-------|--------|
| Service Down | Health check fails | Critical | Restart service |
| High CPU | > 80% for 5 minutes | Warning | Investigate load |
| High Memory | > 500MB for 5 minutes | Warning | Check for leaks |
| Disk Full | > 80% usage | Critical | Free space |
| Slow Queries | > 1s average | Warning | Optimize queries |
| Failed Logins | > 10/min | Warning | Check for attacks |
| Backup Failed | Backup job fails | Critical | Manual backup |

---

## 5. Log File Locations

### 5.1 Server Logs

| Location | Description | Retention |
|----------|-------------|-----------|
| stdout/stderr | Console output | Current session |
| `$LOCALAPPDATA/WorkshopManager/logs/` | Log files (if configured) | 7 days |

### 5.2 Database Logs

| Location | Description | Retention |
|----------|-------------|-----------|
| PostgreSQL logs | Embedded PG logs | 7 days |
| `$LOCALAPPDATA/WorkshopManager/data/` | Data directory | Indefinite |

### 5.3 Backup Logs

| Location | Description | Retention |
|----------|-------------|-----------|
| `$LOCALAPPDATA/WorkshopManager/data/backups/` | Backup files | 7 days |

---

## 6. Performance Monitoring

### 6.1 Key Performance Indicators

| KPI | Target | Measurement |
|-----|--------|-------------|
| Availability | 99.9% | Uptime monitoring |
| Response time (P50) | < 200ms | Request logging |
| Response time (P95) | < 1s | Request logging |
| Response time (P99) | < 2s | Request logging |
| Error rate | < 0.1% | Error logging |

### 6.2 Performance Baselines

| Operation | Expected Duration | Warning Threshold |
|-----------|-------------------|-------------------|
| Login | < 500ms | > 1s |
| Product list | < 200ms | > 500ms |
| Sale creation | < 300ms | > 1s |
| Report generation | < 2s | > 5s |
| Backup | < 5min | > 15min |

### 6.3 Profiling

```bash
# Enable debug logging for profiling
RUST_LOG=debug cargo run -p inventory-server

# Trace specific module
RUST_LOG=inventory_server::routes::auth=trace cargo run -p inventory-server
```

---

## 7. Database Monitoring

### 7.1 Connection Pool Monitoring

```rust
// Check pool statistics
let pool_size = state.db.options().get_max_connections();
let idle = state.db.num_idle();
let active = pool_size - idle;

tracing::info!(
    pool_size = pool_size,
    idle = idle,
    active = active,
    "Database pool statistics"
);
```

### 7.2 Query Monitoring

```rust
// Enable SQL logging
RUST_LOG=sqlx=trace cargo run -p inventory-server

// Monitor slow queries
RUST_LOG=sqlx::query=trace cargo run -p inventory-server
```

### 7.3 Database Health Checks

```sql
-- Check database size
SELECT pg_size_pretty(pg_database_size('workshop_manager'));

-- Check table sizes
SELECT 
    relname AS table_name,
    pg_size_pretty(pg_total_relation_size(relid)) AS total_size
FROM pg_catalog.pg_statio_user_tables
ORDER BY pg_total_relation_size(relid) DESC;

-- Check active connections
SELECT count(*) FROM pg_stat_activity;

-- Check for locks
SELECT * FROM pg_locks WHERE NOT granted;
```

---

## 8. Monitoring Dashboard

### 8.1 Recommended Tools

| Tool | Purpose | Setup |
|------|---------|-------|
| Prometheus | Metrics collection | Optional |
| Grafana | Visualization | Optional |
| pgAdmin | Database monitoring | Optional |
| htop | System monitoring | System tool |

### 8.2 Dashboard Metrics

**System Dashboard:**
- CPU usage over time
- Memory usage over time
- Disk I/O
- Network I/O

**Application Dashboard:**
- Request rate
- Response time distribution
- Error rate
- Active connections

**Database Dashboard:**
- Connection pool usage
- Query duration
- Transaction rate
- Cache hit ratio

---

## 9. Log Analysis

### 9.1 Common Log Queries

```bash
# Find errors
grep -i "error" logs/*.log

# Find slow queries
grep "duration" logs/*.log | awk -F'duration=' '{print $2}' | sort -n

# Find failed logins
grep "Failed login" logs/*.log

# Find rate limit hits
grep "Rate limit" logs/*.log
```

### 9.2 Log Rotation

| Log Type | Rotation | Retention |
|----------|----------|-----------|
| Application logs | Daily | 7 days |
| Database logs | Daily | 7 days |
| Audit logs | Never (append-only) | 7 years |
| Backup logs | With backup | 7 days |

---

## 10. Cross-References

| Document | Description |
|----------|-------------|
| [Installation](../05-deployment/installation.md) | Server setup |
| [Configuration](../05-deployment/configuration.md) | Server configuration |
| [Backup Recovery](../05-deployment/backup-recovery.md) | Backup procedures |
| [Troubleshooting](../05-deployment/troubleshooting.md) | Common issues |
| [Security Overview](../04-security/overview.md) | Security architecture |
