# Plan técnico: endurecer y corregir el sistema de licencias de WorkshopManager

Documento para una IA/desarrollador que implementará los cambios. Está escrito en orden de ejecución.
Cada hallazgo indica archivo, causa, corrección y criterio de aceptación.

> **Regla general para quien implemente esto:** antes de tocar cualquier archivo, léelo completo.
> No inventes nombres de campos, enums ni formatos. Donde este documento dice **[VERIFICAR]**,
> la información no estaba disponible al escribirlo y hay que leer el código real.
> Haz commits pequeños (uno por fase) y corre `cargo check --workspace` y `cargo test --workspace` al terminar cada fase.

---

## 0. Contexto

### 0.1 Arquitectura relevante

| Pieza | Ubicación | Rol |
|---|---|---|
| Servidor Rust (Axum, PostgreSQL embebido) | `crates/workshop-server` | Valida licencia al arrancar; corre como app de bandeja en Windows (`windows_subsystem = "windows"` en release) |
| Tipos/firmas compartidos | `crates/workshop-common` (`features.rs`, `license.rs`, `hardware.rs`) | `License`, `LicenseTier`, firma/verificación Ed25519, hardware ID |
| Herramienta de emisión manual | `crates/license-tool` | Genera keypair y emite licencias firmadas offline |
| Worker de licencias | Proyecto Cloudflare (`src/index.ts`, compilado a `index.js`), D1 (`licenses`, `activation_attempts`) | Activa trials y licencias pagadas, panel admin |
| Instalador | MSI vía `cargo wix` (WiX v3), CI en `.github/workflows/release.yml` | El MSI instala solo el `.exe`; no instala `config/server.toml` |

### 0.2 Flujo actual (resumen)

1. `main.rs` llama `license::load_license(&data_dir)`. Si no existe `license.dat` devuelve `FirstRun(hw_hash)`.
2. En `FirstRun` se llama `license::validate_online(api_url, "trial", hw_hash, trial_days)` contra `POST {api_url}/api/v1/licenses/activate`.
3. Si responde OK, se arma un `License` y se llama `license::save_license`, que intenta firmar localmente con `vendor_secret_key.bin`.
4. Si no hay respuesta, se crea un trial en memoria (`create_trial_license_with_days`) que no se persiste.
5. Un watcher revisa `license.dat` cada 300 s y apaga el servidor si expiró.

### 0.3 Estado ya resuelto (no rehacer)

- `default_license_api_url()` en `config.rs` ahora devuelve la URL real del Worker.
- Se quitó `.danger_accept_invalid_certs(true)` de `validate_online`. **Verificar que realmente no quedó** (`grep -rn danger_accept_invalid_certs crates/`).
- `CREATE_NO_WINDOW` aplicado en `hardware.rs` y en el `Drop` de `db_manager.rs`.
- `hide_console_for_children()` en `main.rs` (solo release).

---

## 1. Hallazgos (resumen priorizado)

| # | Sev. | Dónde | Problema |
|---|---|---|---|
| H1 | **Crítica** | Worker `handleActivate` (rama trial) | `activated_at` se devuelve como `new Date()` en cada llamada: el trial nunca expira |
| H2 | **Crítica** | `server/license.rs::save_license` | Exige `vendor_secret_key.bin` en la PC del cliente: nunca persiste, el trial se reinicia en cada arranque |
| H3 | **Crítica** | `server/main.rs` (rama `None` de `validate_online`) | Fallback offline crea trial nuevo en memoria en cada arranque: trial infinito sin internet |
| H4 | **Alta** | `server/config.rs` / `main.rs` | La duración del trial la decide `trial_days` del `server.toml` local, editable por el usuario |
| H5 | **Alta** | Worker | Trial revocado (`active = 0`) → se crea otro trial nuevo para el mismo hardware |
| H6 | **Alta** | Worker | Trials ilimitados con `hardware_hash` inventados; sin rate limit por IP |
| H7 | **Alta** | Worker | No firma la licencia; el servidor confía en JSON plano |
| H8 | Media | Worker | Rate limit de 5/h cuenta también intentos exitosos |
| H9 | Media | Worker | `transfer_count` devuelto siempre `+1`, aunque no hubo transferencia |
| H10 | Media | Worker | `activated_at` se reinicia en cada reactivación del mismo hardware |
| H11 | Media | Worker | Trial creado con `max_concurrent_viewers=2, max_transfers=3`; en Rust el trial es 1 y 0 |
| H12 | Media | Worker | `generateKey()` usa `Math.random()` |
| H13 | Media | Worker | `/api/v1/licenses/transfer` sin autenticación |
| H14 | Media | Worker | `handlePanelUpdateLicense` con `sets` posiblemente vacío + `id` desde el path sin validar |
| H15 | Media | `server/main.rs` | `process::exit(1)` sin mensaje visible (no hay consola en release) |
| H16 | Media | `workshop-common/features.rs` | `expires_at` nuevo sin `#[serde(default)]`: rompe deserialización de `license.dat` antiguos |
| H17 | Media | `workshop-common/hardware.rs` | Si fallan las 3 consultas, se hashea un valor vacío/parcial (colisiones); 3 procesos PowerShell en cada arranque |
| H18 | Baja | `server/main.rs` | Mensaje "Contacte al proveedor" en `FirstRun` es engañoso (se activa solo) |
| H19 | Baja | Seguridad | Retroceso del reloj del sistema no se detecta |
| H20 | Baja | Worker | `ADMIN_TOKEN` comparado con `===`; CORS `*` en rutas admin; rutas admin duplicadas |

---

## 2. Decisiones de diseño (confirmar con el dueño del producto antes de implementar)

**D1. Fuente de verdad del vencimiento:** el Worker. El servidor NUNCA calcula `expires_at` a partir de `trial_days` local.
El Worker incluye `expires_at` dentro de la licencia **firmada**. `trial_days` pasa a ser configuración del Worker (`env.TRIAL_DAYS`).

**D2. Quién firma:** el Worker, con la clave secreta Ed25519 guardada como *secret* de Cloudflare.
El servidor Rust solo verifica con `VENDOR_PUBLIC_KEY`. `vendor_secret_key.bin` no debe existir jamás en un cliente ni en el MSI.

**D3. Política offline (recomendada):**
- **Primera activación requiere internet.** Sin `license.dat` firmado válido y sin conexión → el servidor NO arranca el trial; muestra mensaje claro ("Se requiere conexión a internet para activar").
- Con licencia ya guardada y firmada: funciona offline. Se revalida contra el Worker cada 24 h; si falla la red, se tolera un período de gracia (propuesto: 7 días para licencias pagadas, 0 extra para trial porque su `expires_at` ya es absoluto).

**D4. Formato de transporte firmado:** el Worker envía `signed_license` = bytes firmados. Rust verifica la firma sobre los **bytes recibidos** y recién después deserializa. Nunca re-serializar para verificar.

**D5. Un trial por hardware, de por vida.** Un trial vencido o revocado no se recrea.

---

## 3. Plan de ejecución

### Fase 0 — Preparación y verificación de secretos

1. Crear rama: `git checkout -b feat/license-hardening`.
2. Verificar que la clave secreta nunca se subió al repo:
   ```powershell
   git log --all --oneline -- vendor_secret_key.bin vendor_public_key.bin
   git log --all -S"vendor_secret_key" --oneline
   ```
   - Si `vendor_secret_key.bin` aparece en algún commit: **considerar la clave comprometida**. Generar un keypair nuevo (`license-tool generate-keypair`), reemplazar `VENDOR_PUBLIC_KEY` en `crates/workshop-server/src/license.rs` y reemitir licencias.
3. Confirmar que `.gitignore` contiene `vendor_secret_key.bin` y `vendor_public_key.bin` (ya está en el commit `dc26633`).
4. **[VERIFICAR]** Formato de `vendor_secret_key.bin`: tamaño en bytes (32 = semilla; 64 = semilla + pública, formato típico de `ed25519-dalek`). Anotarlo; la Fase 3 depende de esto.
5. Leer completos: `crates/workshop-common/src/license.rs` (funciones `sign_license`, `verify_license`, tipos), `features.rs` (`License`, `LicenseTier` y sus atributos `serde`), `crates/license-tool/src/main.rs`.

**Criterio de aceptación:** documentado (en el PR) el formato exacto de firma actual y el nombre serde de cada variante de `LicenseTier`.

---

### Fase 1 — Base de datos del Worker (D1)

Crear migración (por ejemplo `migrations/0002_license_hardening.sql`) y aplicarla con `npx wrangler d1 migrations apply <DB_NAME> --remote`.
**[VERIFICAR]** el esquema actual antes de escribirla (`npx wrangler d1 execute <DB_NAME> --remote --command ".schema"`).

Cambios requeridos:

```sql
-- IP de origen para limitar trials por IP (H6)
ALTER TABLE activation_attempts ADD COLUMN ip TEXT;
CREATE INDEX IF NOT EXISTS idx_attempts_ip_created ON activation_attempts(ip, created_at);
CREATE INDEX IF NOT EXISTS idx_attempts_key_created ON activation_attempts(license_key, created_at);

-- Vencimiento explícito (D1). NULL = permanente
ALTER TABLE licenses ADD COLUMN expires_at TEXT;

-- Un solo trial por hardware, de por vida (D5, H5)
CREATE UNIQUE INDEX IF NOT EXISTS idx_licenses_trial_hw
  ON licenses(hardware_hash) WHERE tier = 'trial';
```

Notas:
- Si ya existen filas de trial duplicadas para un mismo `hardware_hash`, el índice único fallará. Depurar antes (conservar la más antigua).
- Backfill: para trials existentes, `UPDATE licenses SET expires_at = datetime(activated_at, '+7 days') WHERE tier='trial' AND expires_at IS NULL;`
- D1 guarda fechas como texto `YYYY-MM-DD HH:MM:SS` en UTC. Al parsear en JS: `new Date(str.replace(" ", "T") + "Z")`.

**Criterio de aceptación:** las tres sentencias corren sin error en remoto; `.schema` muestra las columnas/índices nuevos.

---

### Fase 2 — Correcciones del Worker (lógica, sin firma todavía)

Archivo: `src/index.ts` (fuente TypeScript; **no editar `index.js` compilado**).

**2.1 Trial (H1, H4, H5, H6, H11).** Reescribir la rama `license_key === "trial"` de `handleActivate`:

1. Leer `TRIAL_DAYS = Number(env.TRIAL_DAYS ?? 7)` (agregar `[vars] TRIAL_DAYS = "7"` en `wrangler.toml`).
2. Buscar cualquier trial de ese hardware: `SELECT * FROM licenses WHERE hardware_hash = ? AND tier = 'trial'` (**sin** filtrar por `active`).
3. Si existe y `active = 0` → responder `403 {error:"Trial revoked"}` y registrar intento fallido. **No crear otro.**
4. Si no existe:
   - Aplicar rate limit por IP (`CF-Connecting-IP`): máximo 3 trials nuevos por IP cada 24 h (contando filas de `activation_attempts` con `success=1` y `license_key='trial'` por esa IP). Si excede → `429`.
   - Insertar con `max_concurrent_viewers = 1`, `max_transfers = 0`, `activated_at = datetime('now')`, `expires_at = datetime('now', '+N days')`.
5. Calcular expiración **desde la base**, no desde "ahora": si `Date.now() > expires_at` → `403 {error:"Trial expired"}`.
6. Responder con `activated_at` y `expires_at` **reales de la fila** (ISO-8601 UTC), `transfer_count: 0`.

**2.2 Rate limit de claves pagadas (H8).** Contar solo fallos:
```sql
SELECT COUNT(*) c FROM activation_attempts
WHERE license_key = ? AND success = 0 AND created_at > datetime('now','-1 hour')
```
Además limitar por IP. Registrar siempre `ip` en `logAttempt`.

**2.3 Transferencias y fechas (H9, H10).** En la activación de clave pagada:
- Calcular `newCount = Number(lic.transfer_count)`. Solo si `lic.hardware_hash` existe **y** difiere del actual: verificar `newCount < max_transfers`, luego `newCount += 1` y `UPDATE ... transfer_count = transfer_count + 1`.
- Si el hardware coincide o la clave nunca se activó: **no** tocar `transfer_count`.
- No reiniciar `activated_at`: usar `activated_at = COALESCE(activated_at, datetime('now'))`. Solo actualizarlo en una transferencia real, si así se decide.
- Responder `transfer_count: newCount` y el `activated_at` real de la fila.

**2.3b** Comprobar `expires_at` también para licencias pagadas con vencimiento (si `expires_at` no es NULL y ya pasó → `403`).

**2.4 Generación de claves (H12).** Reemplazar `Math.random()` por `crypto.getRandomValues` con **rejection sampling** (evitar sesgo de módulo):
```ts
function randomChar(chars: string): string {
  const limit = 256 - (256 % chars.length);
  const b = new Uint8Array(1);
  do { crypto.getRandomValues(b); } while (b[0] >= limit);
  return chars[b[0] % chars.length];
}
```

**2.5 `/transfer` sin auth (H13).** Elegir una de:
(a) eliminar el endpoint y dejar transferencias solo vía `/activate` (ya maneja cambio de hardware con límite), o
(b) exigir `Authorization: Bearer <ADMIN_TOKEN>`.
Recomendado: (a), y borrar `handleTransfer` y su ruta.

**2.6 Panel admin (H14, H20).**
- `handlePanelUpdateLicense`: si `sets` solo contiene `updated_at` (sin campos reales) → `400`. Validar que `id` sea entero (`Number.isInteger`). Validar `tier` contra la lista permitida (`trial,base,reports,advanced,api`) y rangos numéricos.
- Comparación de `ADMIN_TOKEN` a tiempo constante (comparar hashes SHA-256 de ambos con `crypto.subtle.digest` y comparar byte a byte sin salida temprana).
- Eliminar rutas admin duplicadas (`/api/v1/admin/*` vs `/api/*`) o dejar una sola.
- CORS: mantener `*` solo para `/api/v1/licenses/*` (llamadas del servidor). Para admin/panel, no emitir CORS o restringir al origen del panel.
- Recomendado: proteger el panel con Cloudflare Access.

**Criterio de aceptación Fase 2** (probar con `curl`/`Invoke-RestMethod` contra `wrangler dev`):
- Dos llamadas `trial` seguidas con el mismo `hardware_hash` devuelven el **mismo** `activated_at` y `expires_at`.
- Con `TRIAL_DAYS=0` (o forzando `expires_at` en el pasado en la DB) la respuesta es `403 Trial expired`.
- Revocar un trial (`active=0`) y volver a pedir trial → `403`, no se crea fila nueva.
- 4.º trial nuevo desde la misma IP en 24 h → `429`.
- Reactivar 6 veces con éxito la misma clave pagada no dispara el rate limit.
- `transfer_count` no cambia al reactivar en el mismo hardware.

---

### Fase 3 — Firma en el Worker (H7, D2, D4)

1. **Secret de Cloudflare** (nunca en `wrangler.toml` ni en el repo). Usar la semilla de 32 bytes:
   ```powershell
   # Si el archivo tiene 64 bytes, tomar solo los primeros 32
   $b = [IO.File]::ReadAllBytes("vendor_secret_key.bin")
   [Convert]::ToBase64String($b[0..31]) | npx wrangler secret put VENDOR_SECRET_KEY
   ```
2. **Firma con WebCrypto Ed25519** (Workers lo soporta; **[VERIFICAR]** en la doc de Cloudflare el nombre de algoritmo vigente, `Ed25519`). La semilla se envuelve en PKCS#8 con el prefijo fijo:
   ```ts
   const PKCS8_ED25519_PREFIX = Uint8Array.from(
     [0x30,0x2e,0x02,0x01,0x00,0x30,0x05,0x06,0x03,0x2b,0x65,0x70,0x04,0x22,0x04,0x20]);

   async function signPayload(env: Env, payload: Uint8Array): Promise<Uint8Array> {
     const seed = Uint8Array.from(atob(env.VENDOR_SECRET_KEY), c => c.charCodeAt(0));
     if (seed.length !== 32) throw new Error("VENDOR_SECRET_KEY must be a 32-byte seed");
     const pkcs8 = new Uint8Array(PKCS8_ED25519_PREFIX.length + 32);
     pkcs8.set(PKCS8_ED25519_PREFIX); pkcs8.set(seed, PKCS8_ED25519_PREFIX.length);
     const key = await crypto.subtle.importKey("pkcs8", pkcs8, { name: "Ed25519" }, false, ["sign"]);
     return new Uint8Array(await crypto.subtle.sign("Ed25519", key, payload));
   }
   ```
   Evitar `btoa(String.fromCharCode(...bytes))` con arreglos grandes (límite de argumentos); para payloads pequeños está bien, pero preferir un loop.
3. **Formato del payload:** debe ser **exactamente** lo que `verify_license`/`sign_license` de `workshop-common/src/license.rs` esperan **[VERIFICAR]**. Dos opciones:
   - (Preferida) Adaptar el Worker al formato existente de `sign_license` (leerlo y replicarlo byte a byte: orden de campos, codificación, prefijo/sufijo de firma).
   - Si el formato actual es difícil de replicar, **cambiar Rust** para definir uno nuevo y simple: `signed_license = base64( firma(64 bytes) || json_utf8 )`, y que `verify_license` valide `firma` sobre `json_utf8` recibido y luego haga `serde_json::from_slice` de esos mismos bytes. Actualizar `license-tool` para emitir el mismo formato.
4. **Contenido del JSON firmado** (nombres deben coincidir con `#[derive(Deserialize)] License` **[VERIFICAR]**):
   `license_key`, `tier` (nombre serde exacto del enum), `hardware_hash`, `max_viewers` (**ojo:** el Worker hoy usa `max_concurrent_viewers`; mapear), `max_transfers`, `transfer_count`, `activated_at`, `expires_at` (`null` si permanente).
5. **Respuesta del Worker:**
   ```json
   { "success": true, "license": { ...campos legibles... }, "signed_license": "<base64>" }
   ```
   El campo `license` es informativo; el servidor debe usar solo `signed_license`.

**Criterio de aceptación:** un test de integración (o script) obtiene `signed_license` del Worker de desarrollo y `workshop_common::license::verify_license(bytes, VENDOR_PUBLIC_KEY)` devuelve `Ok`. Modificar un solo byte del payload o de la firma debe dar `Err`. Confirmar además que la clave pública derivada de `VENDOR_SECRET_KEY` coincide con `VENDOR_PUBLIC_KEY` del código Rust.

---

### Fase 4 — `workshop-common` (Rust)

1. **H16:** en `License`, marcar el campo nuevo:
   ```rust
   #[serde(default)]
   pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
   ```
   Test: deserializar un JSON antiguo sin `expires_at` → `None`.
2. Añadir (o adaptar) `pub fn verify_signed_license(signed: &[u8], public_key: &[u8]) -> Result<License, String>` que: (a) verifica la firma sobre los bytes crudos, (b) deserializa esos mismos bytes, (c) devuelve la `License`. Sin efectos secundarios.
3. `License::is_valid()` ya comprueba `expires_at`. Añadir tests para: sin expiración, vencida, por vencer, y `days_until_expiry`.
4. **H17 (hardware ID):** en `get_hardware_id()`:
   - Si **todas** las consultas devuelven `None`, retornar `Err` (no hashear vacío).
   - Considerar sustituir las 3 llamadas a PowerShell por lecturas de registro con `winreg` (ya es dependencia en el servidor): `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid` es estable y no lanza procesos. Si se mantiene PowerShell, ejecutar las 3 consultas en **un solo** proceso.
   - Ojo: cambiar el algoritmo cambia el hash y por tanto invalida activaciones existentes. Si ya hay clientes, mantener el algoritmo actual o migrar con compatibilidad.

**Criterio de aceptación:** `cargo test -p workshop-common` en verde con los tests nuevos.

---

### Fase 5 — Servidor (`workshop-server`)

**5.1 `license.rs::validate_online` (H4, H7).**
- Nueva firma: `async fn validate_online(api_url: &str, license_key: &str, hardware_hash: &str) -> OnlineResult` (sin `trial_days`).
- `enum OnlineResult { Ok { signed: Vec<u8>, license: License }, Rejected(String), Unreachable(String) }` — distinguir **rechazo** (HTTP 403/404/429 con `error`) de **sin conexión** (fallo de red/timeout). Hoy ambos son `None`.
- Leer `signed_license` (base64) → `verify_signed_license(bytes, VENDOR_PUBLIC_KEY)`. Si la verificación falla → tratar como `Rejected("firma inválida")`.
- Verificar además que `license.hardware_hash == hardware_hash` local y que `license_key` no esté vacío.
- No usar `danger_accept_invalid_certs`. Mantener timeout de 10 s.
- No loguear el cuerpo completo de la respuesta con datos sensibles; sí el `error` del Worker.

**5.2 Persistencia (H2).**
- Eliminar `load_vendor_secret_key` y reescribir `save_license` → `save_signed_license(&signed_bytes, &data_dir)` (ya existe `save_signed_license`; quitar el `#[allow(dead_code)]` y usarla).
- Escritura atómica: escribir a `license.dat.tmp` y `rename` a `license.dat`.
- Nada en el servidor debe leer `vendor_secret_key.bin`.

**5.3 Carga y decisión de arranque (H3, H4, D3).** `load_license` debe verificar la firma del `license.dat` con `VENDOR_PUBLIC_KEY` y comprobar `hardware_hash` contra el actual. Lógica en `main.rs`:

```
match load_license:
  Valid(lic) (firma OK, hw OK):
      validate_license (expiración) → si expirada: intentar UNA revalidación online;
          si el Worker no renueva → mostrar mensaje + salir (H15)
      → arrancar; programar revalidación cada 24 h
  FirstRun(hw):
      validate_online(api, "trial", hw):
          Ok        → save_signed_license; arrancar
          Rejected  → mostrar mensaje con el motivo (expirado/revocado/límite) y salir
          Unreachable → mostrar "Se requiere internet para activar" y salir   ← elimina el trial infinito (H3)
  Invalid(e): mostrar mensaje y salir
```

- **Eliminar** la rama que crea `create_trial_license_with_days` en memoria. Puede quedar la función solo para tests, o borrarse.
- Eliminar `trial_days`/`license_trial_days` de `Config` y `ServerConfig` (o dejarlo ignorado y marcado deprecado). El servidor ya no lo usa.
- `is_placeholder_key()`: si la clave pública es todo ceros, el servidor **no debe arrancar** (hoy solo loguea y sigue). Es un error de build/configuración.
- Cambiar el log de `FirstRun` "Contacte al proveedor para activar la licencia" por algo acorde ("Activando licencia de prueba…").

**5.4 Revalidación periódica y período de gracia.**
- Tarea `tokio::spawn` cada 24 h: llamar `POST /api/v1/licenses/validate` (ya existe en el Worker) o `/activate` con la clave real.
  - Respuesta `valid:true` → actualizar `license.dat` con la nueva `signed_license` si el Worker la envía (añadirla también a `handleValidate`).
  - `valid:false` (revocada/hardware distinto) → apagar con mensaje.
  - `Unreachable` → guardar `last_online_ok` y permitir hasta N días de gracia (D3); pasado el plazo, apagar con mensaje.
- El watcher actual (300 s) debe usar el mismo criterio: hoy ignora `FirstRun` (`=> {}`), lo que permite eludirlo borrando `license.dat`. Con la nueva política, si `license.dat` desaparece en ejecución → tratar como `Invalid`.

**5.5 Mensajes visibles (H15).** En release no hay consola: crear helper `fn fatal_license_dialog(title: &str, msg: &str)` con `MessageBoxW` (`windows-sys`, feature `Win32_UI_WindowsAndMessaging` **ya habilitada**) y llamarlo antes de cada `std::process::exit(1)` de licencia. Cerrar también el splash (`ready_tx`) antes de mostrar el diálogo si sigue abierto. En Linux, imprimir a stderr.

**5.6 Antimanipulación de reloj (H19, opcional pero recomendado).**
- Guardar `last_seen_utc` cifrado (usar `crypto` AES-256-GCM existente) al arrancar y en cada revalidación.
- Si `now < last_seen_utc - 10 min` → tratar como manipulación: exigir revalidación online.

**5.7 Bandeja (`tray.rs`).** Mostrar tier y días restantes (`days_until_expiry`) en el tooltip/menú. Ya recibe `tray_license: Option<String>`.

**Criterio de aceptación Fase 5:** ver matriz de pruebas (sección 4).

---

### Fase 6 — `license-tool`

- Adaptar la emisión manual al mismo formato de `signed_license` de la Fase 3 (para licencias offline vendidas manualmente).
- Añadir opción `--expires-in-days` / `--expires-at` (hoy siempre `expires_at: None`).
- Documentar en `COMMANDS.md` el flujo: generar keypair → `wrangler secret put` → reemplazar `VENDOR_PUBLIC_KEY` → build.

---

### Fase 7 — Build, CI y empaquetado

1. `release.yml`: no requiere variables de licencia (la URL ya es default). Si se desea sobreescribir por entorno, usar `option_env!` con fallback a la URL de producción.
2. Confirmar que el MSI **no** incluye `vendor_secret_key.bin` (revisar `wix/main.wxs` y el contenido del `.msi` con `lessmsi` o similar).
3. Añadir en CI: `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`.
4. Publicar el Worker: `npx wrangler deploy` (después de Fases 1–3), y **antes** de distribuir el nuevo servidor.
5. Orden de despliegue para no romper clientes existentes: primero migración DB + Worker (compatible hacia atrás si mantiene `license`), luego servidor nuevo.

---

## 4. Matriz de pruebas de aceptación (obligatoria)

Ejecutar sobre un MSI de release instalado en una VM limpia de Windows 10/11 sin Rust.

| # | Escenario | Resultado esperado |
|---|---|---|
| T1 | Primer arranque con internet | Trial activo; `license.dat` creado; log sin `license_api_url vacío` |
| T2 | Cerrar y reabrir (con y sin internet) | Mismo `expires_at`; **no** se reinicia el trial |
| T3 | Primer arranque **sin** internet | Diálogo "Se requiere conexión…"; el servidor no arranca |
| T4 | Trial vencido (forzar `expires_at` pasado en D1 y `license.dat` expirado) | Diálogo de licencia expirada; sale con código 1 |
| T5 | Borrar `license.dat` con trial vencido en el Worker | El Worker responde 403 "Trial expired"; no hay trial nuevo |
| T6 | Revocar licencia en el panel | En ≤24 h (o al reiniciar) el servidor se apaga con mensaje |
| T7 | Editar `server.toml` con `trial_days = 9999` | Sin efecto sobre la expiración |
| T8 | Modificar 1 byte de `license.dat` | Rechazada por firma inválida |
| T9 | Copiar `license.dat` a otra PC | Rechazada por `hardware_hash` distinto |
| T10 | Retroceder reloj del sistema 10 días | Detectado (si se implementó 5.6) |
| T11 | Worker devuelve respuesta con firma inválida (man-in-the-middle simulado) | `validate_online` → `Rejected`; no se activa nada |
| T12 | Arranque en release | Cero ventanas negras visibles (`initdb`, `pg_ctl`, `postgres`, PowerShell) |
| T13 | 4 trials nuevos con hardware_hash distintos desde la misma IP en 24 h | El 4.º recibe 429 |
| T14 | `license.dat` de versión anterior (sin `expires_at`) | Deserializa (H16) y se re-activa/renueva sin pánico |
| T15 | Clave pagada activada 6 veces en la misma PC en 1 h | Sin bloqueo; `transfer_count` sin cambios |

Tests unitarios mínimos (Rust): expiración, `serde(default)`, verificación de firma buena/mala, `verify_signed_license` con bytes alterados, `hardware_id` con todas las consultas vacías → `Err`.
Tests del Worker (`vitest` + `@cloudflare/vitest-pool-workers` o scripts `curl`): T13, T15 y las verificaciones de la Fase 2.

---

## 5. Lista de verificación final (antes de vender)

- [ ] `vendor_secret_key.bin` nunca estuvo en git; solo existe como secret de Cloudflare y en un respaldo cifrado propio.
- [ ] `grep -rn "danger_accept_invalid_certs" crates/` sin resultados.
- [ ] `grep -rn "vendor_secret_key" crates/workshop-server/` sin resultados (el servidor no la necesita).
- [ ] `VENDOR_PUBLIC_KEY` no es todo ceros y corresponde al secret del Worker (test T-firma).
- [ ] `TRIAL_DAYS` del Worker en su valor de producción (no 0 ni 1 de pruebas).
- [ ] Índice único de trial por hardware presente en D1.
- [ ] `ADMIN_TOKEN` largo y aleatorio; panel detrás de Cloudflare Access (recomendado).
- [ ] Matriz T1–T15 ejecutada en VM limpia con el MSI final.
- [ ] Documentación actualizada en `COMMANDS.md` y `docs/PLAN-LICENSE-SERVICE.md` (el estado de las tareas 3 y 4 hoy figura como completado/pendiente de forma inconsistente).

---

## 6. Notas y límites conocidos

- Ninguna protección de licencias en software local es infalible: un atacante con acceso al binario puede parchearlo. Este plan sube el costo de eludirla (firma, expiración absoluta, revalidación) sin depender de secretos en el cliente.
- Cualquier cambio en el algoritmo de `hardware_hash` invalida activaciones existentes.
- Todo lo marcado **[VERIFICAR]** depende de código que no estaba a la vista al escribir este documento (`workshop-common/src/license.rs`, esquema real de D1, `wrangler.toml`, `wix/main.wxs`).
