# Comandos Importantes — WorkshopManager

## Compilación y desarrollo

```powershell
# Compilar todo el workspace
cargo build

# Compilar en release
cargo build --release

# Compilar solo un crate
cargo build -p workshop-server
cargo build -p workshop-viewer

# Verificar sin generar binarios (más rápido)
cargo check -p workshop-viewer

# Ejecutar server (con terminal visible en debug)
cargo run -p workshop-server

# Ejecutar viewer
cargo run -p workshop-viewer
```

## MSI / Instaladores

```powershell
# Server MSI (requiere WiX Toolset v3)
cargo wix -p workshop-server --nocapture --install-version 0.1.0
cargo wix -p workshop-server --nocapture -o "target/wix/WorkshopManagerServer-v0.1.0.msi"

# Viewer MSI
cargo wix -p workshop-viewer --nocapture
cargo wix -p workshop-viewer --nocapture -o "target/wix/WorkshopManagerViewer-v0.1.0.msi"

# Limpiar artefactos WiX anteriores
Remove-Item -Recurse -Force target\wix -ErrorAction SilentlyContinue
```

## Despliegue Linux (servidor)

El server se distribuye como `.deb` (amd64/arm64) y corre como servicio
systemd con el usuario `workshop-server`:

```bash
# Instalar el .deb (el postinst crea el usuario y arranca el servicio)
sudo dpkg -i workshop-server_0.1.0-1_amd64.deb

# Si hay dependencias faltantes
sudo apt-get install -f

# Estado y logs
systemctl status workshop-server
journalctl -u workshop-server -f
sudo ufw allow 8443/tcp

# Rutas del server Linux
/var/lib/workshop-server/.local/share/WorkshopManager/data/   # DB, secretos, cert TLS
/var/lib/workshop-server/.local/share/WorkshopManager/config/server.toml  # Config con API key

# Obtener la API key (para emparejar con el viewer)
sudo cat /var/lib/workshop-server/.local/share/WorkshopManager/config/server.toml | grep api_key

# psql contra la DB embebida
sudo -u workshop-server /var/lib/workshop-server/.local/share/WorkshopManager/data/postgresql/18.3.0/bin/psql \
  -h 127.0.0.1 -p 46679 -U postgres -d workshop_manager

# Diagnóstico "Exec format error" en arm64 (PG embebido con arquitectura equivocada)
file /var/lib/workshop-server/.local/share/WorkshopManager/data/postgresql/18.3.0/bin/postgres

# Remover el paquete (conserva los datos)
sudo dpkg -r workshop-server
# Limpieza total: sudo userdel -r workshop-server && sudo rm -rf /var/lib/workshop-server
```

## Build de release multi-plataforma

```powershell
# MSIs (Windows, requiere WiX Toolset v3)
cargo build --release -p workshop-server -p workshop-viewer
cargo wix -p workshop-server --nocapture
cargo wix -p workshop-viewer --nocapture

# .deb Linux (requiere cargo-deb: cargo install cargo-deb --locked)
# Nota: cargo-deb espera target/release/workshop-server sin extensión .exe
Copy-Item target\release\workshop-server.exe target\release\workshop-server
cargo deb -p workshop-server

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

# Ruta equivalente en el server Linux (servicio workshop-server)
/var/lib/workshop-server/.theseus/postgresql/18.3.0/bin/

# Ruta de datos en desarrollo (relativa al dir de trabajo)
crates\workshop-server\data\pgdata\

# Ruta de datos en producción (MSI)
%LOCALAPPDATA%\WorkshopManager\data\pgdata\

# Conectarse a la base (mientras el server corre)
& "C:\Users\carlos\.theseus\postgresql\18.3.0\bin\psql.exe" -h localhost -U postgres -d workshop_manager

# Limpiar datos corruptos (detener server primero)
Remove-Item -Recurse -Force "%LOCALAPPDATA%\WorkshopManager\data\pgdata"
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
Get-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "WorkshopManagerServer"

# Eliminar manualmente
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "WorkshopManagerServer"

# Agregar manualmente
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "WorkshopManagerServer" -Value "C:\Program Files\WorkshopManager\bin\workshop-server.exe"
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
C:\Users\carlos\Desktop\workshop-manager\

# Datos de PostgreSQL (desarrollo)
C:\Users\carlos\Desktop\workshop-manager\crates\workshop-server\data\pgdata\

# Datos de PostgreSQL (producción)
%LOCALAPPDATA%\WorkshopManager\data\pgdata\

# Llave de cifrado AES-256-GCM (auto-generada, no compartir)
%LOCALAPPDATA%\WorkshopManager\data\.crypto_key

# Configuración del viewer
%LOCALAPPDATA%\WorkshopManager\viewer_storage\

# Assets del server
C:\Users\carlos\Desktop\workshop-manager\crates\workshop-server\assets\icon.png
C:\Users\carlos\Desktop\workshop-manager\crates\workshop-server\assets\icon.ico

# CSS compilado del viewer
C:\Users\carlos\Desktop\workshop-manager\crates\workshop-viewer\index.css

# MSIs generados
C:\Users\carlos\Desktop\workshop-manager\target\wix\

# PostgreSQL embebido (binarios descargados por cargo)
C:\Users\carlos\.theseus\postgresql\18.3.0\bin\
```

## Variables de entorno

```powershell
# Configurar secreto JWT
$env:JWT_SECRET="mi-secreto-personalizado"

# URL de base de datos (opcional, el server usa la embebida por defecto)
$env:DATABASE_URL="postgres://postgres:postgres@localhost:5432/workshop_manager"
```

## Solución de problemas

```powershell
# Error "Permission denied" al iniciar PostgreSQL en MSI
# -> Migrar datos de %ProgramFiles% a %LOCALAPPDATA%:
Move-Item "$env:ProgramFiles\WorkshopManager\bin\data\pgdata" "$env:LOCALAPPDATA\WorkshopManager\data\pgdata"

# Error "postmaster.pid" archivo stale
# -> Detener server, borrar el archivo:
Remove-Item "$env:LOCALAPPDATA\WorkshopManager\data\pgdata\postmaster.pid" -ErrorAction SilentlyContinue

# El server no arranca por puerto ocupado
netstat -ano | findstr :8443
# PID del proceso que ocupa el puerto
taskkill /PID <PID> /F

# Viewer no se conecta al server
# -> Verificar que el server esté corriendo
# -> Revisar la IP configurada en el engranaje de Login
# -> Firewall: permitir puerto 3000
New-NetFirewallRule -DisplayName "WorkshopManager Server" -Direction Inbound -Protocol TCP -LocalPort 8443 -Action Allow

# Reconstruir el viewer con CSS actualizado
# (el CSS está inlinado en el binario vía include_str!)
cargo build -p workshop-viewer
```

## Dependencias del sistema

- **WiX Toolset v3** — para `cargo wix`
  - Instalar desde: https://wixtoolset.org/docs/wix3/
  - Verificar: `candle.exe -?`
- **WebView2 Runtime** — para el viewer (Dioxus Desktop)
  - Incluido en Windows 11 / Windows 10 (actualizaciones recientes)
  - Descargar: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
