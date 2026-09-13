# Incident Response Playbook — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Operations engineers, system administrators

---

## 1. Incident Severity Levels

### 1.1 Severity Classification

| Level | Name | Description | Response Time | Examples |
|-------|------|-------------|---------------|----------|
| **P1** | Critical | System down, data loss, security breach | Immediate | Service outage, data breach, unauthorized access |
| **P2** | High | Major feature degraded, security risk | 1 hour | Database issues, authentication failures |
| **P3** | Medium | Minor feature degraded, performance issues | 4 hours | Slow responses, minor bugs |
| **P4** | Low | Cosmetic issues, documentation gaps | 24 hours | UI glitches, typos |

### 1.2 Escalation Matrix

| Severity | Initial Responder | Escalation | Management |
|----------|-------------------|------------|------------|
| P1 | On-call engineer | Team lead | Workshop owner |
| P2 | On-call engineer | Team lead | — |
| P3 | Developer | On-call engineer | — |
| P4 | Developer | — | — |

---

## 2. Response Procedures

### 2.1 General Response Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Detection   │────▶│  Assessment  │────▶│  Containment │────▶│  Resolution  │
│              │     │              │     │              │     │              │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
       │                    │                    │                    │
       ▼                    ▼                    ▼                    ▼
  ┌──────────┐        ┌──────────┐        ┌──────────┐        ┌──────────┐
  │  Log     │        │  Classify│        │  Mitigate│        │  Document│
  │  Event   │        │  Severity│        │  Impact  │        │  Lessons │
  └──────────┘        └──────────┘        └──────────┘        └──────────┘
```

### 2.2 Response Checklist

**Immediate (0-15 minutes):**
- [ ] Acknowledge alert
- [ ] Assess severity level
- [ ] Notify stakeholders (if P1/P2)
- [ ] Begin incident log

**Short-term (15-60 minutes):**
- [ ] Identify root cause
- [ ] Implement workaround
- [ ] Communicate status
- [ ] Monitor impact

**Resolution (1-4 hours):**
- [ ] Implement fix
- [ ] Verify resolution
- [ ] Restore normal operations
- [ ] Confirm no data loss

**Post-incident (24-48 hours):**
- [ ] Write postmortem
- [ ] Identify improvements
- [ ] Update runbooks
- [ ] Schedule follow-up

---

## 3. Common Incidents

### 3.1 Data Breach

**Symptoms:**
- Unauthorized access detected in audit logs
- Unusual data export patterns
- Multiple failed login attempts from unknown IPs

**Response:**
1. **Immediately** revoke compromised credentials
2. **Isolate** affected systems
3. **Preserve** audit logs and evidence
4. **Notify** workshop owner
5. **Assess** scope of breach
6. **Report** to authorities (if required)

**Communication Template:**
```
SUBJECT: Security Incident - [Brief Description]

We have detected a potential security incident affecting WorkshopManager.

Impact: [Description of impact]
Actions Taken: [List of immediate actions]
Current Status: [Investigating/Mitigating/Resolved]
Next Steps: [Planned actions]

We will provide updates every [timeframe] until resolution.
```

### 3.2 Service Outage

**Symptoms:**
- Health check endpoint unreachable
- Application not responding
- Database connection failures

**Response:**
1. **Verify** outage is real (not network issue)
2. **Check** server process status
3. **Review** application logs
4. **Restart** service if needed
5. **Monitor** recovery
6. **Notify** users of restoration

**Recovery Steps:**
```bash
# Check server status
curl -k https://localhost:8443/health

# Check process
tasklist | findstr inventory-server

# Restart server (if needed)
# Windows: Restart service or run executable
# Linux: systemctl restart workshop-manager
```

### 3.3 Unauthorized Access

**Symptoms:**
- Failed login attempts exceeding normal
- Access from unusual locations
- Privilege escalation attempts

**Response:**
1. **Block** suspicious IP addresses
2. **Revoke** compromised API keys
3. **Reset** affected user passwords
4. **Review** audit logs
5. **Enhance** monitoring
6. **Document** incident

### 3.4 Database Issues

**Symptoms:**
- Slow query performance
- Connection pool exhaustion
- Disk space warnings

**Response:**
1. **Check** database logs
2. **Monitor** connection pool
3. **Identify** slow queries
4. **Optimize** or kill problematic queries
5. **Increase** pool size if needed
6. **Plan** capacity upgrade

### 3.5 Backup Failure

**Symptoms:**
- Backup job not completing
- Backup integrity check failing
- Insufficient disk space for backups

**Response:**
1. **Check** backup logs
2. **Verify** disk space
3. **Run** manual backup
4. **Test** restore process
5. **Notify** if data at risk
6. **Schedule** recurring fix

---

## 4. Communication Templates

### 4.1 Initial Notification

```
SUBJECT: [P1/P2] Incident Detected - WorkshopManager

Incident ID: INC-YYYY-MMDD-XXX
Severity: [P1/P2/P3/P4]
Status: Investigating
Detected: [Timestamp]
Impact: [Description]

Current Actions:
1. [Action 1]
2. [Action 2]

Next Update: [Time]
```

### 4.2 Status Update

```
SUBJECT: [Update] Incident INC-YYYY-MMDD-XXX

Status: [Investigating/Mitigating/Resolved]
Duration: [Time since detection]

Progress:
- [Update 1]
- [Update 2]

Next Steps:
- [Planned action]

Next Update: [Time]
```

### 4.3 Resolution Notice

```
SUBJECT: [Resolved] Incident INC-YYYY-MMDD-XXX

Status: Resolved
Duration: [Total time]
Root Cause: [Brief description]

Actions Taken:
1. [Action 1]
2. [Action 2]

Impact Summary:
- [Impact details]

Postmortem: [Link to document]
```

---

## 5. Post-Incident Review

### 5.1 Postmortem Template

```markdown
# Incident Postmortem

## Metadata
- **Incident ID:** INC-YYYY-MMDD-XXX
- **Date:** YYYY-MM-DD
- **Duration:** X hours Y minutes
- **Severity:** P1/P2/P3/P4
- **Author:** [Name]

## Summary
[1-2 sentence summary of the incident]

## Impact
- **Users affected:** [Number]
- **Data loss:** [Yes/No]
- **Revenue impact:** [If applicable]

## Timeline
| Time | Event |
|------|-------|
| HH:MM | Detection |
| HH:MM | Investigation started |
| HH:MM | Mitigation applied |
| HH:MM | Resolution confirmed |

## Root Cause
[Detailed explanation of what caused the incident]

## Resolution
[How the incident was resolved]

## Lessons Learned
1. [Lesson 1]
2. [Lesson 2]

## Action Items
| Item | Owner | Due Date | Status |
|------|-------|----------|--------|
| [Action] | [Name] | [Date] | [Status] |
```

### 5.2 Review Meeting Agenda

1. **Incident Overview** (5 min)
   - What happened
   - When it happened
   - Impact

2. **Timeline Review** (10 min)
   - Detection time
   - Response time
   - Resolution time

3. **Root Cause Analysis** (15 min)
   - Technical cause
   - Process gaps
   - Communication issues

4. **Action Items** (10 min)
   - Improvements
   - Owners
   - Deadlines

5. **Lessons Learned** (5 min)
   - What went well
   - What could improve
   - Follow-up items

---

## 6. Recovery Procedures

### 6.1 Service Recovery

```bash
# Step 1: Verify outage
curl -k https://localhost:8443/health

# Step 2: Check logs
# Review application logs for errors

# Step 3: Restart service
# Windows: Restart the executable
# Linux: systemctl restart workshop-manager

# Step 4: Verify recovery
curl -k https://localhost:8443/health

# Step 5: Monitor
# Watch logs for 15-30 minutes
```

### 6.2 Database Recovery

```bash
# Step 1: Check database status
# Review PostgreSQL logs

# Step 2: Verify disk space
df -h

# Step 3: Check connections
SELECT count(*) FROM pg_stat_activity;

# Step 4: Restore from backup (if needed)
# Use backup recovery procedures

# Step 5: Verify data integrity
# Run consistency checks
```

### 6.3 Data Recovery

```bash
# Step 1: Identify data loss scope
# Review audit logs

# Step 2: Restore from backup
# Use backup recovery procedures

# Step 3: Verify restored data
# Compare with audit logs

# Step 4: Resume operations
# Monitor for issues
```

---

## 7. Incident Tools

### 7.1 Diagnostic Commands

```bash
# System status
tasklist | findstr inventory-server
systeminfo | findstr /C:"Total Physical Memory"

# Network status
netstat -an | findstr 8443
curl -k https://localhost:8443/health

# Database status
psql -U postgres -c "SELECT * FROM pg_stat_activity;"

# Log analysis
grep -i "error" /path/to/logs/*.log
grep -i "panic" /path/to/logs/*.log
```

### 7.2 Recovery Scripts

```bash
#!/bin/bash
# incident-check.sh - Quick incident assessment

echo "=== System Status ==="
tasklist | findstr inventory-server

echo "=== Health Check ==="
curl -k https://localhost:8443/health

echo "=== Disk Space ==="
df -h

echo "=== Recent Logs ==="
tail -n 100 /path/to/logs/*.log | grep -i "error\|panic\|critical"
```

---

## 8. Cross-References

| Document | Description |
|----------|-------------|
| [Monitoring](./monitoring.md) | Observability and alerting |
| [Maintenance](./maintenance.md) | Preventive maintenance |
| [Troubleshooting](../05-deployment/troubleshooting.md) | Common issues |
| [Backup Recovery](../05-deployment/backup-recovery.md) | Backup procedures |
| [Security Overview](../04-security/overview.md) | Security architecture |
