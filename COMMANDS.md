# Comandos Importantes — Equipos-Rust

## Compilación y desarrollo

```powershell
# Compilar todo el workspace
cargo build

# Compilar en release
cargo build --release

# Compilar solo un crate
cargo build -p server
cargo build -p viewer

# Verificar sin generar binarios (más rápido)
cargo check -p server

# Ejecutar server (con terminal visible en debug)
cargo run -p server

# Ejecutar viewer
cargo run -p viewer
```

## MSI / Instaladores

```powershell
# Server MSI (requiere WiX Toolset v3)
cargo wix -p server --nocapture --install-version 0.3.0
cargo wix -p server --nocapture -o "target/wix/EquiposServer-v0.3.0.msi"

# Viewer MSI
cargo wix -p viewer --nocapture
cargo wix -p viewer --nocapture -o "target/wix/EquiposViewer-v0.3.0.msi"

# Limpiar artefactos WiX anteriores
Remove-Item -Recurse -Force target\wix -ErrorAction SilentlyContinue
```

## Despliegue Linux (servidor)

El server se distribuye como `.deb` (amd64/arm64) y corre como servicio
systemd con el usuario `equipos`:

```bash
# Instalar el .deb (el postinst crea el usuario y arranca el servicio)
sudo dpkg -i server_0.3.0-1_amd64.deb

# Estado y logs
systemctl status equipos-server
journalctl -u equipos-server -f          # esperar "server ready at https://0.0.0.0:3000"
sudo ufw allow 3000                       # abrir puerto si ufw está activo

# Rutas del server Linux
/var/lib/equipos-server/.local/share/EquiposIndustriales/data/   # DB, secretos, cert TLS
/var/lib/equipos-server/.theseus/postgresql/18.3.0/bin/          # binarios PostgreSQL

# psql contra la DB embebida (user/password en .superuser_credentials.json)
cat /var/lib/equipos-server/.local/share/EquiposIndustriales/data/.superuser_credentials.json
PGPASSWORD='<password>' /var/lib/equipos-server/.theseus/postgresql/18.3.0/bin/psql \
  -h 127.0.0.1 -U <user> -d equipos_redes

# Registrar una Device Key en headless (no hay tray icon en Linux)
KEY=$(tr -dc 'A-Z0-9' < /dev/urandom | head -c 32)   # 32 chars A-Z0-9
HASH=$(printf '%s' "$KEY" | sha256sum | awk '{print $1}')
PGPASSWORD='<password>' /var/lib/equipos-server/.theseus/postgresql/18.3.0/bin/psql \
  -h 127.0.0.1 -U <user> -d equipos_redes \
  -c "INSERT INTO device_keys (key_hash, device_name) VALUES ('$HASH', 'carlos-pc');"

# Validar una key sin abrir el viewer (v0.3.1+): 200 {"valid":true} = OK
curl -k -H "X-Device-Key: $KEY" https://<ip>:3000/api/device-key/check

# Diagnóstico "Exec format error" en arm64 (PG embebido con arquitectura
# equivocada — incidente v0.3.0): el binario debe decir "ARM aarch64" en la Pi.
file /var/lib/equipos-server/.theseus/postgresql/18.3.0/bin/postgres

# Remover el paquete (conserva los datos)
sudo dpkg -r server
# Limpieza total: sudo userdel -r equipos && sudo rm -rf /var/lib/equipos-server
```

## Build de release multi-plataforma

```powershell
# MSIs (Windows, requiere WiX Toolset v3)
cargo build --release -p server -p viewer
cargo wix -p server --nocapture
cargo wix -p viewer --nocapture

# .deb Linux (requiere cargo-deb: cargo install cargo-deb --locked)
# Nota: cargo-deb espera target/release/server sin extensión .exe
Copy-Item target\release\server.exe target\release\server
cargo deb -p server

# Versionado del workspace (bump + commit + tag v<version>)
.\scripts\bump.ps1 0.4.0
```

> Los workflows de CI replican estos pasos: `ci.yml` (fmt, clippy, build,
> test, audit en cada push) y `release.yml` (al pushear un tag `v*` publica
> los 6 artefactos: 2 MSIs, 2 .deb, 2 binarios raw).

## PostgreSQL embebido (solo server)

El server gestiona PostgreSQL automáticamente. Para depuración manual:

```powershell
# Ruta de los binarios de PostgreSQL usados por el MSI (Windows)
C:\Users\carlos\.theseus\postgresql\18.3.0\bin\

# Ruta equivalente en el server Linux (servicio equipos-server)
/var/lib/equipos-server/.theseus/postgresql/18.3.0/bin/

# Ruta de datos en desarrollo (relativa al dir de trabajo)
crates\server\data\pgdata\

# Ruta de datos en producción (MSI)
%LOCALAPPDATA%\EquiposIndustriales\data\pgdata\

# Conectarse a la base (mientras el server corre)
& "C:\Users\carlos\.theseus\postgresql\18.3.0\bin\psql.exe" -h localhost -U postgres -d equipos_redes

# Verificar que las credenciales están cifradas en DB
& "C:\Users\carlos\.theseus\postgresql\18.3.0\bin\psql.exe" -h localhost -U postgres -d equipos_redes -c "SELECT id, ip_address, LEFT(clave_windows, 40) AS clave_hex FROM equipos WHERE clave_windows IS NOT NULL;"
# Si está cifrado, clave_hex se ve como: 1a2b3c4d... (hexadecimal)

# Limpiar datos corruptos (detener server primero)
Remove-Item -Recurse -Force "%LOCALAPPDATA%\EquiposIndustriales\data\pgdata"
```

## Escaneo de red (ICMP)

El server requiere permisos de administrador para ICMP en algunas
configuraciones de Windows:

```powershell
# Habilitar ICMP en firewall (si hay bloqueos)
New-NetFirewallRule -DisplayName "ICMP Allow" -Protocol ICMPv4 -IcmpType 8 -Enabled True
```

## Autostart (registro de Windows)

El server gestiona esta entrada desde el menú contextual del tray icon
(vía `winreg`). Al hacer clic en "Autostart" se escribe o elimina la
entrada en `HKCU\...\Run` y se marca/desmarca visualmente el ✓.

```powershell
# Verificar entrada actual
Get-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "EquiposIndustrialesServer"

# Eliminar manualmente
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "EquiposIndustrialesServer"

# Agregar manualmente
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "EquiposIndustrialesServer" -Value "C:\Program Files\EquiposIndustriales\bin\server.exe"
```

## Pruebas de seguridad

```powershell
# Ejecutar suite completa de seguridad (requiere server corriendo)
PowerShell -ExecutionPolicy Bypass -File test_seguridad.ps1

# Tests incluidos:
# 1. Health check
# 2. Login como admin
# 3. POST /register bloqueado (403 si ya hay usuarios)
# 4. GET /api/users sin auth (401)
# 5. Login como viewer
# 6. GET /api/users como viewer (403)
# 7. Crear equipo con credenciales (API devuelve descifrado)
# 8. DB almacena cifrado (AES-256-GCM)
# 9. Auditoría redacta credenciales ([CIFRADO])
```

## Directorios importantes

```
# Código fuente
C:\Users\carlos\Desktop\Equipos-Rust\

# Datos de PostgreSQL (desarrollo)
C:\Users\carlos\Desktop\Equipos-Rust\crates\server\data\pgdata\

# Datos de PostgreSQL (producción)
%LOCALAPPDATA%\EquiposIndustriales\data\pgdata\

# Llave de cifrado AES-256-GCM (auto-generada, no compartir)
%LOCALAPPDATA%\EquiposIndustriales\data\.crypto_key

# Configuración del viewer
%LOCALAPPDATA%\EquiposIndustriales\viewer_storage\

# Assets del server
C:\Users\carlos\Desktop\Equipos-Rust\crates\server\assets\icon.png
C:\Users\carlos\Desktop\Equipos-Rust\crates\server\assets\icon.ico

# CSS compilado del viewer
C:\Users\carlos\Desktop\Equipos-Rust\crates\viewer\index.css

# MSIs generados
C:\Users\carlos\Desktop\Equipos-Rust\target\wix\

# PostgreSQL embebido (binarios descargados por cargo)
C:\Users\carlos\.theseus\postgresql\18.3.0\bin\
```

## Variables de entorno

```powershell
# Configurar secreto JWT
$env:JWT_SECRET="mi-secreto-personalizado"

# URL de base de datos (opcional, el server usa la embebida por defecto)
$env:DATABASE_URL="postgres://postgres:postgres@localhost:5432/equipos_redes"
```

## Solución de problemas

```powershell
# Error "Permission denied" al iniciar PostgreSQL en MSI
# -> Migrar datos de %ProgramFiles% a %LOCALAPPDATA%:
Move-Item "$env:ProgramFiles\EquiposIndustriales\bin\data\pgdata" "$env:LOCALAPPDATA\EquiposIndustriales\data\pgdata"

# Error "postmaster.pid" archivo stale
# -> Detener server, borrar el archivo:
Remove-Item "$env:LOCALAPPDATA\EquiposIndustriales\data\pgdata\postmaster.pid" -ErrorAction SilentlyContinue

# El server no arranca por puerto ocupado
netstat -ano | findstr :3000
# PID del proceso que ocupa el puerto
taskkill /PID <PID> /F

# Viewer no se conecta al server
# -> Verificar que el server esté corriendo
# -> Revisar la IP configurada en el engranaje de Login
# -> Firewall: permitir puerto 3000
New-NetFirewallRule -DisplayName "Equipos Server" -Direction Inbound -Protocol TCP -LocalPort 3000 -Action Allow

# Reconstruir el viewer con CSS actualizado
# (el CSS está inlinado en el binario vía include_str!)
cargo build -p viewer
```

## Dependencias del sistema

- **WiX Toolset v3** — para `cargo wix`
  - Instalar desde: https://wixtoolset.org/docs/wix3/
  - Verificar: `candle.exe -?`
- **WebView2 Runtime** — para el viewer (Dioxus Desktop)
  - Incluido en Windows 11 / Windows 10 (actualizaciones recientes)
  - Descargar: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
