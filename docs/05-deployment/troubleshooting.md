# Troubleshooting

Common issues and solutions for WorkshopManager.

## Server Won't Start

### Port 8443 Already in Use

**Symptoms:**
```
ERROR Failed to start server: Address already in use (os error 10048)
```

**Windows:**
```powershell
netstat -ano | findstr :8443
taskkill /PID <pid> /F
```

**Linux:**
```bash
ss -tlnp | grep 8443
sudo kill <pid>
```

### Database Locked

**Symptoms:**
```
ERROR Failed to start embedded PostgreSQL: database is locked
```

**Solutions:**

1. Ensure no other WorkshopManager server is running
2. Check for leftover PostgreSQL processes:

**Windows:**
```powershell
tasklist | findstr postgres
taskkill /IM postgres.exe /F
```

**Linux:**
```bash
ps aux | grep postgres | grep workshop
sudo kill <pid>
```

3. Delete the `postmaster.pid` file in the data directory
4. Restart the server

### Missing Data Directory

**Symptoms:**
```
ERROR Failed to create data directory
```

**Windows:**
```powershell
New-Item -ItemType Directory -Force -Path "$env:LOCALAPPDATA\WorkshopManager\data"
```

**Linux:**
```bash
sudo mkdir -p /var/lib/workshop-server/.local/share/WorkshopManager/data
sudo chown -R workshop-server:workshop-server /var/lib/workshop-server
```

### TLS Certificate Error

**Symptoms:**
```
ERROR Failed to generate self-signed certificate
```

**Solution:** Ensure the data directory is writable. Check that `rcgen` can generate keys (rare issue).

## Viewer Can't Connect

### TLS Certificate Error

**Symptoms:** Viewer shows connection error or TLS handshake failure.

**Solutions:**

1. Verify the server is running:

**Windows:**
```powershell
curl https://127.0.0.1:8443/health -k
```

**Linux:**
```bash
curl -k https://localhost:8443/health
```

2. Check `tls_accept_invalid_certs = true` in `config/viewer.toml`
3. If using custom certificates, ensure the certificate is trusted by the system

### API Key Mismatch

**Symptoms:**
```
401 Unauthorized: Invalid API key
```

**Solutions:**

1. Compare `api_key` in both `config/server.toml` and `config/viewer.toml`
2. They must be identical
3. Check for the `WORKSHOP_MANAGER_API_KEY` environment variable override

**Linux — how to find the correct API key:**
```bash
# The server runs as workshop-server user, config is here:
sudo cat /var/lib/workshop-server/.local/share/WorkshopManager/config/server.toml | grep api_key

# Or search for it:
sudo find /var/lib/workshop-server -name "server.toml" 2>/dev/null
```

### Device Key Required

**Symptoms:**
```
403 Forbidden: Device key required
```

**Solutions:**

1. If `require_device_key = true` in `server.toml`, configure a device key in `viewer.toml`
2. Or set `require_device_key = false` in `server.toml`
3. Generate a device key via the admin UI and assign it to the workstation

### Server Not Reachable

**Symptoms:** Viewer shows "Connection refused" or timeout.

**Solutions:**

1. Verify the server is running
2. Check the `base_url` in `viewer.toml` matches the server address
3. Verify the port is correct (default: 8443)
4. Check firewall rules if connecting over LAN

**Linux — open firewall port:**
```bash
sudo ufw allow 8443/tcp
sudo ufw status
```

### Connection Timeout (API endpoints hang)

**Symptoms:** `/health` responds OK but API endpoints like `/api/auth/setup-status` timeout.

**Solutions:**

1. Check if PostgreSQL is running:
```bash
ps aux | grep postgres | grep workshop
ss -tlnp | grep 46679
```

2. Restart the server (this resets the DB connection pool):
```bash
sudo systemctl restart workshop-server
```

3. If the issue persists, check server logs:
```bash
sudo journalctl -u workshop-server -n 100 --no-pager
```

### Viewer Can't Connect from Another Machine

**Symptoms:** Server works locally but viewer on another machine times out.

**Solutions:**

1. Verify server binds to `0.0.0.0` (not `127.0.0.1`) in `server.toml`:
```toml
[server]
host = "0.0.0.0"
```

2. Open the firewall port on the server:
```bash
sudo ufw allow 8443/tcp
```

3. Verify TLS cert includes the server's LAN IP (auto-detected by default)

## Authentication Errors

### Expired JWT Token

**Symptoms:**
```
401 Unauthorized: Token expired
```

**Solution:** Log out and log in again. JWT tokens expire after the configured duration.

### Wrong Credentials

**Symptoms:**
```
401 Unauthorized: Invalid credentials
```

**Solutions:**

1. Verify email and password
2. Check if the user account is active (not deactivated)
3. Check for typos or extra whitespace

### Invalid Token

**Symptoms:**
```
401 Unauthorized: Invalid token
```

**Solutions:**

1. The `.jwt_secret` may have been regenerated - log in again
2. Clear the stored token in the viewer (log out and back in)

## Database Errors

### Migration Failure

**Symptoms:**
```
ERROR Failed to run migrations: ...
```

**Solutions:**

1. Check the server logs for the specific migration error
2. Ensure the embedded PostgreSQL is running (check for `postgres` processes)
3. Try deleting the data directory and starting fresh (WARNING: loses all data)

### Connection Pool Exhausted

**Symptoms:**
```
ERROR Pool timed out
```

**Solutions:**

1. Restart the server
2. Check for long-running queries (server logs)
3. Reduce concurrent API requests

### Database Corrupted

**Symptoms:** Various database errors, data inconsistencies.

**Solutions:**

1. Restore from backup (see [backup-recovery.md](backup-recovery.md))
2. If no backup available, delete the data directory and restart (loses all data)

## Performance Issues

### Slow Queries

**Symptoms:** Application feels sluggish, API responses are slow.

**Solutions:**

1. Check server logs for slow query warnings
2. Ensure the data directory is on an SSD
3. Restart the server to reset connection pools

### High Memory Usage

**Solutions:**

1. Restart the server periodically
2. Check for connection leaks in logs
3. Reduce the number of concurrent connections

### Large Database

**Solutions:**

1. Archive old data (sales, repairs) periodically
2. Run `VACUUM ANALYZE` on the database
3. Ensure sufficient disk space

## Log Files Location

### Windows

Logs are written to stdout and to `%LOCALAPPDATA%\WorkshopManager\data\logs\server.log`.

### Linux (systemd service)

Logs are captured by journald and also written to the data directory:

```bash
# Ver últimas 100 líneas de log
sudo journalctl -u workshop-server -n 100 --no-pager

# Logs en tiempo real
sudo journalctl -u workshop-server -f

# Buscar errores
sudo journalctl -u workshop-server | grep -i error

# Ver log del archivo
sudo cat /var/lib/workshop-server/.local/share/WorkshopManager/data/logs/server.log | tail -50
```

### Linux (manual execution)

```bash
./workshop-server 2>&1 | tee server.log
```

### Log Levels

The server uses `tracing_subscriber` with info level. Key log events:

| Event | Level | Description |
|-------|-------|-------------|
| Server startup | INFO | Config loaded, secrets initialized, DB ready |
| Backup | INFO/ERROR | Backup creation and pruning |
| Auth | WARN | Failed login attempts |
| API | ERROR | Request handling errors |
| DB | ERROR | Connection and query errors |

### Windows System Tray

On Windows, the server runs a system tray icon. Right-click for options. The server logs to the terminal window.

## Linux Service Management

### systemd Commands

```bash
# Estado del servicio
sudo systemctl status workshop-server

# Iniciar / detener / reiniciar
sudo systemctl start workshop-server
sudo systemctl stop workshop-server
sudo systemctl restart workshop-server

# Habilitar inicio automático
sudo systemctl enable workshop-server

# Ver logs
sudo journalctl -u workshop-server -f
```

### Data Directory Structure (Linux)

```
/var/lib/workshop-server/.local/share/WorkshopManager/data/
├── config/server.toml          # Configuración del server
├── pgdata/                     # Datos de PostgreSQL embebido
├── postgresql/                 # Binarios de PostgreSQL
├── backups/                    # Backups automáticos
├── logs/                       # Archivos de log
├── .crypto_key                 # Clave de cifrado AES-256
├── .jwt_secret                 # Secreto para tokens JWT
├── .postgres_password          # Password de PostgreSQL
├── server.crt                  # Certificado TLS autofirmado
└── server.key                  # Clave privada TLS
```

### .deb Package Installation

```bash
# Instalar
sudo dpkg -i workshop-server_*.deb

# Si hay dependencias faltantes
sudo apt-get install -f

# Verificar instalación
dpkg -L workshop-server

# Verificar que el service está habilitado
systemctl is-enabled workshop-server
```

## Getting Help

If the issue is not covered here:

1. Check the server terminal output for error messages
2. Review the [server.toml](configuration.md) configuration
3. Check file permissions on the data directory
4. Ensure all prerequisites are installed (see [prerequisites.md](prerequisites.md))
