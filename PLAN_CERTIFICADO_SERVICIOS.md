# Plan de Implementación — Certificado de Servicios

**Fecha inicio:** 2026-09-09
**Estado:** COMPLETADO

---

## Fase A: Base de datos y tipos compartidos

- [x] **A1** — Migración `0009_repair_parts_product_fk.sql`
  - Archivo: `crates/workshop-server/migrations/0009_repair_parts_product_fk.sql`
  - Descripción: Agregar `product_id UUID REFERENCES products(id)` a `repair_parts`
  - Verificación: SQL revisado manualmente

- [x] **A2** — Actualizar struct `RepairPart` en common
  - Archivo: `crates/workshop-common/src/lib.rs`
  - Descripción: Agregar `product_id: Option<uuid::Uuid>` al struct `RepairPart`
  - Verificación: `cargo check -p workshop-common`

- [x] **A3** — Agregar DTOs del certificado
  - Archivo: `crates/workshop-common/src/dto.rs`
  - Descripción: Agregar DTOs: `ClientSearchResult`, `VehicleSummary`, `ServiceCertificate`, `WorkshopInfo`, `ClientInfo`, `VehicleInfo`, `ServiceEntry`, `PartEntry`. Agregar `product_id` a `RepairPartResponse` y `AddRepairPartRequest`
  - Verificación: `cargo check -p workshop-common`

---

## Fase B: Backend — endpoints

- [x] **B1** — Endpoint búsqueda de cliente
  - Archivo: `crates/workshop-server/src/routes/reports.rs`, `mod.rs`
  - Descripción: `GET /api/reports/client-search?q=...` — busca por patente, nombre o email
  - Verificación: `cargo check -p workshop-server`

- [x] **B2** — Endpoint datos del certificado
  - Archivo: `crates/workshop-server/src/routes/reports.rs`, `mod.rs`
  - Descripción: `GET /api/reports/client-certificate?email=...&plate=...` — retorna `ServiceCertificate`
  - Verificación: `cargo check -p workshop-server`

- [x] **B3** — Actualizar `add_repair_part` con `product_id`
  - Archivo: `crates/workshop-server/src/routes/repairs.rs`
  - Descripción: Aceptar `product_id` opcional en INSERT de repair_parts
  - Verificación: `cargo check -p workshop-server`

---

## Fase C: Backend — generación PDF

- [x] **C1** — Dependencia genpdf
  - Archivo: `crates/workshop-server/Cargo.toml`
  - Descripción: Agregar `genpdf = "0.2"`
  - Verificación: `cargo check -p workshop-server`

- [x] **C2** — Módulo certificate.rs
  - Archivo: `crates/workshop-server/src/certificate.rs`
  - Descripción: Función `generate_certificate_pdf()` con header taller, cliente/vehículo, tabla servicios, tabla repuestos (sin costos), footer fecha. Usa LiberationSans del sistema vía `C:\Windows\Fonts\`.
  - Verificación: `cargo check -p workshop-server`

- [x] **C3** — Endpoint descarga PDF
  - Archivo: `crates/workshop-server/src/routes/reports.rs`, `mod.rs`, `main.rs`
  - Descripción: `GET /api/reports/client-certificate.pdf?email=...&plate=...` retorna `application/pdf`
  - Verificación: `cargo build -p workshop-server`

---

## Fase D: Frontend — UI

- [x] **D1** — Métodos en API client
  - Archivo: `crates/workshop-viewer/src/api.rs`
  - Descripción: `client_search()` y `download_certificate()` con manejo de errores HTTP
  - Verificación: `cargo check -p workshop-viewer`

- [x] **D2** — Página service_certificate.rs
  - Archivo: `crates/workshop-viewer/src/pages/service_certificate.rs`
  - Descripción: Página con buscador (Input + Button), resultados por cliente/vehículo, botón descargar PDF. Usa AppShell, Card, Button, Input, Spinner. CSS: `.card`, `.form-group`, `.btn .btn-primary`, `.badge .badge-info`, `.alert-danger`, `.alert-success`, `.empty-state`
  - Verificación: `cargo check -p workshop-viewer`

- [x] **D3** — Registrar ruta y sidebar
  - Archivo: `crates/workshop-viewer/src/routes.rs`, `layout.rs`, `mod.rs`
  - Descripción: `Route::ServiceCertificatePage` en `/service-certificate`, enlace "Certificado Servicios" en sidebar con icono DocumentText
  - Verificación: `cargo check -p workshop-viewer`

- [x] **D4** — Dropdown productos en repairs
  - Archivo: `crates/workshop-viewer/src/pages/repairs.rs`
  - Descripción: En pestaña "Insumos", `<select>` con productos del inventario. Al seleccionar, auto-llena nombre y costo. `product_id` se envía al servidor.
  - Verificación: `cargo check -p workshop-viewer`

---

## Fase E: Verificación final

- [x] **E1** — Build + lint + tests
  - Verificación:
    - `cargo build --workspace` ✅
    - `cargo clippy --workspace -- -D warnings` ✅
    - `cargo fmt --all --check` ✅
    - `cargo test --workspace` ✅ (84 tests pasan)

---

## Archivos modificados

| Archivo | Cambio |
|---------|--------|
| `crates/workshop-server/migrations/0009_repair_parts_product_fk.sql` | NUEVO — FK product_id |
| `crates/workshop-common/src/lib.rs` | RepairPart.product_id |
| `crates/workshop-common/src/dto.rs` | DTOs certificado + product_id |
| `crates/workshop-server/Cargo.toml` | genpdf = "0.2" |
| `crates/workshop-server/src/main.rs` | mod certificate |
| `crates/workshop-server/src/certificate.rs` | NUEVO — generación PDF |
| `crates/workshop-server/src/routes/mod.rs` | pub(crate) mod reports |
| `crates/workshop-server/src/routes/reports.rs` | 3 endpoints + certificate.rs |
| `crates/workshop-server/src/routes/repairs.rs` | product_id en list/add |
| `crates/workshop-viewer/src/api.rs` | client_search + download_certificate |
| `crates/workshop-viewer/src/pages/mod.rs` | mod service_certificate |
| `crates/workshop-viewer/src/pages/service_certificate.rs` | NUEVO — página UI |
| `crates/workshop-viewer/src/pages/repairs.rs` | dropdown productos |
| `crates/workshop-viewer/src/routes.rs` | ServiceCertificatePage |
| `crates/workshop-viewer/src/pages/layout.rs` | sidebar link |
