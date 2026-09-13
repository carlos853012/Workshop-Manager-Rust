# Auditoría Integral — WorkshopManager

**Fecha:** 2026-09-13
**Alcance:** CSS/Design System, Arquitectura de Componentes, Seguridad, Calidad
**Skills aplicadas:** software-architect-senior, devsecops-senior, qa-senior

---

## Estado de las Reglas Existentes

### Lo que COBIERTAN bien (NO necesita cambios)

| Regla | Archivo | Hallazgo |
|-------|---------|----------|
| No `unsafe`, `unwrap()`, `expect()` | CONSTITUCION.md §3.1-3.2 | ✅ Cumplido — excepto `certificate.rs:140,148,156` |
| SQL parametrizado | CONSTITUCION.md §5.2 | ✅ Cumplido — 0 vulnerabilidades SQL injection |
| Validación de entradas | CONSTITUCION.md §3.3, §16.6 | ✅ Parcialmente — cubre campos principales, faltan longitudes máximas en campos opcionales |
| Secretos no hardcodeados | CONSTITUCION.md §3.4 | ⚠️ API key default hardcodeada en `config.rs:29` |
| Cambios mínimos y localizados | CONSTITUCION.md §2.3 | ✅ Cumplido |
| Workflow obligatorio de 5 pasos | CONSTITUCION.md §12, .clinerules, .cursorrules | ✅ Cumplido |

### Huecos IDENTIFICADOS (necesitan nuevas reglas)

| Área | Hueco | Impacto |
|------|-------|---------|
| **Design System CSS** | Sin reglas sobre uso de tokens, clases consistentes, wrapping de tablas | 19 clases huérfanas, 18 reglas muertas, 4 patrones de tabla diferentes |
| **Arquitectura de Componentes** | Sin reglas sobre Atomic Design, extracción de modals, patrones de error handling | modals inline sin validación, errores tragados silenciosamente |
| **Seguridad avanzada** | Sin reglas sobre Argon2id params, body limits, CORS, security headers | Parámetros por defecto insuficientes, sin límite de payload |
| **IDOR** | Sin regla explícita de workshop_id enforcement | `device_keys` no tiene workshop_id |
| **Frontend error handling** | Sin reglas sobre patrón de errores en UI | alert classes inconsistentes, Some modals ignoran errores |
| **Responsive** | Sin reglas sobre breakpoints o layouts adaptativos | 8 elementos sin responsive, modals con min-width fijo |

---

## Hallazgos por Categoría

### 🔴 SEGURIDAD ( Prioridad: ALTA )

- [ ] **S1 — Argon2id parameters** — `auth.rs:30` usa `Argon2::default()` (19MB memoria). OWASP recomienda 64MB mínimo para resistir ataques GPU. **Fix:** configurar `Params::new(65536, 3, 4, None)` explícitamente.
- [ ] **S2 — Secretos en disco sin protección real** — `secrets.rs:50-55` usa `attrib +H` en Windows (cualquiera los lee). **Fix:** usar DPAPI o Windows Credential Manager.
- [ ] **S3 — API key default hardcodeada** — `config.rs:29` `"dev-key-change-in-production"` en 4 ubicaciones. **Fix:** auto-generar en primer arranque y persistir.
- [ ] **S4 — Sin body size limit** — `main.rs:117` no configura `DefaultBodyLimitLayer`. Un atacante puede enviar payloads gigantes. **Fix:** agregar `.layer(DefaultBodyLimitLayer::new(10 * 1024 * 1024))`.
- [ ] **S5 — Device keys sin workshop_id** — `migrations/0004_device_keys.sql` no tiene `workshop_id`. IDOR entre talleres. **Fix:** agregar columna + filtrar queries.
- [ ] **S6 — CORS excesivamente permisivo** — `main.rs:121-130` permite `Any` methods/headers. **Fix:** restringir a GET, POST, PUT, DELETE + headers específicos.
- [ ] **S7 — Rate limiting solo en login** — `auth.rs:34` protege solo login. Falta en register y otros endpoints. **Fix:** agregar rate limiting por IP a nivel middleware.
- [ ] **S8 — status endpoint re-parsea JWT** — `auth.rs:199` duplica validación JWT. **Fix:** usar `Extension<AuthenticatedUser>` como otros endpoints.
- [ ] **S9 — Sin security headers** — Falta HSTS, X-Content-Type-Options, X-Frame-Options. **Fix:** agregar `SetResponseHeaderLayer`.
- [ ] **S10 — Sin token refresh/revocation** — JWT de 8 horas sin forma de invalidar. **Fix:** agregar token version en tabla users.
- [ ] **S11 — Email validation débil** — `auth.rs:264` solo verifica `@` y `.`. **Fix:** usar regex `^[^\s@]+@[^\s@]+\.[^\s@]+$`.
- [ ] **S12 — TLS key sin protección Windows** — `tls.rs:90-98` mismo issue que S2. **Fix:** DPAPI.
- [ ] **S13 — JWT secret entropy** — `secrets.rs:25` usa `thread_rng()` en vez de `OsRng`. **Fix:** generar 32 bytes con `OsRng` + hex encode.

### 🟠 CSS / DESIGN SYSTEM ( Prioridad: MEDIA-ALTA )

- [ ] **C1 — `.data-table-wrapper` sin CSS** — Clase usada en `data_table.rs:35` y `reports.rs:163` pero sin regla en `index.css`. **Fix:** agregar `overflow-x: auto`.
- [ ] **C2 — Filas de tabla demasiado anchas** — `padding: var(--space-md)` (16px) en todas las celdas. **Fix:** reducir a `padding: var(--space-sm) 0.75rem`.
- [ ] **C3 — 4 patrones de wrapping de tabla** — `DataTable` componente, `products-table`, bare `table`, `pos-cart-table`. **Fix:** estandarizar siempre `div.data-table-wrapper > table.data-table`.
- [ ] **C4 — 19 clases CSS huérfanas** — Usadas en .rs pero no definidas en CSS: `clickable-row`, `text-mono`, `text-xs`, `text-sm`, `text-lg`, `grid-3`, `justify-end`, `gap-2`, `mb-4`, `ml-sm`, `form-grid`, `sale-detail`, `detail-value`, `detail-section-title`, `detail-total`, `pos-scan-section`, `pos-cart-row`, `tooltip`, `tooltip-text`.
- [ ] **C5 — 18 reglas CSS muertas** — Definidas pero nunca referenciadas: `.container`, `.dashboard-toolbar`, `.pos-remove-btn`, `.flex-col`, `.justify-between`, `.h-full`, `.hidden`, `.alert-warning`, `.alert-info`, `.tabular-nums`, `.badge` + 5 variantes.
- [ ] **C6 — 10 violaciones de font-size tokens** — Valores hardcodeados (`0.9rem`, `0.75rem`, `0.85rem`, etc.) en vez de `var(--font-size-sm)`.
- [ ] **C7 — 7 colores hardcodeados** — `#ffffff`, `#000000`, `rgba(0,0,0,0.5)` en vez de CSS variables.
- [ ] **C8 — 2 border-radius sin token** — `border-radius: 50%` en vez de `var(--radius-full)`.
- [ ] **C9 — Focus ring duplicado 3 veces** — Mismo `box-shadow` copiado en 3 inputs. **Fix:** definir variable `--focus-shadow`.
- [ ] **C10 — Tooltip roto** — `opacity: 0` inline sin regla `:hover`. Nunca se muestra.
- [ ] **C11 — `ButtonVariant::Secondary` sin CSS** — Se renderiza como Ghost. **Fix:** agregar `.btn-secondary`.
- [ ] **C12 — `btn-icon btn-primary` sin hover** — Sin feedback visual en hover.
- [ ] **C13 — 8 elementos sin responsive** — `.pos-container`, `.pos-footer-row`, `.form-row-parts`, `.modal`, `.modal-lg`, `.header`, `.login-card`, `.sidebar-logo`.
- [ ] **C14 — Login card double-padding** — `login-card` + `card-body` suman padding excesivo.

### 🟠 ARQUITECTURA DE COMPONENTES ( Prioridad: MEDIA-ALTA )

- [ ] **A1 — `users.rs` modals inline** — Crea y edita usuarios con modals inline en la página. Todos los demás extraen a organisms. **Fix:** extraer a `user_form_modal.rs`.
- [ ] **A2 — `users.rs` sin validación** — Create modal envía sin verificar email/password. **Fix:** agregar `form_error` + validación antes de submit.
- [ ] **A3 — `repair_detail_modal.rs` traga errores** — 4 closures ignoran resultados de API. **Fix:** agregar error signal + mostrar al usuario.
- [ ] **A4 — `sale_detail_modal.rs` sin footer** — No hay botón "Cerrar". **Fix:** agregar `footer: rsx! { Button { variant: Ghost, "Cerrar" } }`.
- [ ] **A5 — `device_keys.rs` sin Card title** — Botón fuera del Card, empty state inconsistente. **Fix:** mover botón al `footer` del Card.
- [ ] **A6 — `service_certificate.rs` sin redirect en auth failure** — `on_download` no redirige a Login. **Fix:** agregar `navigator.push(Route::Login {})`.
- [ ] **A7 — Modals sin check `show` antes de fetch** — `repair_detail_modal.rs:62` y `certificate_detail_modal.rs:35` fetchan datos innecesariamente.
- [ ] **A8 — `stock_entry_modal.rs` guard redundante** — Verifica `show` antes y después del Modal.
- [ ] **A9 — `ConfirmModal` sin variant prop** — Siempre usa Danger, incluso para acciones no destructivas. **Fix:** agregar prop `variant`.
- [ ] **A10 — Cancel buttons inconsistentes** — `users.rs` no usa `class: "cancel-button"` como los demás.
- [ ] **A11 — Alert CSS class inconsistente** — `device_keys.rs` usa `alert-error` en vez de `alert-danger`.
- [ ] **A12 — Empty states inconsistentes** — 6 textos diferentes, 3 patrones CSS diferentes.

### 🟡 CALIDAD ( Prioridad: BAJA )

- [ ] **Q1 — 3 componentes muertos** — `Badge`, `Tooltip`, `FormGroup` definidos pero nunca usados.
- [ ] **Q2 — `eprintln!` en producción** — `repair_detail_modal.rs:101`.
- [ ] **Q3 — Signal mutabilidad inconsistente** — `let mut error` vs `let error` en 16 archivos.
- [ ] **Q4 — `refresh` naming inconsistente** — `device_keys.rs` usa `refresh_token: u32` vs `refresh: i32`.
- [ ] **Q5 — Naming inconsistente** — `evt` vs `event` (42 vs 3), `.clone()` vs sin clone.
- [ ] **Q6 — File naming** — `connection_settings.rs` no sigue patrón `*_modal.rs`.
- [ ] **Q7 — 2 dependencias sin usar** — `tracing-appender`, `obfstr` en server Cargo.toml.
- [ ] **Q8 — `certificate.rs` expect() en producción** — Líneas 140, 148, 156 violan CONSTITUCION.md §3.2.

---

## Plan de Implementación

### Fase 1 — Seguridad (antes de producción)
| # | Tarea | Archivos | Estado |
|---|-------|----------|--------|
| 1.1 | Hardening Argon2id (S1) | `auth.rs` | ⬜ |
| 1.2 | Auto-generar API key (S3) | `config.rs`, `secrets.rs` | ⬜ |
| 1.3 | DefaultBodyLimitLayer (S4) | `main.rs` | ⬜ |
| 1.4 | workshop_id en device_keys (S5) | `migrations/`, `device_key.rs`, `routes/device_keys.rs` | ⬜ |
| 1.5 | Restringir CORS (S6) | `main.rs` | ⬜ |
| 1.6 | Security headers (S9) | `main.rs` | ⬜ |
| 1.7 | Rate limiting extendido (S7) | `main.rs`, `middleware.rs` | ⬜ |
| 1.8 | Refactor status endpoint (S8) | `routes/auth.rs` | ⬜ |

### Fase 2 — CSS Design System
| # | Tarea | Archivos | Estado |
|---|-------|----------|--------|
| 2.1 | Definir `.data-table-wrapper` + compactar filas (C1, C2) | `index.css` | ⬜ |
| 2.2 | Unificar wrapping de tablas (C3) | `products.rs`, `device_keys.rs`, `sale_detail_modal.rs`, `parts_tab.rs`, `parts_tab_cert.rs` | ⬜ |
| 2.3 | Agregar 19 clases CSS huérfanas (C4) | `index.css` | ⬜ |
| 2.4 | Eliminar 18 reglas CSS muertas (C5) | `index.css` | ⬜ |
| 2.5 | Reemplazar font-size hardcodeados por tokens (C6) | `index.css` | ⬜ |
| 2.6 | Reemplazar colores hardcodeados por variables (C7) | `index.css` | ⬜ |
| 2.7 | Fix border-radius tokens (C8) | `index.css` | ⬜ |
| 2.8 | Tokenizar focus ring (C9) | `index.css` | ⬜ |
| 2.9 | Fix tooltip (C10) | `tooltip.rs`, `index.css` | ⬜ |
| 2.10 | Fix `btn-secondary` y `btn-icon btn-primary` (C11, C12) | `index.css` | ⬜ |
| 2.11 | Agregar responsive a 8 elementos (C13) | `index.css` | ⬜ |
| 2.12 | Fix login card padding (C14) | `index.css` | ⬜ |

### Fase 3 — Arquitectura de Componentes
| # | Tarea | Archivos | Estado |
|---|-------|----------|--------|
| 3.1 | Extraer `users.rs` modals (A1) | `users.rs`, nuevo `user_form_modal.rs` | ⬜ |
| 3.2 | Agregar validación a `users.rs` (A2) | `users.rs` o `user_form_modal.rs` | ⬜ |
| 3.3 | Fix error handling en `repair_detail_modal` (A3) | `repair_detail_modal.rs` | ⬜ |
| 3.4 | Agregar footer a `sale_detail_modal` (A4) | `sale_detail_modal.rs` | ⬜ |
| 3.5 | Fix `device_keys.rs` layout (A5) | `device_keys.rs` | ⬜ |
| 3.6 | Fix auth redirect en `service_certificate` (A6) | `service_certificate.rs` | ⬜ |
| 3.7 | Agregar check `show` antes de fetch (A7) | `repair_detail_modal.rs`, `certificate_detail_modal.rs` | ⬜ |
| 3.8 | Limpiar guard redundante (A8) | `stock_entry_modal.rs` | ⬜ |
| 3.9 | Agregar variant a `ConfirmModal` (A9) | `confirm_modal.rs` | ⬜ |
| 3.10 | Unificar cancel buttons (A10) | `users.rs` | ⬜ |
| 3.11 | Unificar alert classes (A11) | `device_keys.rs`, `pos.rs`, `sale_detail_modal.rs` | ⬜ |
| 3.12 | Unificar empty states (A12) | Varios | ⬜ |

### Fase 4 — Limpieza
| # | Tarea | Archivos | Estado |
|---|-------|----------|--------|
| 4.1 | Eliminar componentes muertos (Q1) | `badge.rs`, `tooltip.rs`, `form_group.rs` | ⬜ |
| 4.2 | Eliminar `eprintln!` (Q2) | `repair_detail_modal.rs` | ⬜ |
| 4.3 | Unificar signal mutabilidad (Q3) | 16 archivos | ⬜ |
| 4.4 | Unificar refresh naming (Q4) | `device_keys.rs` | ⬜ |
| 4.5 | Unificar naming conventions (Q5) | Varios | ⬜ |
| 4.6 | Renombrar archivos (Q6) | `connection_settings.rs` | ⬜ |
| 4.7 | Eliminar dependencias sin usar (Q7) | `Cargo.toml` | ⬜ |
| 4.8 | Reemplazar `expect()` en producción (Q8) | `certificate.rs` | ⬜ |

---

## Estadísticas Finales

| Categoría | Total | Crítico/Alto | Medio | Bajo |
|-----------|-------|--------------|-------|------|
| Seguridad | 13 | 2 | 7 | 4 |
| CSS/Design | 14 | 0 | 14 | 0 |
| Arquitectura | 12 | 2 | 7 | 3 |
| Calidad | 8 | 0 | 0 | 8 |
| **Total** | **47** | **4** | **28** | **15** |
