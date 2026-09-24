# PLAN_MEJORA

Última actualización: 2026-09-24

## Estado general
- Total items: 28
- Completados: 16
- Parciales: 3
- Pendientes (este repo): 4
- Pendientes (Worker): 2
- No verificables: 1 (GAP-2 — D1 remoto)
- Dependientes: 2 (bloqueados por otros pendientes)
- Notas: 1 (tray sin días restantes)

---

## Completados

| ID | Descripción | Fuente | Commit / Evidencia |
|---|---|---|---|
| H1 | Worker: `activated_at` no se reinicia en trials existentes | Worker | `index.ts:191-197` — retorna `activated_at` y `expires_at` originales de la DB |
| H2 | `save_license` ya no necesita `vendor_secret_key` | Server | `31622ae` — `server/license.rs:62-70` → `save_signed_license` atómica |
| H3 | Fallback offline ya no crea trial infinito | Server | `31622ae` — `main.rs:168-174` → `Unreachable` hace `exit(1)` |
| H4 | `trial_days` eliminado de config | Server | `31622ae` — `config.rs:15-20` → `LicenseSection` solo tiene `api_url` |
| H5 | Worker: trial revocado no se recrea | Worker | `index.ts:179-182` — retorna 403 "Trial revoked" si `active=0` |
| H6 | Worker: rate limit 3 trials/IP/24h | Worker | `index.ts:200-208` — `WHERE ip = ? AND license_key = 'trial' AND success = 1` |
| H8 | Worker: rate limit solo cuenta fallos | Worker | `index.ts:236-242` — `WHERE success = 0` |
| H9 | Worker: `transfer_count` solo incrementa si HW difiere | Worker | `index.ts:270-277` — solo si `hardware_hash !== hardware_hash` |
| H10 | Worker: `activated_at` no se reinicia | Worker | `index.ts:282-287` — solo setea `hardware_hash` y `active=1` |
| H11 | Worker: trial inserta `viewers=1, transfers=0` | Worker | `index.ts:216-218` — `VALUES (?, 'trial', ?, 1, 0, 0, 0, 0, 0, ?, ?)` |
| H12 | Worker: `generateKey()` usa `crypto.getRandomValues` | Worker | `index.ts:101-116` — rejection sampling con `limit = 256 - (256 % chars.length)` |
| H13 | Worker: endpoint `/transfer` eliminado | Worker | `index.ts:494` — comentario: "H13: /transfer removed" |
| H16 | `expires_at` tiene `#[serde(default)]` | Server | `features.rs:133` — pre-existente |
| H17 | `get_hardware_id()` no hashea vacío | Server | `c02d27c` — `hardware.rs:8-18` → winreg + PowerShell fallback + `Err` |
| H15 | `exit(1)` con diálogo visible en release | Server | `fn fatal_license_dialog` (`MessageBoxW` / stderr) + cierre de splash (`ready_tx`) antes del diálogo en los 5 exits de licencia (`main.rs`) |

---

## Parciales

| ID | Descripción | Qué falta | Evidencia |
|---|---|---|---|
| H7 | Worker firma pero servidor tiene fallback JSON plano | Servidor verifica `signed_license` (`server/license.rs:146-170`) pero fallback a JSON sin firma (`:173-197`). Worker SÍ envía `signed_license` (`index.ts:196,229,295,335`) — el fallback es solo compatibilidad con Workers antiguos. | `server/license.rs:173-197` + `index.ts:341-351` |
| H14 | `handlePanelUpdateLicense` sin validación de tier/rangos | Sin whitelist de `tier` (`:443`). Sin rango para `max_transfers` (`:448`). SQL parametrizado (`:453`). `id` es `license_key` (string), no numérico. | `index.ts:429-454` |
| H18 | Mensaje "Contacte al proveedor" engañoso | `main.rs:143` OK ("Activando licencia de prueba…"), pero `:134` y `:182` dicen "Contacte al proveedor" (confuso en trial) | `main.rs:134,182` |

---

## Pendientes

### Este repo (workshop-manager)

| ID | Descripción | Prioridad | Criterio de aceptación |
|---|---|---|---|
| H19 | Retroceso del reloj no detectado | Baja | Guardar `last_seen_utc` cifrado (AES-256-GCM). Si `now < last_seen_utc - 10min` → exigir revalidación online |
| GAP-1 | Endpoint `/revalidate` no implementado | Alta | Endpoint que el servidor llame para verificar licencia vigente. Worker ya tiene `handleValidate` (`index.ts:299-337`) — solo falta el caller en el servidor |
| GAP-3 | Ping silencioso / heartbeat no implementado | Media | Tarea `tokio::spawn` cada 24h que llame al Worker. Actualizar `license.dat` si renueva. Watcher actual solo lee disco |
| REV | Revalidación periódica no implementada | Alta | GAP-1 + GAP-3 + período de gracia. Watcher debe tratar `license.dat` desaparecido como `Invalid` |

### Worker (workshop-license-panel-v2)

| ID | Descripción | Prioridad | Criterio de aceptación |
|---|---|---|---|
| H20 | `ADMIN_TOKEN` hardcoded + comparación `===` | Alta | Token aleatorio largo; comparación SHA-256 constant-time (`crypto.subtle.digest`) |
| GAP-5 | CORS `*` global incluye admin | Media | CORS restringido a solo `/api/v1/licenses/*` (llamadas del servidor). Admin/panel sin CORS o con origen fijo |

---

## No verificables

| ID | Descripción | Razón |
|---|---|---|
| GAP-2 | Contador de migraciones en D1 no existe | Requiere acceso a D1 remoto para verificar si existe tabla de versiones. `schema.sql` tiene migraciones comentadas pero sin tracking |

---

## Dependientes (bloqueados)

| ID | Descripción | Bloqueado por |
|---|---|---|
| GAP-4 | Refresh tokens JWT | Requiere decisión de diseño. Relacionado con B-7 de AUDITORIA.md |
| GAP-6 | Revocación detectada en cliente | Depende de GAP-1 (`/revalidate`) |

---

## Notas

### Tray sin días restantes
`tray.rs:104-105` muestra tier y key (`"Licencia: BASE (ABC1-1234)"`) pero no muestra `days_until_expiry`. Fase 5.7 de PLAN_MEJORA_LICENCIAS.md lo pide.

### H7 — Fallback JSON es compatibilidad, no debilidad
El Worker actual SÍ firma todas las respuestas (`signLicenseForResponse` en `:341-351`). El fallback JSON en el servidor (`:173-197`) es solo para compatibilidad con Workers antiguos. Si se garantiza que el Worker siempre envía `signed_license`, el fallback puede eliminarse.

### Seguridad del Viewer
`danger_accept_invalid_certs` en viewer (`api.rs:92`, `server_settings.rs:85`) — riesgo aceptado para cliente desktop local.

### Archivos clave del sistema de licencias

**Server (Rust):**
| Archivo | Rol |
|---|---|
| `crates/workshop-common/src/features.rs` | `License`, `LicenseTier`, `Feature`, `is_valid()`, `is_expired()` |
| `crates/workshop-common/src/license.rs` | `sign_license()`, `verify_license()`, `validate_hardware()` |
| `crates/workshop-common/src/hardware.rs` | `get_hardware_id()` — winreg + PowerShell fallback |
| `crates/workshop-server/src/license.rs` | `load_license()`, `validate_online()`, `save_signed_license()` |
| `crates/workshop-server/src/main.rs:120-186` | Flujo de arranque de licencia |
| `crates/workshop-server/src/main.rs:329-367` | Watcher de expiración (300s) |
| `crates/workshop-server/src/config.rs:15-28` | `LicenseSection` (solo `api_url`) |
| `crates/license-tool/src/main.rs` | CLI para generar keypair y emitir licencias offline |

**Worker (TypeScript):**
| Archivo | Rol |
|---|---|
| `src/index.ts` | Worker completo — activation, validation, signing, admin, panel |
| `schema.sql` | Esquema D1 — `licenses` (16 cols), `activation_attempts` (6 cols) |
| `wrangler.jsonc` | Config — `vars`, `d1_databases`, `assets` |
