# Plan — Límites de módulos por tipo de licencia

Objetivo: aplicar los `Feature`/`LicenseTier` ya existentes (hoy definidos pero **sin uso**)
para que el acceso a módulos dependa del tier de licencia.

## Principios

- **Servidor = fuente de verdad**: valida contra el `License` verificado en memoria (`AppState.license`).
  Un `403` no es evitable desde el cliente.
- **Viewer = UX**: oculta/bloquea visualmente. No es seguridad por sí solo.
- Rol (`Admin/Mechanic/Seller`) y licencia (`Feature`) son **ortogonales**: rol = *quién*, licencia = *qué*.
- Ambos crates dependen de `workshop-common`, así que comparten `Feature` y `LicenseTier` sin duplicar lógica.
- **No cambia el endpoint de licencia**: el viewer ya recibe `tier: String`; solo lo parsea.

## Mapeo corregido (feature → módulo real)

| Grupo de rutas / página | Feature exigida | Notas |
|---|---|---|
| `/api/analytics/*` (Dashboard, KPIs, revenue) | **base** (`Dashboard`) | alimenta el Dashboard base — NO gatear |
| `/api/products` | `Inventory` | base |
| `/api/sales` | `Sales` | base |
| `/api/repairs` | `Repairs` | base |
| `/api/suppliers` | `Suppliers` | base |
| `/api/reports/*` + "Reportes" + "Certificado Servicios" | `ClientHistory` (sentinel del tier Reports) | Reportes y Certificado se desbloquean juntos |
| `/api/audit`, `/users`, `/device-keys` | por **rol admin** | sin cambio |
| `AdvancedAnalytics`, `ExcelExport`, `RestApi`, `Webhooks`, `Integrations`, `MultiViewer`, `AutoBackup`, `MultiWorkshop` | **sin gate hoy** | no tienen endpoint/página todavía |

> `ClientHistory` se usa como sentinel del grupo Reports: `LicenseTier::features()` es acumulativo,
> así que `has_feature(ClientHistory)` es `true` exactamente para el tier Reports y superiores.

## Alcance

- Server + Viewer.
- Módulo no incluido → **oculto del menú + pantalla bloqueada** si se navega directo.
- Solo **módulos completos** (no acciones puntuales dentro de un módulo).

---

## Fase A — Common (helper, aditivo)

### T1. `FromStr` para `LicenseTier` — `crates/workshop-common/src/features.rs`
- Implementar `std::str::FromStr` aceptando formas de `Display`
  (`"Trial"`, `"Base"`, `"Reports"`, `"Advanced"`, `"API"`) y de serde (`"Api"`), case-insensitive.
- Tests unitarios (parseo correcto + error para desconocido).
- **Verificar:** `cargo test -p workshop-common`

## Fase B — Server (enforcement)

### T2. Helper puro `license_allows` — `crates/workshop-server/src/middleware.rs`
- `license_allows(license: Option<&License>, feature: &Feature) -> bool`.
- Si `None`/sin licencia → features de `Trial`.
- **Verificar:** `cargo test -p workshop-server`

### T3. Middleware `require_feature(feature)` — `middleware.rs`
- Closure sobre `from_fn_with_state` que lee `state.license`, llama `license_allows`
  y devuelve `403 FORBIDDEN` si no está permitida.
- **Verificar:** `cargo check -p workshop-server`

### T4. Aplicar gate al grupo `/reports` — `crates/workshop-server/src/routes/mod.rs`
- Cambiar firma `protected_routes()` → `protected_routes(state: AppState)`.
- `.route_layer(require_feature(Feature::ClientHistory))` sobre `.nest("/reports", ...)`.
- Dejar sin gate: `/analytics`, `/products`, `/sales`, `/repairs`, `/suppliers`, `/config`, `/auth`.
- **Verificar:** `cargo check -p workshop-server`

### T5. Actualizar llamada — `crates/workshop-server/src/main.rs`
- `routes::protected_routes()` → `routes::protected_routes(state.clone())`.
- **Verificar:** `cargo build -p workshop-server`

### T6. Tests del gate
- `license_allows(None, ClientHistory) == false`; con tier `Reports` → `true`; con `Trial` → `false`.
- **Verificar:** `cargo test -p workshop-server`, `cargo clippy -p workshop-server -- -D warnings`

## Fase C — Viewer (UX)

### T7. `AuthState::has_feature` — `crates/workshop-viewer/src/app_state.rs`
- Parsear `tier` almacenado a `LicenseTier` (usando `FromStr` de T1).
- `has_feature(&Feature) -> bool`; si `license_info` es `None` → solo base (DLC oculto).
- **Verificar:** `cargo check -p workshop-viewer`

### T8. Filtrar menú lateral — `crates/workshop-viewer/src/pages/layout.rs`
- Envolver "Reportes" y "Certificado Servicios" en `if auth.has_feature(Feature::ClientHistory)`.
- **Verificar:** `cargo check -p workshop-viewer`

### T9. Organismo `UpgradeRequired` — `crates/workshop-viewer/src/components/organisms/upgrade_required.rs`
- Mensaje y CTA que abre `license_activation_modal` (ya existe).
- Registrar el módulo donde corresponda.
- **Verificar:** `cargo check -p workshop-viewer`

### T10. Gate por ruta en `AppShell` — `pages/layout.rs`
- Helper `required_feature(&Route) -> Option<Feature>`
  (Reportes → `ClientHistory`, Certificado → `PdfReports`; resto `None`).
- Si falta la feature, renderizar `UpgradeRequired` en lugar de `{children}`.
- **Verificar:** `cargo check -p workshop-viewer`

### T11. i18n + validación global
- Textos del bloqueo en `crates/workshop-viewer/src/i18n.rs`.
- **Verificar:** `cargo clippy --workspace -- -D warnings`, `cargo fmt --all --check`, `cargo test --workspace`

## Fase D — Smoke manual

### T12. Verificación dual-tier
- Con `Trial`: Reportes/Certificado ocultos y `/api/reports/*` → `403`.
- Con tier `Reports`: visibles y `200`.
- Documentar pasos en la respuesta. **Sin commit** hasta aprobación explícita.

---

## Riesgos y notas

- **Cambio de firma** `protected_routes()` (T4/T5): toca solo `mod.rs` + `main.rs`. Localizado, reversible.
- **Sin licencia = Trial**: no rompe instalaciones nuevas.
- **CSS**: no requiere cambios (reuso de clases existentes). Recordar que cambios de CSS exigen rebuild del viewer.
- **Mapeo desde endpoints/páginas reales**, no desde nombres aspiracionales del enum (evita romper el Dashboard).
- **Sin commits** hasta permiso explícito (regla de `AGENTS.md`).
