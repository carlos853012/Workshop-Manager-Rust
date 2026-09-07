# Auditoría WorkshopManager — Plan de Corrección

**Fecha:** 2026-09-07
**Estado:** En progreso

---

## 🔴 CRÍTICOS (corregir hoy)

- [x] **C-1** IDOR: `list_repairs` sin filtro `workshop_id` — `server/routes/repairs.rs:40-81`
- [x] **C-2** IDOR: `create_sale` sin filtro `workshop_id` en SELECT/UPDATE de productos — `server/routes/sales.rs:201-235`
- [x] **C-3** IDOR: `users` CRUD sin filtro `workshop_id` — `server/routes/users.rs:65,96,186,267`
- [x] **C-4** IDOR: `list_repair_parts` sin verificación de pertenencia — `server/routes/repairs.rs:387-415`
- [x] **C-5** Sin control de roles: seller modifica productos, mechanic crea ventas — `products.rs`, `sales.rs`
- [x] **C-6** Endpoint `reports/client-history` roto: columna `total` no existe — `reports.rs:129`
- [x] **C-7** `StockEntryModal` borra cost/min_stock/supplier al ingresar stock — `viewer/products.rs:542-559`
- [x] **C-8** `ApiResponse<()>` rompe DELETEs en viewer — `viewer/api.rs:462-466`
- [x] **C-9** Migración 0005 rota para DBs con datos — `migrations/0005:3`

## 🟠 ALTA (esta semana)

- [x] **A-1** Rate limiter escrito pero nunca conectado — login sin protección
- [x] **A-2** Viewer acepta cualquier cert TLS (`accept_invalid_certs(true)`)
- [x] **A-3** Auditoría post-commit → 500 con éxito real → duplicados en POS
- [x] **A-4** JWT sin revocación — usuario desactivado con acceso 24h
- [x] **A-5** IVA 19% hardcodeado en server Y viewer
- [x] **A-6** 10 DTOs duplicados entre server e inventory-common
- [x] **A-7** Tests faltantes: lógica monetaria + aislamiento multi-taller
- [x] **A-8** Errores de DB expuestos al cliente (~40 sitios)
- [x] **A-9** API key hardcodeada y commiteada en git
- [x] **A-10** CORS totalmente permisivo
- [x] **A-11** Backup sin cifrar + contraseña en línea de comandos
- [x] **A-12** Clippy warning rompe CI — `home.rs:112`

## 🟡 MEDIA (próximo sprint)

- [x] **M-1** Backup duerme 24h antes del primer backup
- [ ] **M-2** No hay DOWN migrations
- [x] **M-3** Deadlock por orden de locks en ventas concurrentes
- [ ] **M-4** Barcode con sufijo aleatorio — colisión ~40% con 1000 productos
- [ ] **M-5** repair_parts no descuenta stock de products
- [x] **M-6** `create_repair` sin transacción
- [x] **M-7** Viewer no hace logout en 401
- [ ] **M-8** Enums decodificados como String en reports
- [x] **M-9** Descuentos sin validar rango — totales negativos posibles
- [x] **M-10** Permisos archivos sensibles en Windows
- [x] **M-11** `unwrap()`/`expect()` en producción
- [x] **M-12** Audit incompleto — add/remove parts sin log_change
- [x] **M-13** Device keys sin UI
- [x] **M-14** 30 `#[allow(dead_code)]` espurios
- [x] **M-15** Índice único ignora soft-delete
- [ ] **M-16** `DbManager::stop()` dead code — PG sin shutdown limpio

## 🔵 BAJA (backlog)

- [ ] **B-1** Eliminar dependencias no usadas (tracing-appender, obfstr, thiserror, ed25519-dalek)
- [ ] **B-2** Eliminar código muerto (Badge, Icon, FormGroup, Tooltip, layout.rs, PosSaleDetail/PosSaleResponse, AuthError, 12 iconos)
- [ ] **B-3** Reescribir AGENTS.md al estado real
- [ ] **B-4** Crear README.md
- [ ] **B-5** Crear script de restore para backups
- [ ] **B-6** Unificar IVA como config centralizada
- [ ] **B-7** refresh token o TTL corto para JWT
- [ ] **B-8** Anulación/devolución de ventas
- [ ] **B-9** repair_parts vinculados a products
- [ ] **B-10** Endpoint de lectura de audit logs

## 📦 DEPENDENCIAS

- [ ] **D-1** Upgrade sqlx 0.7.4 → 0.8.1+ (RUSTSEC-2024-0363)
- [ ] **D-2** Evaluar webbrowser 0.8.15 → 1.2.2+ (RUSTSEC-2026-0257)
- [ ] **D-3** Evaluar rsa 0.9.10 (sin fix disponible)
