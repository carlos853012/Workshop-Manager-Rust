# Plan v0.2.0 — WorkshopManager

**Fecha:** 2026-09-19
**Autor:** Auditoría automática (seguridad + código + QA)
**Estado:** En Progreso (FASE 1 y 2 completadas)

---

## Resumen Ejecutivo

Auditoría completa de WorkshopManager v0.1.0 revela **19 hallazgos de seguridad**, **15 de calidad de código**, y **gaps críticos de testing**. El plan prioriza correcciones de seguridad y testing de integración antes de nuevas features.

| Fase | Foco | Días Est. | Prioridad |
|------|------|-----------|-----------|
| 1 | Seguridad Crítica | 2-3 | **MUST** |
| 2 | Calidad de Código | 2-3 | **SHOULD** |
| 3 | Testing Crítico | 3-4 | **MUST** |
| 4 | Seguridad Media | 1-2 | **SHOULD** |
| 5 | Dependencias Vulnerables | 2-3 | **MUST** |
| **Total** | | **10-15** | |

---

## FASE 1 — Seguridad Crítica ✅ COMPLETADA

### C-1: SQL Injection en analytics.rs ✅
**Archivo:** `crates/workshop-server/src/routes/analytics.rs:201-215`
**Problema:** `format!()` interpola `group_expr` directamente en SQL.
**Remediación:** Reemplazado por `match` con expresiones constantes.
**Estado:** Resuelto — commit `7632dea`

### C-2: TLS deshabilitado en validación de licencia ✅
**Archivo:** `crates/workshop-server/src/license.rs:83`
**Problema:** `danger_accept_invalid_certs(true)` permite MITM.
**Remediación:** Eliminado `danger_accept_invalid_certs`.
**Estado:** Resuelto — commit `7632dea`

### C-3: API key hardcoded en config de ejemplo ✅
**Archivo:** `config/server.toml:4`
**Problema:** `api_key = "dev-key-change-in-production"` commiteado.
**Remediación:** Valor removido, auto-generación en config.rs.
**Estado:** Resuelto — commit `7632dea`

### C-4: unwrap() en código de producción (3 instancias) ✅
**Archivos:**
- `routes/repairs.rs:581` → `ok_or_else()`
- `routes/analytics.rs:196-199` → `ok_or_else()`
- `components/organisms/user_form_modal.rs:79` → `let Some() else`

**Estado:** Resuelto — commit `7632dea`

### C-5: Rate limiter memory leak ✅
**Archivo:** `crates/workshop-server/src/rate_limiter.rs`
**Problema:** Sin limpieza periódica de entradas expiradas.
**Remediación:** Agregado cleanup cada 60 segundos + campo `last_cleanup`.
**Estado:** Resuelto — commit `7632dea`

---

## FASE 2 — Calidad de Código ✅ PARCIALMENTE COMPLETADA

### Q-1: Email validation inconsistente (5 archivos) ✅
**Archivos:** auth.rs, users.rs, sales.rs, repairs.rs, suppliers.rs
**Problema:** Cada archivo usa regex diferente.
**Remediación:** Creado `validation.rs` con `validate_email()` compartido.
**Estado:** Resuelto — commit `7632dea`

### Q-2: hide_password_hash duplicado ✅
**Archivos:** auth.rs:278, users.rs:315
**Problema:** Función idéntica en 2 archivos.
**Remediación:** Extraído a `validation.rs`.
**Estado:** Resuelto — commit `7632dea`

### Q-3: IVA 19% hardcodeado en POS ⏳ PENDIENTE
**Archivo:** `crates/workshop-viewer/src/pages/pos.rs`
**Problema:** IVA rate es configurable en server.toml pero viewer muestra fijo 19%.
**Remediación:** Agregar rate al DashboardResponse o endpoint de config pública.
**Esfuerzo:** Medio

### Q-4: Regex recompilado por request ✅
**Archivo:** `routes/auth.rs:291`
**Problema:** `Regex::new()` se ejecuta en cada login.
**Remediación:** Usar `once_cell::sync::Lazy` en `validation.rs`.
**Estado:** Resuelto — commit `7632dea`

### Q-5: Secrets clonados por request ⏳ PENDIENTE
**Archivo:** `crates/workshop-server/src/main.rs`
**Problema:** `Secrets` (String + Vec<u8>) se clona en cada request.
**Remediación:** Envolver en `Arc`.
**Esfuerzo:** Bajo

---

## FASE 3 — Testing Crítico (3-4 días)

### T-1: Tests de integración HTTP (SIN DB)
**Descripción:** Tests que envían requests reales a handlers mockgeando la DB.
**Tests a crear:**
- `auth_flows.rs` — POST /api/auth/login → 200 + JWT
- `protected_routes.rs` — GET /api/products sin token → 401
- `admin_only.rs` — GET /api/users como Seller → 403
- `error_responses.rs` — POST body inválido → 400
- `rate_limiting.rs` — 5 intentos fallidos → 429

**Herramienta:** `axum::test::TestClient` o `tower::ServiceExt::oneshot()`
**Esfuerzo:** Alto

### T-2: Tests de RBAC
**Descripción:** Verificar que Admin, Seller, Mechanic tienen los permisos correctos.
**Tests a crear:**
- Admin puede acceder a todos los endpoints
- Seller no puede acceder a /api/users
- Mechanic no puede crear productos
- Token expirado → 401
- Token con rol inválido → 403

**Esfuerzo:** Medio

### T-3: Tests de CRUD con DB embebida
**Descripción:** Operaciones completas Create→Read→Update→Delete.
**Tests a crear:**
- `product_crud.rs` — Crear, leer, actualizar, borrar producto
- `sale_with_stock.rs` — Venta descuenta stock, stock insuficiente → error
- `repair_lifecycle.rs` — Pending → InProgress → Completed
- `user_registration.rs` — Register → login → usar token

**Esfuerzo:** Alto

### T-4: Fix tests flaky
**Archivos:**
- `crypto.rs:88-111` — Usar `tempfile::tempdir()` en vez de directorio hardcodeado
- `tls.rs:144-158` — Usar `tempfile::tempdir()` para auto-cleanup
- `backup.rs:93-112` — Usar timestamps explícitos en vez de `thread::sleep`
- `hardware.rs:101-116` — Marcar `#[ignore]` o condicionar a CI

**Esfuerzo:** Bajo

### T-5: Tests de concurrencia
**Descripción:** Verificar que no hay race conditions.
**Tests a crear:**
- Rate limiter con múltiples tasks concurrentes
- Stock deduction concurrente (deadlock prevention)
- Login concurrente con el mismo email

**Esfuerzo:** Medio

---

## FASE 4 — Seguridad Media (1-2 días)

### S-1: JWT sin refresh token
**Archivo:** `crates/workshop-server/src/auth.rs`
**Problema:** TTL 24h sin mecanismo de refresh o revocación.
**Remediación:** Implementar refresh tokens con TTL corto (15min-1h).
**Esfuerzo:** Alto

### S-2: Secretos como archivos plaintext
**Archivo:** `crates/workshop-server/src/secrets.rs`
**Problema:** `.jwt_secret` y `.crypto_key` sin cifrar en disco.
**Remediación:** Usar OS keychain (DPAPI/Keychain/keyctl).
**Esfuerzo:** Alto

### S-3: Sin CSP header
**Archivo:** `crates/workshop-server/src/main.rs:235`
**Problema:** Falta Content-Security-Policy.
**Remediación:** Agregar header CSP apropiado.
**Esfuerzo:** Bajo

### S-4: License key placeholder
**Archivo:** `crates/workshop-server/src/license.rs:10-13`
**Problema:** VENDOR_PUBLIC_KEY es todos `0x00`.
**Remediación:** Rechazar si no se generaron claves reales.
**Esfuerzo:** Bajo

---

## FASE 5 — Dependencias Vulnerables (2-3 días)

**Fuente:** `cargo audit` en CI pipeline (2026-09-19)

### D-1: sqlx 0.7.4 → 0.8.1+ (MEDIUM)
**CVE:** RUSTSEC-2024-0363
**Problema:** Binary Protocol Misinterpretation caused by Truncating or Overflowing Casts
**Remediación:** Upgrade sqlx a 0.8.x (breaking changes en API)
**Esfuerzo:** Alto
**Notas:** Requiere cambios en queries y tipos. sqlx 0.8 tiene nuevos tipos `PgPool` y cambios en `FromRow`.
**Archivos afectados:** `workshop-common`, `workshop-server`

### D-2: rustls 0.23.43 → 0.23.45+ (MEDIUM)
**CVE:** RUSTSEC-2026-0285
**Problema:** TLS 1.3 handshake messages incorrectly accepted across encryption level boundaries
**Remediación:** Upgrade rustls (patch update)
**Esfuerzo:** Bajo
**Notas:** Dependencia transitive via axum-server, reqwest, sqlx-core. Actualizar Cargo.lock.

### D-3: lopdf 0.26.0 → 0.42.0+ (HIGH)
**CVE:** RUSTSEC-2026-0187
**Problema:** Stack overflow via deeply nested PDF objects
**Remediación:** **NO DISPONIBLE** — genpdf 0.2.0 (última versión) usa lopdf 0.26.0
**Esfuerzo:** N/A
**Notas:** Dependencia transitive de genpdf → printpdf → lopdf. Esperar update de genpdf o migrar a otro generador PDF.

### D-4: webbrowser 0.8.15 → 1.2.2+ (MEDIUM)
**CVE:** RUSTSEC-2026-0257
**Problema:** Unix `BROWSER` handling allows browser argument injection
**Remediación:** **NO DISPONIBLE** — dioxus-desktop 0.6.3 usa webbrowser 0.8.x
**Esfuerzo:** N/A
**Notas:** Dependencia transitive de dioxus-desktop. Esperar update de dioxus.

### D-5: rsa 0.9.10 (MEDIUM)
**CVE:** RUSTSEC-2023-0071
**Problema:** Marvin Attack: potential key recovery through timing sidechannels
**Remediación:** **NO DISPONIBLE** — no hay fix disponible
**Esfuerzo:** N/A
**Notas:** Dependencia transitive. No se usa directamente para operaciones críticas.

### Resumen Dependencias

| Crate | Severidad | Fix Disponible | Acción |
|-------|-----------|----------------|--------|
| sqlx | MEDIUM | ✅ 0.8.1+ | Upgrade (breaking changes) |
| rustls | MEDIUM | ✅ 0.23.45+ | Upgrade (patch) |
| lopdf | HIGH | ❌ | Esperar genpdf |
| webbrowser | MEDIUM | ❌ | Esperar dioxus |
| rsa | MEDIUM | ❌ | Aceptar riesgo |

---

## Hallazgos de Testing (Resumen)

### Cobertura Actual
| Crate | Tests | Foco |
|-------|-------|------|
| workshop-common | ~45 | Money, patente, license, features, hardware |
| workshop-server | ~40 | JWT, crypto, middleware, rate limiter, TLS |
| workshop-viewer | ~6 | API parsing, charts |
| license-tool | 0 | **Nada** |

### Gaps Críticos
- **0 tests de integración HTTP** — ningún endpoint se testea real
- **0 tests de RBAC** — no se verifica Seller vs Admin
- **0 tests de CRUD** — solo validación de inputs
- **0 tests de DB** — queries, transacciones, migraciones
- **0 tests de concurrencia** — rate limiter, stock deduction

---

## Priorización Final

| # | ID | Fase | Esfuerzo | Impacto | Estado |
|---|-----|------|----------|---------|--------|
| 1 | C-1 | Seguridad | Medio | Elimina SQL injection | ✅ Completado |
| 2 | C-2 | Seguridad | Bajo | Elimina MITM en licencia | ✅ Completado |
| 3 | C-3 | Seguridad | Bajo | Elimina secreto hardcoded | ✅ Completado |
| 4 | C-4 | Seguridad | Bajo | Cumple reglas constitucionales | ✅ Completado |
| 5 | C-5 | Seguridad | Medio | Previene DoS y memory leak | ✅ Completado |
| 6 | D-2 | Dependencias | Bajo | Fix TLS handshake vulnerability | Pendiente |
| 7 | D-1 | Dependencias | Alto | Fix SQL binary protocol | Pendiente |
| 8 | T-1 | Testing | Alto | Cubre auth, JWT, middleware, RBAC | Pendiente |
| 9 | T-2 | Testing | Medio | Verifica control de acceso | Pendiente |
| 10 | Q-1 | Calidad | Bajo | Consistencia en validación | ✅ Completado |
| 11 | T-4 | Testing | Bajo | Elimina tests flaky | Pendiente |
| 12 | Q-3 | Calidad | Medio | IVA configurable en UI | Pendiente |

---

## Controles que SÍ están bien (v0.1.0)

- ✅ SQL parametrizado en 95%+ de queries
- ✅ Argon2id con parámetros OWASP (64MB, 3 iter, 4 parallelism)
- ✅ AES-256-GCM para cifrado con nonces aleatorios
- ✅ Rate limiting en login (5 intentos / 300s)
- ✅ Audit log para operaciones CRUD
- ✅ Password hash redactado en respuestas
- ✅ Transacciones para operaciones que modifican stock
- ✅ Deadlock prevention en ventas (ordenamiento por product_id)
- ✅ Validación de inputs en todos los endpoints
- ✅ Role-based access control (admin/mechanic/seller)
- ✅ TLS con certificados autofirmados generados con rcgen
- ✅ Body limit de 10MB
- ✅ Error sanitization en responses
- ✅ Headers: HSTS, X-Content-Type-Options, X-Frame-Options
- ✅ CORS restringido a localhost

---

## Definición de Hecho

La versión v0.2.0 estará lista cuando:

1. **Todos los hallazgos críticos (C-1 a C-5) estén resueltos**
2. **Dependencias vulnerables (D-1, D-2) actualizadas**
3. **Tests de integración HTTP cubran al menos:**
   - Login exitoso y fallido
   - Acceso a endpoint protegido con/without token
   - RBAC (Admin vs Seller vs Mechanic)
   - Error responses (400, 401, 403, 404)
4. **Tests de CRUD cubran al menos:**
   - Product create→read→update→delete
   - Sale con stock deduction
   - Repair lifecycle
5. **Ningún test sea flaky**
6. **Clippy limpio, 0 warnings**
7. **`cargo audit` sin vulnerabilidades high/medium resolubles**
8. **Documentación actualizada**

---

## Referencias

- [OWASP Top 10 (2021)](https://owasp.org/Top10/)
- [Constitutional Rules](../CONSTITUCION.md)
- [AGENTS.md](../AGENTS.md)
- [CHANGELOG.md](CHANGELOG.md)
