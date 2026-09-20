# Plan de Implementacion: License Service con Cloudflare Workers + D1

**Fecha:** 2026-09-19
**Estado:** Planificacion
**Objetivo:** Sistema anti-pirateria completo basado en la especificacion tecnica del producto

---

## 1. Arquitectura final

```
                    INTERNET
                       |
            +----------v----------+
            |     CLOUDFLARE      |
            |                     |
            |  Workers            |
            |  +---------------+  |
            |  | License API   |  |
            |  | Admin Panel   |  |
            |  +-------+-------+  |
            |          |          |
            |       D1 DB        |
            |   (licencias)      |
            +----------+----------+
                       |
              SOLO EN ACTIVACION
                       |
===============================================
                       |
                 CLIENTE/TALLER
                       |
            +----------v----------+
            |       SERVER        |
            |                     |
            |  Valida key ->      |
            |  Cloudflare ->      |
            |  Firma local ->     |
            |  Guarda local ->    |
            |  Funciona offline   |
            +----------+----------+
                       |
                  LAN / TAILSCALE
                       |
          +------------+------------+
          |            |            |
          v            v            v
       Viewer 1    Viewer 2    Viewer (Tailscale)
```

### Principios clave

- **Offline-first:** Despues de activar, el server funciona sin internet
- **Secret key local:** La clave privada Ed25519 NUNCA sale del PC del cliente
- **Firma local:** El server firma la licencia con `sign_license()`
- **Validacion online:** Solo al activar o transferir

---

## 2. Decisiones de diseno

| Decision | Valor | Razon |
|----------|-------|-------|
| Firma de licencia | **Server** (secret_key local) | La clave privada nunca sale del PC |
| Input de key | **Tray icon** del server | UX simple, sin UI adicional |
| Trial sin internet | **Automatico** (7 dias) | No bloquear al cliente |
| Backend | **Cloudflare Workers + D1** | Gratis, serverless, sin mantenimiento |
| Admin | **Panel HTML/JS** en el Worker | Accesible desde cualquier navegador |
| Hardware fingerprint | **CPU + Motherboard + Disk** | Compuesto, SHA-256 |
| Tolerancia a cambios | **No implementar aun** | Fase futura |

---

## 3. Flujo de activacion

### 3.1 Primera instalacion (con internet)

```
1. Cliente instala Server
2. Server arranca -> no hay license.dat
3. Server entra en modo "FirstRun"
4. Tray icon muestra: "Licencia no encontrada. Ingrese su key."
5. Cliente escribe la key en el tray icon
6. Server envia a Cloudflare:
   POST /api/v1/licenses/activate
   { "license_key": "TALLER-001", "hardware_hash": "a3f8b2c1..." }

7. Cloudflare D1 verifica:
   - Key existe? -> SI
   - Esta activa? -> SI
   - hw_hash coincide? -> SI (primera vez)
   - Actualiza: hardware_hash, activated_at

8. Cloudflare retorna campos de la licencia (sin firma)
9. Server firma con sign_license() usando secret_key local
10. Server guarda license.dat localmente
11. Server funciona (offline despues)
```

### 3.2 Primera instalacion (sin internet)

```
1. Cliente instala Server
2. Server arranca -> no hay license.dat
3. Server intenta validar online -> falla (sin internet)
4. Server crea trial automatico (7 dias)
5. Tray icon muestra: "Modo trial. Expira en 7 dias."
6. Cuando tenga internet -> cliente ingresa key -> se activa
```

### 3.3 Reinstalacion en otra PC

```
1. Cliente instala Server en PC nueva
2. Server arranca -> no hay license.dat
3. Tray icon pide key
4. Cliente escribe la misma key: "TALLER-001"
5. Server envia a Cloudflare:
   POST /api/v1/licenses/activate
   { "license_key": "TALLER-001", "hardware_hash": "nuevo_hash..." }

6. Cloudflare D1 verifica:
   - Key existe? -> SI
   - hw_hash coincide? -> NO (es diferente)
   - Tiene transferencias disponibles? -> SI
   - Revoke old, activate new
   - transfer_count + 1

7. Cloudflare retorna nueva licencia
8. Server firma y guarda license.dat
9. Funciona en PC nueva
10. PC vieja pierde licencia (hw_hash ya no coincide)
```

### 3.4 Viewer se conecta

```
1. Viewer abre -> conecta al Server
2. Server verifica:
   - Licencia local valida? -> SI
   - Sesiones activas < max_concurrent_viewers? -> SI
   - Viewer autenticado? -> SI
3. Viewer funciona
4. Si limite alcanzado -> 429 Too Many Requests
```

---

## 4. Cloudflare Workers + D1

### 4.1 Estructura del proyecto

```
cloudflare-license-service/
+-- src/
|   +-- index.ts              # Worker principal
|   +-- routes/
|   |   +-- activate.ts       # POST /api/v1/licenses/activate
|   |   +-- validate.ts       # POST /api/v1/licenses/validate
|   |   +-- transfer.ts       # POST /api/v1/licenses/transfer
|   |   +-- admin.ts          # CRUD licencias (protegido)
|   +-- lib/
|   |   +-- db.ts             # D1 queries
|   |   +-- auth.ts           # Admin auth
|   +-- types.ts              # TypeScript types
+-- admin/
|   +-- index.html            # Panel admin (SPA)
|   +-- app.js                # Logica admin
|   +-- style.css
+-- migrations/
|   +-- 0001_licenses.sql
+-- wrangler.toml
+-- package.json
+-- tsconfig.json
```

### 4.2 D1 Schema

```sql
-- migrations/0001_licenses.sql

CREATE TABLE licenses (
    id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))),
    license_key TEXT NOT NULL UNIQUE,
    product TEXT NOT NULL DEFAULT 'workshop',
    tier TEXT NOT NULL DEFAULT 'base',
    hardware_hash TEXT,
    active INTEGER NOT NULL DEFAULT 1,
    activated_at TEXT,
    max_concurrent_viewers INTEGER NOT NULL DEFAULT 2,
    remote_access INTEGER NOT NULL DEFAULT 0,
    advanced_reports INTEGER NOT NULL DEFAULT 0,
    backup INTEGER NOT NULL DEFAULT 0,
    max_transfers INTEGER NOT NULL DEFAULT 3,
    transfer_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX idx_licenses_key ON licenses(license_key);
CREATE INDEX idx_licenses_hw ON licenses(hardware_hash);
CREATE INDEX idx_licenses_active ON licenses(active);

-- Rate limiting
CREATE TABLE activation_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_key TEXT NOT NULL,
    hardware_hash TEXT NOT NULL,
    ip_address TEXT,
    success INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_attempts_key ON activation_attempts(license_key, created_at);
```

### 4.3 Worker principal

```typescript
// src/index.ts

export interface Env {
    DB: D1Database;
    ADMIN_TOKEN: string;
}

export default {
    async fetch(request: Request, env: Env): Promise<Response> {
        const url = new URL(request.url);

        const corsHeaders = {
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
            'Access-Control-Allow-Headers': 'Content-Type, Authorization',
        };

        if (request.method === 'OPTIONS') {
            return new Response(null, { headers: corsHeaders });
        }

        // Public endpoints
        if (url.pathname === '/api/v1/licenses/activate' && request.method === 'POST') {
            return handleActivate(request, env, corsHeaders);
        }
        if (url.pathname === '/api/v1/licenses/validate' && request.method === 'POST') {
            return handleValidate(request, env, corsHeaders);
        }
        if (url.pathname === '/api/v1/licenses/transfer' && request.method === 'POST') {
            return handleTransfer(request, env, corsHeaders);
        }

        // Admin endpoints (protected)
        if (url.pathname.startsWith('/api/v1/admin/')) {
            return handleAdmin(request, env, corsHeaders);
        }

        return new Response('Not Found', { status: 404, headers: corsHeaders });
    }
};
```

### 4.4 Activacion

```typescript
// src/routes/activate.ts

interface ActivateRequest {
    license_key: string;
    hardware_hash: string;
}

export async function handleActivate(
    request: Request,
    env: Env,
    corsHeaders: Record<string, string>
): Promise<Response> {
    const body: ActivateRequest = await request.json();

    // 1. Rate limit check (5 intentos por hora)
    const attempts = await env.DB.prepare(
        `SELECT COUNT(*) as count FROM activation_attempts
         WHERE license_key = ? AND created_at > datetime('now', '-1 hour')`
    ).bind(body.license_key).first();

    if (attempts.count >= 5) {
        return new Response(JSON.stringify({ error: 'Too many attempts' }), {
            status: 429,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }

    // 2. Look up license
    const license = await env.DB.prepare(
        `SELECT * FROM licenses WHERE license_key = ?`
    ).bind(body.license_key).first();

    if (!license) {
        await logAttempt(env, body, false, 'Key not found');
        return new Response(JSON.stringify({ error: 'Invalid license key' }), {
            status: 404,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }

    // 3. Check if active
    if (!license.active) {
        await logAttempt(env, body, false, 'License revoked');
        return new Response(JSON.stringify({ error: 'License revoked' }), {
            status: 403,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }

    // 4. Check if already activated on different hardware
    if (license.hardware_hash && license.hardware_hash !== body.hardware_hash) {
        // Different hardware - this is a transfer
        if (license.transfer_count >= license.max_transfers) {
            await logAttempt(env, body, false, 'Max transfers reached');
            return new Response(JSON.stringify({ error: 'Max transfers reached' }), {
                status: 403,
                headers: { ...corsHeaders, 'Content-Type': 'application/json' }
            });
        }

        // Revoke old, activate new
        await env.DB.prepare(
            `UPDATE licenses SET
                hardware_hash = ?,
                transfer_count = transfer_count + 1,
                activated_at = datetime('now'),
                updated_at = datetime('now')
             WHERE license_key = ?`
        ).bind(body.hardware_hash, body.license_key).run();
    } else {
        // First activation or same hardware
        await env.DB.prepare(
            `UPDATE licenses SET
                hardware_hash = ?,
                active = 1,
                activated_at = datetime('now'),
                updated_at = datetime('now')
             WHERE license_key = ?`
        ).bind(body.hardware_hash, body.license_key).run();
    }

    // 5. Log success
    await logAttempt(env, body, true, null);

    // 6. Return license data (server will sign locally with vendor key)
    return new Response(JSON.stringify({
        success: true,
        license: {
            license_key: license.license_key,
            tier: license.tier,
            hardware_hash: body.hardware_hash,
            max_concurrent_viewers: license.max_concurrent_viewers,
            remote_access: !!license.remote_access,
            advanced_reports: !!license.advanced_reports,
            backup: !!license.backup,
            max_transfers: license.max_transfers,
            transfer_count: Number(license.transfer_count) + 1,
            activated_at: new Date().toISOString(),
        }
    }), {
        headers: { ...corsHeaders, 'Content-Type': 'application/json' }
    });
}

async function logAttempt(env: Env, body: ActivateRequest, success: boolean, error: string | null) {
    await env.DB.prepare(
        `INSERT INTO activation_attempts (license_key, hardware_hash, success, error_message)
         VALUES (?, ?, ?, ?)`
    ).bind(body.license_key, body.hardware_hash, success ? 1 : 0, error).run();
}
```

### 4.5 Validacion

```typescript
// src/routes/validate.ts

export async function handleValidate(
    request: Request,
    env: Env,
    corsHeaders: Record<string, string>
): Promise<Response> {
    const body = await request.json();

    const license = await env.DB.prepare(
        `SELECT * FROM licenses WHERE license_key = ? AND active = 1`
    ).bind(body.license_key).first();

    if (!license) {
        return new Response(JSON.stringify({ valid: false }), {
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }

    const hwMatch = license.hardware_hash === body.hardware_hash;

    return new Response(JSON.stringify({
        valid: hwMatch,
        license: hwMatch ? {
            license_key: license.license_key,
            tier: license.tier,
            hardware_hash: license.hardware_hash,
            max_concurrent_viewers: license.max_concurrent_viewers,
            remote_access: !!license.remote_access,
            advanced_reports: !!license.advanced_reports,
            backup: !!license.backup,
            max_transfers: license.max_transfers,
            transfer_count: license.transfer_count,
            activated_at: license.activated_at,
        } : null
    }), {
        headers: { ...corsHeaders, 'Content-Type': 'application/json' }
    });
}
```

### 4.6 Admin API

```typescript
// src/routes/admin.ts

export async function handleAdmin(
    request: Request,
    env: Env,
    corsHeaders: Record<string, string>
): Promise<Response> {
    const url = new URL(request.url);

    // Verify admin token
    const authHeader = request.headers.get('Authorization');
    if (authHeader !== `Bearer ${env.ADMIN_TOKEN}`) {
        return new Response(JSON.stringify({ error: 'Unauthorized' }), {
            status: 401,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }

    // GET /api/v1/admin/licenses - List all
    if (url.pathname === '/api/v1/admin/licenses' && request.method === 'GET') {
        const licenses = await env.DB.prepare(
            `SELECT * FROM licenses ORDER BY created_at DESC`
        ).all();
        return new Response(JSON.stringify(licenses), {
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }

    // POST /api/v1/admin/licenses - Create new
    if (url.pathname === '/api/v1/admin/licenses' && request.method === 'POST') {
        const body = await request.json();
        const key = generateLicenseKey();

        await env.DB.prepare(
            `INSERT INTO licenses (license_key, tier, max_concurrent_viewers, remote_access, advanced_reports, backup, max_transfers)
             VALUES (?, ?, ?, ?, ?, ?, ?)`
        ).bind(
            key,
            body.tier || 'base',
            body.max_concurrent_viewers || 2,
            body.remote_access ? 1 : 0,
            body.advanced_reports ? 1 : 0,
            body.backup ? 1 : 0,
            body.max_transfers || 3
        ).run();

        return new Response(JSON.stringify({ license_key: key }), {
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }

    // DELETE /api/v1/admin/licenses/:key - Revoke
    if (url.pathname.startsWith('/api/v1/admin/licenses/') && request.method === 'DELETE') {
        const key = url.pathname.split('/').pop();
        await env.DB.prepare(
            `UPDATE licenses SET active = 0, updated_at = datetime('now') WHERE license_key = ?`
        ).bind(key).run();
        return new Response(JSON.stringify({ success: true }), {
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }

    return new Response('Not Found', { status: 404, headers: corsHeaders });
}

function generateLicenseKey(): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    const segments = [8, 8, 8, 8];
    return segments.map(len =>
        Array.from({ length: len }, () => chars[Math.floor(Math.random() * chars.length)]).join('')
    ).join('-');
}
```

### 4.7 wrangler.toml

```toml
name = "workshop-license-service"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[vars]
# ADMIN_TOKEN se configura via: wrangler secret put ADMIN_TOKEN

[[d1_databases]]
binding = "DB"
database_name = "workshop-licenses"
database_id = "TU-DATABASE-ID"
```

---

## 5. Cambios en Server (Rust)

### 5.1 Config

**config/server.toml** - agregar seccion license:

```toml
[license]
api_url = "https://tu-worker.workers.dev"
trial_days = 7
```

**crates/workshop-server/src/config.rs** - agregar campo license_api_url:

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub api_key: String,
    pub require_device_key: bool,
    pub max_viewers: u32,
    pub iva_rate: f64,
    pub icon_bg: [u8; 3],
    pub icon_fg: [u8; 3],
    pub cors_origins: Vec<String>,
    pub license_api_url: String,  // NUEVO
}
```

### 5.2 Keypair real

```powershell
cargo build --release -p license-tool
.\target\release\license-tool.exe generate-keypair --secret secret_key.bin --public public_key.bin
```

**crates/workshop-server/src/license.rs** - reemplazar placeholder:

```rust
// ANTES (placeholder):
const VENDOR_PUBLIC_KEY: &[u8] = &[0x00; 32];

// DESPUES (bytes reales de public_key.bin):
const VENDOR_PUBLIC_KEY: &[u8] = &[
    0xAA, 0xBB, 0xCC, 0xDD, // ... completar con bytes reales
];
```

### 5.3 Validate online

**crates/workshop-server/src/license.rs** - reescribir:

```rust
pub async fn validate_online(
    api_url: &str,
    license_key: &str,
    hardware_hash: &str,
) -> Option<License> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let body = serde_json::json!({
        "license_key": license_key,
        "hardware_hash": hardware_hash,
    });

    let resp = client
        .post(format!("{}/api/v1/licenses/activate", api_url))
        .json(&body)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let data: serde_json::Value = resp.json().await.ok()?;
    let license_data = data.get("license")?;

    // Parse license fields from Cloudflare response
    let license = License {
        license_key: license_data.get("license_key")?.as_str()?.to_string(),
        tier: serde_json::from_value(license_data.get("tier")?.clone()).ok()?,
        hardware_hash: license_data.get("hardware_hash")?.as_str()?.to_string(),
        max_concurrent_viewers: license_data.get("max_concurrent_viewers")?.as_u64()? as u32,
        remote_access: license_data.get("remote_access")?.as_bool()?,
        advanced_reports: license_data.get("advanced_reports")?.as_bool()?,
        backup: license_data.get("backup")?.as_bool()?,
        max_transfers: license_data.get("max_transfers")?.as_u64()? as u32,
        transfer_count: license_data.get("transfer_count")?.as_u64()? as u32,
        activated_at: chrono::DateTime::parse_from_rfc3339(
            license_data.get("activated_at")?.as_str()?
        ).ok()?.with_timezone(&chrono::Utc),
        expires_at: None,
    };

    Some(license)
}
```

### 5.4 FirstRun flow

**crates/workshop-server/src/main.rs** - modificar lineas 111-129:

```rust
LicenseStatus::FirstRun(hw_hash) => {
    tracing::info!("Primera instalacion. Hardware: {}", &hw_hash[..16]);

    // 1. Intentar validar online
    let api_url = &state.config.license_api_url;
    match license::validate_online(api_url, "trial", &hw_hash).await {
        Some(lic) => {
            // Online validation succeeded (trial from Cloudflare)
            let _ = license::save_license(&lic, &data_dir);
            Some(lic)
        }
        None => {
            // Offline - create local trial
            tracing::warn!("Sin conexion. Creando trial local por {} dias.",
                state.config.license_trial_days);
            Some(license::create_trial_license(&hw_hash, state.config.license_trial_days))
        }
    }
}
```

### 5.5 Input de key via tray icon

**crates/workshop-server/src/tray.rs** - agregar menu item:

```rust
// En el menu, agregar opcion "Ingresar licencia"
let enter_license_item = MenuItem::new("Ingresar licencia", true, None)?;

// En el match de eventos:
TrayEvent::MenuItem { id, .. } if id == "enter_license" => {
    // Abrir dialogo o input para key
    // Por simplicidad, usar un archivo temporal
    let key = read_license_key_from_user();
    if let Some(key) = key {
        // Validar online
        let hw = hardware::get_hardware_id().unwrap_or_default();
        match license::validate_online(&config.license_api_url, &key, &hw).await {
            Some(lic) => {
                let _ = license::save_license(&lic, &data_dir);
                // Reiniciar server
            }
            None => {
                // Error: key invalida o sin internet
            }
        }
    }
}
```

### 5.6 Expiracion de trial

**crates/workshop-common/src/features.rs** - agregar expires_at:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub license_key: String,
    pub tier: LicenseTier,
    pub hardware_hash: String,
    pub max_concurrent_viewers: u32,
    pub remote_access: bool,
    pub advanced_reports: bool,
    pub backup: bool,
    pub max_transfers: u32,
    pub transfer_count: u32,
    pub activated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,  // NUEVO
}
```

**crates/workshop-common/src/license.rs** - crear trial con expiracion:

```rust
pub fn create_trial_license(hardware_hash: &str, days: u32) -> License {
    License {
        license_key: "TRIAL".into(),
        tier: LicenseTier::Trial,
        hardware_hash: hardware_hash.into(),
        max_concurrent_viewers: 1,
        remote_access: false,
        advanced_reports: false,
        backup: false,
        max_transfers: 0,
        transfer_count: 0,
        activated_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::days(days as i64)),
    }
}
```

**crates/workshop-server/src/license.rs** - validar expiracion:

```rust
pub fn validate_license(license: &License) -> Result<(), String> {
    // Verificar expiracion
    if let Some(expires_at) = license.expires_at {
        if chrono::Utc::now() > expires_at {
            return Err("Licencia expirada".into());
        }
    }
    // ... resto de validaciones existentes
}
```

### 5.7 Reemplazar wmic (Win11 compat)

**crates/workshop-common/src/hardware.rs** - reemplazar wmic_value:

```rust
fn wmic_value(class: &str, field: &str) -> String {
    let output = Command::new("powershell")
        .args([
            "-NoProfile", "-NonInteractive", "-Command",
            &format!(
                "Get-CimInstance {} | Select-Object -ExpandProperty {}",
                class, field
            )
        ])
        .output()
        .ok();

    String::from_utf8_lossy(&output.map(|o| o.stdout).unwrap_or_default())
        .trim()
        .to_string()
}
```

### 5.8 Viewer sessions (control de Viewers simultaneos)

**crates/workshop-server/migrations/XX_create_viewer_sessions.sql:**

```sql
CREATE TABLE viewer_sessions (
    id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))),
    viewer_id TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    session_token TEXT NOT NULL UNIQUE,
    connected_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_heartbeat TEXT NOT NULL DEFAULT (datetime('now')),
    connection_type TEXT NOT NULL DEFAULT 'lan',
    client_version TEXT,
    ip_address TEXT,
    status TEXT NOT NULL DEFAULT 'active'
);

CREATE INDEX idx_viewer_sessions_active ON viewer_sessions(status, last_heartbeat);
CREATE INDEX idx_viewer_sessions_user ON viewer_sessions(user_id);
```

---

## 6. Cambios en License struct (workshop-common)

### 6.1 Features DLC

**crates/workshop-common/src/features.rs** - actualizar Feature enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    // Core
    Products,
    Sales,
    Repairs,
    Suppliers,
    Users,

    // Reportes
    Reports,
    AdvancedReports,

    // Seguridad
    Backup,

    // Acceso
    RemoteAccess,

    // Multi-sucursal
    MultiSite,

    // Integraciones
    Integrations,

    // Usuarios adicionales
    AdditionalUsers,
}
```

### 6.2 Capabilities por tier

```rust
impl LicenseTier {
    pub fn features(&self) -> Vec<Feature> {
        match self {
            LicenseTier::Trial => vec![
                Feature::Products,
                Feature::Sales,
            ],
            LicenseTier::Base => vec![
                Feature::Products,
                Feature::Sales,
                Feature::Repairs,
                Feature::Suppliers,
                Feature::Users,
            ],
            LicenseTier::Reports => vec![
                Feature::Products,
                Feature::Sales,
                Feature::Repairs,
                Feature::Suppliers,
                Feature::Users,
                Feature::Reports,
                Feature::AdvancedReports,
            ],
            LicenseTier::Advanced => vec![
                Feature::Products,
                Feature::Sales,
                Feature::Repairs,
                Feature::Suppliers,
                Feature::Users,
                Feature::Reports,
                Feature::AdvancedReports,
                Feature::Backup,
                Feature::RemoteAccess,
            ],
            LicenseTier::Api => vec![
                Feature::Products,
                Feature::Sales,
                Feature::Repairs,
                Feature::Suppliers,
                Feature::Users,
                Feature::Reports,
                Feature::AdvancedReports,
                Feature::Backup,
                Feature::RemoteAccess,
                Feature::MultiSite,
                Feature::Integrations,
                Feature::AdditionalUsers,
            ],
        }
    }

    pub fn max_concurrent_viewers(&self) -> u32 {
        match self {
            LicenseTier::Trial => 1,
            LicenseTier::Base => 2,
            LicenseTier::Reports => 3,
            LicenseTier::Advanced => 5,
            LicenseTier::Api => 999,
        }
    }

    pub fn max_transfers(&self) -> u32 {
        match self {
            LicenseTier::Trial => 0,
            LicenseTier::Base => 3,
            LicenseTier::Reports => 5,
            LicenseTier::Advanced => 10,
            LicenseTier::Api => 999,
        }
    }
}
```

---

## 7. Resumen de esfuerzo

| Paso | Componente | Cambio | Dias |
|------|-----------|--------|------|
| 1 | Cloudflare | Crear Worker + D1 + Admin panel | 2-3 |
| 2 | Server config | Agregar license_api_url, license_trial_days | 0.5 |
| 3 | Server license.rs | Reemplazar VENDOR_PUBLIC_KEY + VALIDATION_URL | 0.5 |
| 4 | Server license.rs | Reescribir validate_online() | 0.5 |
| 5 | Common features.rs | Agregar expires_at, remote_access, etc. | 1 |
| 6 | Common license.rs | Agregar trial expiry | 0.5 |
| 7 | Common hardware.rs | Reemplazar wmic por PowerShell | 0.5 |
| 8 | Server tray.rs | Agregar input de key en tray icon | 1 |
| 9 | Server main.rs | Modificar FirstRun flow | 1 |
| 10 | Server migrations | Crear viewer_sessions table | 0.5 |
| 11 | Testing | Probar flujo completo | 1-2 |
| **Total** | | | **8-11 dias** |

---

## 8. Orden de implementacion

```
1. Crear proyecto Cloudflare (Worker + D1 + Admin)
2. Generar keypair real y reemplazar placeholder
3. Implementar validate_online() en Rust
4. Agregar trial expiry a License struct
5. Reemplazar wmic por PowerShell
6. Agregar input de key en tray icon
7. Modificar FirstRun flow en main.rs
8. Agregar viewer_sessions para control de Viewers
9. Testing completo
```

---

## 9. Seguridad

| Vector de ataque | Proteccion |
|-----------------|------------|
| Copiar license.dat a otra PC | Firma Ed25519 + hardware_hash binding |
| Forzar firma Ed25519 | Imposible sin secret_key.bin (local) |
| Reemplazar public key en binario | Code signing (futuro) |
| Brute force de keys | Rate limiting en Cloudflare (5/hora) |
| Hackear Cloudflare Worker | Solo obtiene datos, no secret_key |
| Trial infinito | Expiracion de 7 dias en License struct |
| Modificar binario | Compilacion con strip + LTO |

---

## 10. Referencias

- Especificacion tecnica del producto (secciones 7, 14, 15, 16, 18, 19)
- AGENTS.md - Arquitectura del proyecto
- CONSTITUCION.md - Reglas de desarrollo
- docs/10-licensing/ - Documentacion existente
- Cloudflare Workers docs: https://developers.cloudflare.com/workers/
- Cloudflare D1 docs: https://developers.cloudflare.com/d1/
