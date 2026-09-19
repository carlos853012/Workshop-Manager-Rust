# Plan v0.2.0 — WorkshopManager

**Fecha:** 2026-09-19
**Autor:** Auditoría automática (seguridad + código + QA)
**Estado:** Planificación

---

## Resumen Ejecutivo

Auditoría completa de WorkshopManager v0.1.0 revela **19 hallazgos de seguridad**, **15 de calidad de código**, y **gaps críticos de testing**. El plan prioriza correcciones de seguridad y testing de integración antes de nuevas features.

| Fase | Foco | Días Est. | Prioridad |
|------|------|-----------|-----------|
| 1 | Seguridad Crítica | 2-3 | **MUST** |
| 2 | Calidad de Código | 2-3 | **SHOULD** |
| 3 | Testing Crítico | 3-4 | **MUST** |
| 4 | Seguridad Media | 1-2 | **SHOULD** |
| **Total** | | **8-12** | |

---

## FASE 1 — Seguridad Crítica (2-3 días)

### C-1: SQL Injection en analytics.rs
**Archivo:** `crates/workshop-server/src/routes/analytics.rs:201-215`
**Problema:** `format!()` interpola `group_expr` directamente en SQL.
**Remediación:** Usar `match` con expresiones constantes o reestructurar queries.
**Esfuerzo:** Medio

### C-2: TLS deshabilitado en validación de licencia
**Archivo:** `crates/workshop-server/src/license.rs:83`
**Problema:** `danger_accept_invalid_certs(true)` permite MITM.
**Remediación:** Eliminar o usar feature flag para development.
**Esfuerzo:** Bajo

### C-3: API key hardcoded en config de ejemplo
**Archivo:** `config/server.toml:4`
**Problema:** `api_key = "dev-key-change-in-production"` commiteado.
**Remediación:** Generar siempre al primer arranque, remover valor hardcoded.
**Esfuerzo:** Bajo

### C-4: unwrap() en código de producción (3 instancias)
**Archivos:**
- `routes/repairs.rs:581`
- `routes/analytics.rs:196-199`
- `components/organisms/user_form_modal.rs:79`

**Remediación:** Reemplazar con `ok_or()` o `map_err()`.
**Esfuerzo:** Bajo

### C-5: Rate limiter memory leak
**Archivo:** `crates/workshop-server/src/rate_limiter.rs`
**Problema:** Sin limpieza periódica de entradas expiradas.
**Remediación:** Agregar tokio task para limpieza periódica + límite de entries.
**Esfuerzo:** Medio

---

## FASE 2 — Calidad de Código (2-3 días)

### Q-1: Email validation inconsistente (5 archivos)
**Archivos:** auth.rs, users.rs, sales.rs, repairs.rs, suppliers.rs
**Problema:** Cada archivo usa regex diferente.
**Remediación:** Crear `fn validate_email()` en módulo compartido.
**Esfuerzo:** Bajo

### Q-2: hide_password_hash duplicado
**Archivos:** auth.rs:278, users.rs:315
**Problema:** Función idéntica en 2 archivos.
**Remediación:** Extraer a módulo compartido.
**Esfuerzo:** Bajo

### Q-3: IVA 19% hardcodeado en POS
**Archivo:** `crates/workshop-viewer/src/pages/pos.rs`
**Problema:** IVA rate es configurable en server.toml pero viewer muestra fijo 19%.
**Remediación:** Agregar rate al DashboardResponse o endpoint de config pública.
**Esfuerzo:** Medio

### Q-4: Regex recompilado por request
**Archivo:** `routes/auth.rs:291`
**Problema:** `Regex::new()` se ejecuta en cada login.
**Remediación:** Usar `once_cell::sync::Lazy`.
**Esfuerzo:** Bajo

### Q-5: Secrets clonados por request
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

| # | ID | Fase | Esfuerzo | Impacto |
|---|-----|------|----------|---------|
| 1 | C-1 | Seguridad | Medio | Elimina SQL injection |
| 2 | C-2 | Seguridad | Bajo | Elimina MITM en licencia |
| 3 | C-3 | Seguridad | Bajo | Elimina secreto hardcoded |
| 4 | C-4 | Seguridad | Bajo | Cumple reglas constitucionales |
| 5 | C-5 | Seguridad | Medio | Previene DoS y memory leak |
| 6 | T-1 | Testing | Alto | Cubre auth, JWT, middleware, RBAC |
| 7 | T-2 | Testing | Medio | Verifica control de acceso |
| 8 | Q-1 | Calidad | Bajo | Consistencia en validación |
| 9 | T-4 | Testing | Bajo | Elimina tests flaky |
| 10 | Q-3 | Calidad | Medio | IVA configurable en UI |

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
2. **Tests de integración HTTP cubran al menos:**
   - Login exitoso y fallido
   - Acceso a endpoint protegido con/without token
   - RBAC (Admin vs Seller vs Mechanic)
   - Error responses (400, 401, 403, 404)
3. **Tests de CRUD cubran al menos:**
   - Product create→read→update→delete
   - Sale con stock deduction
   - Repair lifecycle
4. **Ningún test sea flaky**
5. **Clippy limpio, 0 warnings**
6. **Documentación actualizada**

---

## Referencias

- [OWASP Top 10 (2021)](https://owasp.org/Top10/)
- [Constitutional Rules](../CONSTITUCION.md)
- [AGENTS.md](../AGENTS.md)
- [CHANGELOG.md](CHANGELOG.md)
