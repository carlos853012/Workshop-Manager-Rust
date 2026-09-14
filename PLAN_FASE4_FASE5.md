# Plan: Completar Phase 4 (CRUD) + Phase 5 (Viewer)

## FASE 4: CRUD Endpoints

### 4.1 Fix analytics.rs — workshop_id isolation
- [x] Agregar `WHERE workshop_id = $1` a queries de dashboard
- [x] Agregar `WHERE workshop_id = $1` a queries de KPIs
- [x] Extraer `workshop_id` del `AuthenticatedUser` en el extractor
- [x] Tests unitarios para cálculo de average_sale
- **Estado:** ✅ COMPLETADO

### 4.2 Fix reports.rs — workshop_id + client-history endpoint
- [x] Agregar `WHERE workshop_id = $1` a `report_clients`
- [x] Fix `report_history` — audit_log no tiene workshop_id, query corregida
- [x] Nuevo endpoint `GET /api/reports/client-history?email=XXX`
- [x] DTO `ClientHistoryResponse` con ventas + reparaciones del cliente
- [x] Tests unitarios
- **Estado:** ✅ COMPLETADO

### 4.3 Dashboard real — conectar home.rs a la API
- [x] Método `get_dashboard()` en `api.rs`
- [x] Método `get_kpis()` en `api.rs`
- [x] Reemplazar valores hardcodeados `"--"` en `home.rs` con datos reales
- [x] Loading state + error handling
- [x] 4 cards con datos reales + subtitle
- [x] 2 cards de resumen (ticket promedio, clientes, reparaciones activas)
- **Estado:** ✅ COMPLETADO

### 4.4 Reports real — implementar reports.rs del viewer
- [x] Método `list_clients()` en `api.rs`
- [x] Método `get_client_history()` en `api.rs`
- [x] Tabla de clientes con columnas: email, nombre, compras, gastado, última compra
- [x] Click en cliente → historial (ventas + reparaciones)
- [x] Botón volver desde historial a lista
- [x] Resumen: total gastado, compras, reparaciones
- **Estado:** ✅ COMPLETADO

### 4.5 Users real — implementar users.rs del viewer
- [x] Método `list_users()` en `api.rs`
- [x] Método `create_user()` en `api.rs`
- [x] Método `update_user()` en `api.rs`
- [x] Método `delete_user()` en `api.rs`
- [x] Tabla de usuarios con CRUD (solo admin)
- [x] Modal crear usuario (email, nombre, password, rol)
- [x] Modal editar usuario (nombre, rol)
- [x] Eliminar con ConfirmModal
- [x] Badge de rol (Admin=Mechanic, Mechanic=Warning, Seller=Info)
- **Estado:** ✅ COMPLETADO

### 4.6 ApiClient methods — métodos faltantes
- [x] `get_dashboard()` — GET /api/analytics/dashboard
- [x] `get_kpis()` — GET /api/analytics/kpis
- [x] `list_clients()` — GET /api/reports/clients
- [x] `get_client_history()` — GET /api/reports/client-history
- [x] `list_users()` — GET /api/users
- [x] `create_user()` — POST /api/users
- [x] `update_user()` — PUT /api/users/:id
- [x] `delete_user()` — DELETE /api/users/:id
- [x] DTOs: DashboardData, KpisData, ClientReport, ClientHistory, CreateUserRequest, UpdateUserRequest
- **Estado:** ✅ COMPLETADO

---

## FASE 5: Viewer Desktop (lo que falta)

### 5.1 Dashboard — KPIs cards + recent activity
- [ ] 4 cards principales con datos reales (productos, ventas, reparaciones, proveedores)
- [ ] Sección de actividad reciente o resumen
- **Estado:** pendiente

### 5.2 Reports — client list + history
- [ ] Tabla paginada de clientes
- [ ] Detalle de historial por cliente (ventas + reparaciones)
- **Estado:** pendiente

### 5.3 Users — CRUD management
- [ ] Tabla de usuarios con rol, email, estado
- [ ] Modal crear usuario (nombre, email, password, rol)
- [ ] Modal editar usuario
- [ ] Eliminar con confirmación
- **Estado:** pendiente

### 5.4 Build + clippy + tests
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo fmt --all --check`
- **Estado:** pendiente

---

## Resumen de archivos afectados

| Archivo | Cambio |
|---------|--------|
| `workshop-server/src/routes/analytics.rs` | +workshop_id filters |
| `workshop-server/src/routes/reports.rs` | +workshop_id filters, +client-history endpoint |
| `workshop-viewer/src/api.rs` | +8 métodos, +6 DTOs nuevos |
| `workshop-viewer/src/pages/home.rs` | Dashboard real con API |
| `workshop-viewer/src/pages/reports.rs` | Client list + history UI |
| `workshop-viewer/src/pages/users.rs` | CRUD users UI |
