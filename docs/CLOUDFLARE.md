# Panel de Administración de Licencias — Documentación Completa

## Descripción general

Sistema de licencias para aplicaciones de escritorio/web, compuesto por:

1. **API de licencias** (Cloudflare Worker) — maneja activación, validación, transferencia y revocación de licencias.
2. **Panel web de administración** — interfaz para gestionar licencias, ver estadísticas y revisar intentos de activación.
3. **Base de datos D1** — almacena licencias e intentos de activación.

---

## Arquitectura

```
                        Cloudflare Workers
                     ┌──────────────────────────┐
                     │   workshop-license-worker │
                     │                          │
  App cliente ──────►│  /api/v1/licenses/*      │──── D1 (licenses)
  (desktop/web)      │  /api/v1/admin/licenses/*│
                     │                          │
  Navegador ────────►│  /  (panel web HTML)     │──── D1 (activation_attempts)
  (admin)            │  /api/licenses           │
                     │  /api/stats              │
                     │  /api/attempts           │
                     │  /api/login              │
                     └──────────────────────────┘
```

---

## Cuenta y recursos

| Recurso | Valor |
|---|---|
| Cuenta Cloudflare | `carlitos24oz@gmail.com` |
| Worker | `workshop-license-worker` |
| URL del panel | `https://workshop-license-worker.carlitos24oz.workers.dev` |
| Base de datos D1 | `af8ed708-1e29-4943-8d7f-4e59ef03071b` |
| Binding D1 | `DB` |
| Secreto admin | `ADMIN_TOKEN` (configurado en Variables and Secrets) |
| Subdominio workers.dev | `carlitos24oz.workers.dev` |

---

## Estructura del proyecto

```
workshop-license-worker/
├── src/
│   └── index.ts          # Worker: API + servir panel web
├── public/                # Frontend (panel web)
│   ├── index.html         # Estructura del panel
│   ├── style.css          # Estilos (tema oscuro, responsive)
│   └── app.js             # Lógica del frontend
├── schema.sql             # Esquema de la base de datos D1
├── wrangler.jsonc         # Configuración de Wrangler
├── tsconfig.json          # Configuración de TypeScript
├── package.json           # Dependencias y scripts
└── README.md              # Resumen del proyecto
```

---

## Base de datos D1

### Tabla: licenses

| Columna | Tipo | Descripción |
|---|---|---|
| `id` | INTEGER PK | Autoincremental |
| `license_key` | TEXT UNIQUE | Clave de licencia (ej. `XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX`) |
| `tier` | TEXT | Nivel: `trial`, `base`, `pro`, `enterprise` |
| `hardware_hash` | TEXT | Hash del hardware donde se activó |
| `active` | INTEGER | 1 = activa, 0 = revocada |
| `max_concurrent_viewers` | INTEGER | Máximo de viewers concurrentes |
| `remote_access` | INTEGER | 1 = habilitado, 0 = deshabilitado |
| `advanced_reports` | INTEGER | 1 = habilitado, 0 = deshabilitado |
| `backup` | INTEGER | 1 = habilitado, 0 = deshabilitado |
| `max_transfers` | INTEGER | Máximo de transferencias a otro hardware |
| `transfer_count` | INTEGER | Transferencias realizadas |
| `activated_at` | TEXT | Fecha de activación |
| `created_at` | TEXT | Fecha de creación (auto) |
| `updated_at` | TEXT | Fecha de última actualización |

### Tabla: activation_attempts

| Columna | Tipo | Descripción |
|---|---|---|
| `id` | INTEGER PK | Autoincremental |
| `license_key` | TEXT | Clave usada en el intento |
| `hardware_hash` | TEXT | Hardware del intento |
| `success` | INTEGER | 1 = éxito, 0 = fallo |
| `error_message` | TEXT | Mensaje de error (si falló) |
| `created_at` | TEXT | Fecha del intento (auto) |

---

## API de licencias (para apps cliente)

### Endpoints públicos (sin autenticación)

#### POST /api/v1/licenses/activate

Activa una licencia en un hardware específico. Soporta trials automáticos.

**Request:**
```json
{
  "license_key": "XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX",
  "hardware_hash": "hash-del-pc-del-cliente"
}
```

**Casos especiales:**
- Si `license_key` es `"trial"`, crea o recupera una licencia trial automática para ese hardware.
- Rate limit: 5 intentos por hora por license_key.
- Si el hardware_hash no coincide y quedan transferencias, se transfiere automáticamente.

**Response (200):**
```json
{
  "success": true,
  "license": {
    "license_key": "XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX",
    "tier": "base",
    "hardware_hash": "hash-del-pc",
    "max_concurrent_viewers": 2,
    "remote_access": false,
    "advanced_reports": false,
    "backup": false,
    "max_transfers": 3,
    "transfer_count": 0,
    "activated_at": "2026-09-20T15:58:03.000Z"
  }
}
```

**Errores:**
- `400` — Missing license_key or hardware_hash
- `403` — License revoked / Max transfers reached
- `404` — Invalid license key
- `429` — Too many attempts (rate limit)

---

#### POST /api/v1/licenses/validate

Valida si una licencia está activa y coincide con el hardware.

**Request:**
```json
{
  "license_key": "XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX",
  "hardware_hash": "hash-del-pc-del-cliente"
}
```

**Response (200):**
```json
{
  "valid": true,
  "license": {
    "license_key": "XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX",
    "tier": "base",
    "hardware_hash": "hash-del-pc",
    "max_concurrent_viewers": 2,
    "remote_access": false,
    "advanced_reports": false,
    "backup": false,
    "max_transfers": 3,
    "transfer_count": 0,
    "activated_at": "2026-09-20T15:58:03"
  }
}
```

Si no coincide o no existe: `{ "valid": false }`

---

#### POST /api/v1/licenses/transfer

Transfiere una licencia a otro hardware (consume una transferencia).

**Request:**
```json
{
  "license_key": "XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX",
  "hardware_hash": "nuevo-hash-del-pc"
}
```

**Response (200):**
```json
{
  "success": true,
  "transfer_count": 1
}
```

**Errores:**
- `403` — Max transfers reached
- `404` — Invalid license

---

### Endpoints admin (requieren header `Authorization: Bearer <ADMIN_TOKEN>`)

#### GET /api/v1/admin/licenses

Lista todas las licencias.

**Response (200):**
```json
[
  {
    "id": 1,
    "license_key": "XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX",
    "tier": "base",
    "hardware_hash": "hash-del-pc",
    "active": 1,
    "max_concurrent_viewers": 2,
    "remote_access": 0,
    "advanced_reports": 0,
    "backup": 0,
    "max_transfers": 3,
    "transfer_count": 0,
    "activated_at": "2026-09-20T15:58:03",
    "created_at": "2026-09-20T15:58:03",
    "updated_at": null
  }
]
```

---

#### POST /api/v1/admin/licenses

Crea una nueva licencia.

**Request:**
```json
{
  "tier": "base",
  "max_concurrent_viewers": 2,
  "remote_access": false,
  "advanced_reports": false,
  "backup": false,
  "max_transfers": 3
}
```

**Response (200):**
```json
{
  "success": true,
  "license_key": "XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX"
}
```

---

#### DELETE /api/v1/admin/licenses/:key

Revoca una licencia (la marca como `active = 0`, no la elimina).

**Response (200):**
```json
{
  "success": true
}
```

---

## API del panel web (requieren `Authorization: Bearer <ADMIN_TOKEN>`)

### POST /api/login
Verifica el token de administrador. No requiere auth.

**Request:** `{ "token": "ADMIN_TOKEN" }`
**Response:** `{ "success": true }` o `{ "success": false, "message": "Token inválido" }`

### GET /api/licenses
Lista todas las licencias (para el panel).
**Response:** `{ "licenses": [...] }`

### POST /api/licenses
Crea una nueva licencia (mismo formato que `/api/v1/admin/licenses`).
**Response:** `{ "success": true, "license_key": "..." }`

### PATCH /api/licenses/:id
Actualiza una licencia (revocar, activar, cambiar features).

**Request (ejemplo - revocar):**
```json
{ "active": false }
```

**Campos actualizables:** `tier`, `max_concurrent_viewers`, `remote_access`, `advanced_reports`, `backup`, `max_transfers`, `active`

**Response:** `{ "success": true }`

### DELETE /api/licenses/:id
Elimina una licencia permanentemente.
**Response:** `{ "success": true }`

### GET /api/stats
Estadísticas del dashboard.
**Response:**
```json
{
  "total": 10,
  "active": 8,
  "revoked": 2,
  "trials": 3
}
```

### GET /api/attempts
Últimos 50 intentos de activación.
**Response:**
```json
{
  "attempts": [
    {
      "id": 1,
      "license_key": "XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX",
      "hardware_hash": "hash-del-pc",
      "success": 1,
      "error_message": null,
      "created_at": "2026-09-20T15:58:03"
    }
  ]
}
```

---

## Panel web de administración

### URL
```
https://workshop-license-worker.carlitos24oz.workers.dev
```

### Acceso
Ingresa con el valor de `ADMIN_TOKEN` configurado en el Worker.

### Funciones

| Función | Descripción |
|---|---|
| **Login** | Autenticación con token de administrador |
| **Dashboard** | Estadísticas: total, activas, revocadas, trials |
| **Crear licencia** | Formulario con tier, viewers máx, transferencias, features |
| **Tabla de licencias** | Key, tier, estado, hardware, viewers, transferencias, features, fecha |
| **Revocar licencia** | Marca la licencia como inactiva |
| **Activar licencia** | Reactiva una licencia revocada |
| **Eliminar licencia** | Borra la licencia permanentemente |
| **Intentos de activación** | Log de los últimos 50 intentos (éxito/fallo, error, fecha) |
| **Responsive** | Funciona en móvil y escritorio |

### Tiers de licencia

| Tier | Descripción |
|---|---|
| `trial` | Trial automático (2 viewers, sin features, 3 transferencias) |
| `base` | Licencia base (configurable) |
| `pro` | Licencia pro (configurable) |
| `enterprise` | Licencia enterprise (configurable) |

### Features configurables por licencia

| Feature | Descripción |
|---|---|
| `max_concurrent_viewers` | Número máximo de viewers concurrentes |
| `remote_access` | Acceso remoto habilitado |
| `advanced_reports` | Reportes avanzados habilitado |
| `backup` | Backup habilitado |
| `max_transfers` | Máximo de transferencias de hardware |

---

## Configuración de Wrangler (wrangler.jsonc)

```jsonc
{
  "name": "workshop-license-worker",
  "main": "src/index.ts",
  "compatibility_date": "2026-09-20",
  "assets": {
    "directory": "./public/",
    "binding": "ASSETS",
    "run_worker_first": true
  },
  "d1_databases": [
    {
      "binding": "DB",
      "database_id": "af8ed708-1e29-4943-8d7f-4e59ef03071b"
    }
  ]
}
```

### Explicación

| Campo | Valor | Descripción |
|---|---|---|
| `name` | `workshop-license-worker` | Nombre del Worker |
| `main` | `src/index.ts` | Archivo principal del Worker |
| `compatibility_date` | `2026-09-20` | Fecha de compatibilidad de la API |
| `assets.directory` | `./public/` | Carpeta con archivos estáticos (HTML/CSS/JS) |
| `assets.binding` | `ASSETS` | Nombre del binding para acceder a los assets |
| `assets.run_worker_first` | `true` | Las peticiones pasan primero por el Worker (API), si no coinciden, se sirve el HTML |
| `d1_databases[0].binding` | `DB` | Nombre del binding para acceder a D1 |
| `d1_databases[0].database_id` | `af8ed708-...` | ID de la base de datos D1 |

---

## Despliegue

### Método: Workers Builds (GitHub)

El Worker está conectado a un repositorio de GitHub. Cada `git push` a la rama `main` dispara un build y deploy automático en Cloudflare.

### Configuración del build

| Campo | Valor |
|---|---|
| Repositorio | GitHub (privado) |
| Branch | `main` |
| Build command | `npm install` |
| Deploy command | `npx wrangler deploy` |

### Pasos para desplegar cambios

1. Modifica el código en tu computadora
2. Sube los cambios a GitHub:
   ```bash
   git add -A
   git commit -m "Descripción del cambio"
   git push
   ```
3. Cloudflare detecta el cambio, compila y despliega automáticamente
4. Verifica el resultado en el dashboard de Cloudflare → Worker → Builds

---

## Configuración del secreto ADMIN_TOKEN

### Dónde está configurado

En el dashboard de Cloudflare:
1. Workers & Pages → workshop-license-worker → Settings → Variables and Secrets
2. El secreto `ADMIN_TOKEN` está definido ahí

### Cómo cambiarlo

1. Ve a Settings → Variables and Secrets
2. Edita o elimina `ADMIN_TOKEN`
3. Agrega uno nuevo (tipo: Secret, nombre: `ADMIN_TOKEN`, valor: tu token)
4. Haz un deploy (push vacío a GitHub):
   ```bash
   git commit --allow-empty -m "redeploy with new secret"
   git push
   ```

### Notas de seguridad

- El secreto no se puede ver una vez creado (aparece como `••••••••`)
- Si lo olvidas, debes crear uno nuevo (no se puede recuperar)
- El secreto se usa tanto para la API admin como para el login del panel web

---

## Cómo integrar la API en tus apps

### Flujo de licenciamiento

```
1. Cliente compra → Recibe license_key
2. App cliente → POST /api/v1/licenses/activate (con hardware_hash)
3. App cliente → Guarda la licencia localmente
4. App cliente → Periódicamente: POST /api/v1/licenses/validate
5. Si validate = false → App se bloquea
6. Si cliente cambia de PC → POST /api/v1/licenses/transfer
```

### Cómo generar el hardware_hash

El `hardware_hash` es un identificador único de la máquina del cliente. Depende del lenguaje:

**Python:**
```python
import subprocess
import hashlib

def get_hardware_hash():
    # Windows
    result = subprocess.check_output('wmic csproduct get UUID', shell=True).decode()
    uuid = result.strip().split('\n')[-1].strip()
    return hashlib.sha256(uuid.encode()).hexdigest()
```

**C# (.NET):**
```csharp
using System.Management;
using System.Security.Cryptography;
using System.Text;

string GetHardwareHash()
{
    var searcher = new ManagementObjectSearcher("SELECT UUID FROM Win32_ComputerSystemProduct");
    var uuid = searcher.Get().Cast<ManagementObject>().First()["UUID"].ToString();
    return SHA256Hash(uuid);
}
```

**JavaScript/Node.js (Electron):**
```javascript
const os = require('os');
const crypto = require('crypto');

function getHardwareHash() {
  const info = os.hostname() + os.platform() + os.arch() + JSON.stringify(os.cpus()[0]);
  return crypto.createHash('sha256').update(info).digest('hex');
}
```

### Ejemplo de activación (Python)

```python
import requests

API_URL = "https://workshop-license-worker.carlitos24oz.workers.dev"

def activate_license(license_key, hardware_hash):
    response = requests.post(f"{API_URL}/api/v1/licenses/activate", json={
        "license_key": license_key,
        "hardware_hash": hardware_hash
    })
    if response.status_code == 200:
        data = response.json()
        if data["success"]:
            print("Licencia activada:", data["license"])
            return data["license"]
    else:
        print("Error:", response.json())
    return None

def validate_license(license_key, hardware_hash):
    response = requests.post(f"{API_URL}/api/v1/licenses/validate", json={
        "license_key": license_key,
        "hardware_hash": hardware_hash
    })
    data = response.json()
    return data.get("valid", False)

# Uso
hw = get_hardware_hash()
license = activate_license("XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX", hw)
if license:
    # Guardar license localmente
    pass

# Validación periódica
if validate_license(license["license_key"], hw):
    print("Licencia válida")
else:
    print("Licencia inválida o revocada")
```

### Ejemplo de activación (C#)

```csharp
using System.Net.Http;
using System.Text;
using Newtonsoft.Json;

var API_URL = "https://workshop-license-worker.carlitos24oz.workers.dev";
var http = new HttpClient();

async Task<dynamic> ActivateLicense(string licenseKey, string hardwareHash)
{
    var body = new { license_key = licenseKey, hardware_hash = hardwareHash };
    var content = new StringContent(JsonConvert.SerializeObject(body), Encoding.UTF8, "application/json");
    var response = await http.PostAsync($"{API_URL}/api/v1/licenses/activate", content);
    var json = await response.Content.ReadAsStringAsync();
    return JsonConvert.DeserializeObject<dynamic>(json);
}
```

### Ejemplo de activación (JavaScript)

```javascript
const API_URL = "https://workshop-license-worker.carlitos24oz.workers.dev";

async function activateLicense(licenseKey, hardwareHash) {
  const res = await fetch(`${API_URL}/api/v1/licenses/activate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ license_key: licenseKey, hardware_hash: hardwareHash }),
  });
  const data = await res.json();
  if (data.success) {
    console.log("Licencia activada:", data.license);
    return data.license;
  }
  console.error("Error:", data.error);
  return null;
}
```

---

## Mantenimiento

### Cambiar el ADMIN_TOKEN

1. Dashboard → Worker → Settings → Variables and Secrets
2. Editar `ADMIN_TOKEN` con un nuevo valor
3. Hacer deploy (push a GitHub)

### Crear licencias manualmente

Desde el panel web:
1. Abrir `https://workshop-license-worker.carlitos24oz.workers.dev`
2. Login con ADMIN_TOKEN
3. Sección "Crear nueva licencia"
4. Configurar tier, viewers, transferencias, features
5. Click "Generar licencia"

### Revocar una licencia

Desde el panel web:
1. Tabla de licencias → botón "Revocar"
2. La licencia queda como `active = 0`
3. La app cliente recibirá `valid: false` al validar

### Eliminar una licencia

Desde el panel web:
1. Tabla de licencias → botón "Eliminar"
2. La licencia se borra permanentemente de la base de datos

### Ver intentos de activación

Desde el panel web:
1. Pestaña "Intentos"
2. Ver los últimos 50 intentos (éxito/fallo, error, fecha)

---

## Troubleshooting

### El panel muestra `{"status":"ok","version":"2.0"}` en lugar del HTML

Causa: La ruta `/` estaba devolviendo JSON en lugar de servir el panel.

Solución: Se cambió el health check de `/` a `/health` en `src/index.ts`. Si vuelve a pasar, verificar que el código tenga:
```typescript
if (path === '/health' && method === 'GET') {
  return jsonResponse(200, { status: 'ok', version: '2.0' });
}
```

### El secreto ADMIN_TOKEN no aparece

Causa: El deploy desde GitHub puede reemplazar la configuración del Worker.

Solución:
1. Dashboard → Worker → Settings → Variables and Secrets
2. Agregar `ADMIN_TOKEN` como Secret
3. Hacer un nuevo deploy (push a GitHub)

### La app cliente recibe error de CORS

El Worker tiene CORS configurado para permitir cualquier origen (`*`). Si hay problemas, verificar que el código incluya:
```typescript
const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, PATCH, DELETE, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type, Authorization',
};
```

### El build falla en Cloudflare

Verificar en el dashboard:
1. Worker → Builds → ver el log del último build
2. Comprobar que `npm install` y `npx wrangler deploy` se ejecutan sin errores
3. Verificar que `wrangler.jsonc` tenga el `database_id` correcto

---

## Historial de cambios

| Fecha | Cambio |
|---|---|
| 2026-09-20 | Creación inicial del Worker con API de licencias |
| 2026-09-20 | Generación del panel web de administración |
| 2026-09-20 | Fusión del panel web con la API existente |
| 2026-09-20 | Conexión del repo a Workers Builds (GitHub) |
| 2026-09-20 | Fix: servir panel en `/` en lugar de JSON |
| 2026-09-20 | Configuración del secreto ADMIN_TOKEN |
| 2026-09-20 | Documentación completa del sistema |

---

## URLs importantes

| Recurso | URL |
|---|---|
| Panel web | `https://workshop-license-worker.carlitos24oz.workers.dev` |
| Health check | `https://workshop-license-worker.carlitos24oz.workers.dev/health` |
| API activate | `https://workshop-license-worker.carlitos24oz.workers.dev/api/v1/licenses/activate` |
| API validate | `https://workshop-license-worker.carlitos24oz.workers.dev/api/v1/licenses/validate` |
| API transfer | `https://workshop-license-worker.carlitos24oz.workers.dev/api/v1/licenses/transfer` |
| Dashboard Cloudflare | `https://dash.cloudflare.com/2fb333489645a3863fa34fe48af00e44/workers/services/view/workshop-license-worker/production` |
| Workers & Pages | `https://dash.cloudflare.com/?to=/:account/workers-and-pages` |

---

*Documentación generada el 2026-09-20*
