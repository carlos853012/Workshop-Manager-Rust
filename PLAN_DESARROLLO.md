# WorkshopManager - Plan de Desarrollo Detallado

## Información del Proyecto

| Campo | Valor |
|-------|-------|
| **Nombre** | WorkshopManager |
| **Versión** | 0.1.0 |
| **Lenguaje** | Rust (edition 2021) |
| **Backend** | Axum 0.7 + Tokio + PostgreSQL embebido |
| **Frontend** | Dioxus Desktop 0.5 + WebView2 |
| **Distribución** | MSI (Windows) + .deb (Linux x64/ARM64) |
| **Licenciamiento** | Offline, Ed25519, hardware-bound |

---

## Convenciones del Proyecto

### Criterios de Aceptación Globales

Cada tarea DEBE cumplir **TODOS** estos criterios antes de continuar con la siguiente:

1. **Compilación**: `cargo build --workspace` sin errores
2. **Linting**: `cargo clippy --workspace -- -D warnings` sin warnings
3. **Formatting**: `cargo fmt --all --check` pasa
4. **Seguridad**: Sin `unsafe`, sin `unwrap()`/`expect()`/`panic!()` en código de producción
5. **Documentación**: Funciones públicas tienen `///` doc comments
6. **Tests**: Cada función tiene al menos 1 test unitario

### Flujo de Trabajo Obligatorio (5 Pasos)

Antes de modificar código:
1. **Entender**: Explicar qué se entiende y qué problema se resuelve
2. **Archivos**: Identificar archivos afectados y dependencias
3. **Riesgos**: Analizar efectos secundarios y impacto
4. **Plan**: Proponer plan de implementación paso a paso
5. **Ejecutar**: Generar cambios una vez alineado el plan

---

## FASE 1: Setup del Proyecto (Semana 1)

### Tarea 1.1: Crear workspace Cargo.toml

**Descripción:** Crear la estructura base del workspace con los 3 crates.

**Archivos a crear:**
```
workshop-manager/
├── Cargo.toml
├── .gitignore
├── .cargo/config.toml
├── crates/
│   ├── workshop-common/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── workshop-server/
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/main.rs
│   └── workshop-viewer/
│       ├── Cargo.toml
│       ├── build.rs
│       └── src/main.rs
```

**Criterios de aceptación:**
- [ ] `cargo build --workspace` compila sin errores
- [ ] `cargo clippy --workspace` no tiene warnings
- [ ] `cargo fmt --all --check` pasa
- [ ] Los 3 crates aparecen en `cargo metadata`
- [ ] `.gitignore` excluye target/, dev-data/, *.msi, *.deb

---

### Tarea 1.2: Configurar profiles de release

**Descripción:** Configurar optimizaciones de compilación para binarios mínimos y seguros.

**Archivo:** `Cargo.toml` (workspace root)

**Contenido:**
```toml
[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
opt-level = "z"
```

**Criterios de aceptación:**
- [ ] `cargo build --release --workspace` compila
- [ ] Binarios son significativamente más pequeños que debug
- [ ] No hay símbolos de debug en el binario (verificar con `nm` o `objdump`)

---

### Tarea 1.3: Crear .gitignore completo

**Descripción:** Crear .gitignore que excluya todos los archivos que no deben ir al repositorio.

**Criterios de aceptación:**
- [ ] Excluye: target/, dev-data/, *.msi, *.deb, *.exe
- [ ] Excluye: .env, *.key, *.pem, secrets/
- [ ] Excluye: pgdata/, *.db, *.sqlite
- [ ] Incluye: Cargo.toml, src/, config/, scripts/

---

### Tarea 1.4: Copiar skills de noc-system

**Descripción:** Copiar los 9 skills de `.opencode/skills/` de noc-system.

**Directorio origen:** `C:\Users\carlos\Desktop\noc-system\.opencode\skills\`
**Directorio destino:** `workshop-manager/.opencode/skills/`

**Skills a copiar:**
1. business-analyst-senior
2. database-specialist-senior
3. devops-sre-senior
4. devsecops-senior
5. production-maintenance-senior
6. project-manager-senior
7. qa-senior
8. senior-developer-code-review
9. software-architect-senior

**Criterios de aceptación:**
- [ ] Los 9 directorios existen con sus SKILL.md
- [ ] Los references están completos (37 archivos)
- [ ] No hay archivos corruptos o faltantes

---

### Tarea 1.5: Crear CONSTITUCION.md

**Descripción:** Adaptar la CONSTITUCION.md de Equipos-Rust, eliminando sección OT/SCADA y agregando reglas de taller mecánico.

**Archivo origen:** `C:\Users\carlos\Desktop\Equipos-Rust\CONSTITUCION.md`
**Archivo destino:** `workshop-manager/CONSTITUCION.md`

**Modificaciones:**
1. Eliminar sección 16 (OT/SCADA/PLC) - no aplica
2. Agregar sección "Dominio de Negocio - Taller Mecánico":
   - Inventario de productos/repuestos
   - Ventas y métodos de pago
   - Reparaciones de motocicletas
   - Gestión de proveedores
   - Reports PDF para clientes
3. Mantener todas las demás secciones intactas

**Criterios de aceptación:**
- [ ] Sección 16 (OT/SCADA) eliminada
- [ ] Nueva sección de dominio agregada
- [ ] Reglas de seguridad Rust mantenidas
- [ ] Flujo de 5 pasos mantenido
- [ ] Modo Revisor Senior mantenido

---

### Tarea 1.6: Crear AGENTS.md

**Descripción:** Crear instrucciones para asistentes IA.

**Criterios de aceptación:**
- [ ] Planning workflow definido
- [ ] Constitutional rules referenciadas
- [ ] Arquitectura documentada
- [ ] Key commands documentados
- [ ] CI/CD pipeline documentado
- [ ] Routes API documentadas

---

### Tarea 1.7: Crear COMMANDS.md

**Descripción:** Crear archivo de comandos útiles.

**Criterios de aceptación:**
- [ ] Comandos de compilación
- [ ] Comandos de linting
- [ ] Comandos de testing
- [ ] Comandos de build MSI/DEB
- [ ] Variables de entorno

---

### Tarea 1.8: Crear archivos de asistentes IA

**Descripción:** Crear .clinerules, .cursorrules, .geminirules.

**Criterios de aceptación:**
- [ ] Los 3 archivos existen
- [ ] Referencian CONSTITUCION.md
- [ ] Reglas de Rust incluidas
- [ ] Flujo de 5 pasos incluido

---

## FASE 2: Domain Layer (Semana 2)

### Tarea 2.1: Configurar dependencias de workshop-common

**Descripción:** Definir las dependencias del crate common.

**Archivo:** `crates/workshop-common/Cargo.toml`

**Dependencias:**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
sqlx = { version = "0.7", features = ["postgres", "chrono", "uuid"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
ed25519-dalek = "2.1"
```

**Criterios de aceptación:**
- [ ] `cargo build -p workshop-common` compila
- [ ] Dependencias se resuelven correctamente

---

### Tarea 2.2: Crear entidades de dominio

**Descripción:** Definir todas las estructuras del dominio.

**Archivo:** `crates/workshop-common/src/lib.rs`

**Entidades:**
- Product
- Sale, SaleItem
- Repair, RepairUpdate
- Supplier
- User
- AuditLog
- Report

**Enums:**
- PaymentMethod (Cash, Card, Transfer)
- RepairStatus (Pending, InProgress, Completed, Cancelled)
- Priority (High, Medium, Low)
- UserRole (Admin, Mechanic, Seller)
- EntityStatus (Active, Inactive, Deleted)

**Criterios de aceptación:**
- [ ] Todas las estructuras compiladas
- [ ] Derives: Serialize, Deserialize, Clone, Debug, FromRow
- [ ] Enums con strum_macros (Display, EnumString)
- [ ] Tests para conversión de enums

---

### Tarea 2.3: Crear features.rs

**Descripción:** Definir sistema de features para DLCs.

**Archivo:** `crates/workshop-common/src/features.rs`

**Criterios de aceptación:**
- [ ] Todos los features listados
- [ ] LicenseTier enum definido
- [ ] License struct con métodos has_feature(), is_valid()
- [ ] Tests para has_feature()

---

### Tarea 2.4: Crear license.rs

**Descripción:** Implementar validación Ed25519.

**Archivo:** `crates/workshop-common/src/license.rs`

**Criterios de aceptación:**
- [ ] verify_license() funciona
- [ ] extract_hardware_hash() es consistente
- [ ] validate_hardware() funciona
- [ ] Tests con keys generadas pasan

---

### Tarea 2.5: Crear dto.rs

**Descripción:** Definir DTOs para API.

**Archivo:** `crates/workshop-common/src/dto.rs`

**Criterios de aceptación:**
- [ ] Request structs definidas
- [ ] Response structs definidas
- [ ] ApiResponse<T> genérico
- [ ] PaginatedResponse<T> genérico

---

## FASE 3: Server Bootstrap (Semanas 3-4)

### Tarea 3.1: Configurar dependencias del server

**Descripción:** Definir todas las dependencias del server.

**Archivo:** `crates/workshop-server/Cargo.toml`

**Dependencias principales:**
```toml
[dependencies]
workshop-common = { path = "../workshop-common" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "chrono", "uuid"] }
postgresql_embedded = { version = "0.20", features = ["bundled", "rustls"] }
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-appender = "0.2"
argon2 = "0.5"
jsonwebtoken = "9"
aes-gcm = "0.10"
tower-http = { version = "0.6", features = ["cors", "limit", "trace"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
dirs = "5.0"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
rand = "0.8"
base64 = "0.22"
obfstr = "0.4"
```

**Criterios de aceptación:**
- [ ] `cargo build -p workshop-server` compila
- [ ] Todas las dependencias se resuelven

---

### Tarea 3.2: Crear main.rs

**Descripción:** Implementar punto de entrada con bootstrap completo.

**Flujo:**
1. Init tracing logger
2. Load config (server.toml)
3. Generate/load secrets
4. Start PostgreSQL embedded
5. Run migrations
6. Create connection pool
7. Create AppState
8. Build Axum router
9. Start server with graceful shutdown

**Criterios de aceptación:**
- [ ] Server arranca sin errores
- [ ] PostgreSQL embebido se inicia
- [ ] Health check responde
- [ ] Graceful shutdown funciona
- [ ] Logs se escriben

---

### Tarea 3.3: Crear state.rs

**Descripción:** Definir estado compartido.

**Criterios de aceptación:**
- [ ] AppState implementa Clone
- [ ] FromRef funciona para sub-estados
- [ ] Documentación completa

---

### Tarea 3.4: Crear config.rs

**Descripción:** Implementar carga de configuración TOML.

**Archivo:** `crates/workshop-server/src/config.rs`

**Criterios de aceptación:**
- [ ] ServerConfig se carga desde TOML
- [ ] Auto-generación si falta el archivo
- [ ] Override por variables de entorno
- [ ] Test de carga

---

### Tarea 3.5: Crear secrets.rs

**Descripción:** Implementar generación y persistencia de secretos.

**Criterios de aceptación:**
- [ ] Secretos se generan en primer arranque
- [ ] Secretos se persisten en disco
- [ ] Secretos se reutilizan
- [ ] Override por env vars
- [ ] Permisos 0600 en archivos

---

### Tarea 3.6: Crear crypto.rs

**Descripción:** Implementar AES-256-GCM.

**Criterios de aceptación:**
- [ ] encrypt/decrypt funciona
- [ ] Nonce aleatorio
- [ ] Tests roundtrip pasan

---

### Tarea 3.7: Crear error.rs

**Descripción:** Definir AppError enum.

**Criterios de aceptación:**
- [ ] 7+ variantes definidas
- [ ] IntoResponse implementado
- [ ] Display implementado

---

### Tarea 3.8: Crear schema.rs

**Descripción:** Implementar migraciones SQL.

**Criterios de aceptación:**
- [ ] Migración 0001_initial.sql creada
- [ ] sqlx::migrate!() funciona
- [ ] Todas las tablas se crean

---

### Tarea 3.9: Crear db_manager.rs

**Descripción:** Wrapper para PostgreSQL embebido.

**Criterios de aceptación:**
- [ ] setup() instala PostgreSQL
- [ ] start() inicia el servicio
- [ ] stop() detiene el servicio
- [ ] Connection string se genera

---

### Tarea 3.10: Crear middleware.rs

**Descripción:** Implementar cadena de middleware.

**Middlewares:**
1. require_device_key
2. authenticate_middleware
3. require_admin
4. rate_limiter

**Criterios de aceptación:**
- [ ] Cadena funciona correctamente
- [ ] JWT se verifica
- [ ] Roles se validan
- [ ] Rate limiting funciona

---

### Tarea 3.11: Crear auth.rs (routes)

**Descripción:** Implementar endpoints de autenticación.

**Endpoints:**
- POST /api/auth/login
- POST /api/auth/register
- GET /api/auth/status

**Criterios de aceptación:**
- [ ] Login funciona
- [ ] Register crea usuario
- [ ] Status retorna usuario actual
- [ ] Passwords hasheados con Argon2

---

### Tarea 3.12: Crear rate_limiter.rs

**Descripción:** Implementar rate limiting dual.

**Criterios de aceptación:**
- [ ] IpRateLimiter (3/60s)
- [ ] UsernameRateLimiter (5/60s)
- [ ] Eviction automática
- [ ] HTTP 429 en exceso

---

### Tarea 3.13: Crear audit.rs

**Descripción:** Implementar auditoría con diff JSON.

**Criterios de aceptación:**
- [ ] log_change() registra cambios
- [ ] Diff solo campos modificados
- [ ] Campos sensibles redactados [CIFRADO]
- [ ] Timestamps con timezone

---

### Tarea 3.14: Crear tls.rs

**Descripción:** Generar certificados autofirmados.

**Criterios de aceptación:**
- [ ] Certificado se genera con rcgen
- [ ] Se persiste en disco
- [ ] Se reutiliza en arranques subsecuentes
- [ ] HTTPS funciona

---

### Tarea 3.15: Crear backup.rs

**Descripción:** Implementar backup automático.

**Criterios de aceptación:**
- [ ] Backup se ejecuta automáticamente
- [ ] Archivos comprimidos (gzip)
- [ ] Restore atómico (rename swap)
- [ ] Retención configurable

---

## FASE 4: CRUD Endpoints (Semanas 5-7)

### Tarea 4.1: Crear routes/products.rs

**Descripción:** CRUD de productos.

**Endpoints:**
- GET /api/products
- GET /api/products/:id
- POST /api/products
- PUT /api/products/:id
- DELETE /api/products/:id

**Criterios de aceptación:**
- [ ] Todos los endpoints funcionan
- [ ] Validación de inputs
- [ ] Paginación en GET list
- [ ] RBAC (admin, seller)
- [ ] Audit log en CREATE/UPDATE/DELETE

---

### Tarea 4.2: Crear routes/sales.rs

**Descripción:** CRUD de ventas con items.

**Endpoints:**
- GET /api/sales
- GET /api/sales/:id
- POST /api/sales (crea venta + items + descuenta stock)

**Criterios de aceptación:**
- [ ] Venta se crea con items
- [ ] Stock se descuenta automáticamente
- [ ] Rollback si falla la venta
- [ ] RBAC (seller, admin)

---

### Tarea 4.3: Crear routes/repairs.rs

**Descripción:** CRUD de reparaciones.

**Endpoints:**
- GET /api/repairs
- GET /api/repairs/:id
- POST /api/repairs
- PUT /api/repairs/:id

**Criterios de aceptación:**
- [ ] Estados se manejan correctamente
- [ ] Repair updates se crean automáticamente
- [ ] RBAC (mechanic, admin)

---

### Tarea 4.4: Crear routes/suppliers.rs

**Descripción:** CRUD de proveedores.

**Endpoints:**
- GET /api/suppliers
- GET /api/suppliers/:id
- POST /api/suppliers
- PUT /api/suppliers/:id

**Criterios de aceptación:**
- [ ] CRUD completo funciona
- [ ] RBAC (admin)

---

### Tarea 4.5: Crear routes/analytics.rs

**Descripción:** Dashboard y KPIs.

**Endpoints:**
- GET /api/analytics/dashboard
- GET /api/analytics/kpis

**Criterios de aceptación:**
- [ ] KPIs calculados correctamente
- [ ] Respuesta optimizada (una sola query)

---

### Tarea 4.6: Crear routes/users.rs

**Descripción:** Gestión de usuarios.

**Endpoints:**
- GET /api/users
- POST /api/users
- PUT /api/users/:id
- DELETE /api/users/:id

**Criterios de aceptación:**
- [ ] RBAC (admin only)
- [ ] No se puede eliminar a sí mismo
- [ ] Password hasheado

---

### Tarea 4.7: Crear routes/reports.rs

**Descripción:** Generación de reports PDF.

**Endpoints:**
- GET /api/clients (lista clientes únicos)
- GET /api/clients/:name/history
- GET /api/reports

**Criterios de aceptación:**
- [ ] Historial completo por cliente
- [ ] Ventas + reparaciones incluidas

---

## FASE 5: Viewer Desktop (Semanas 8-10)

### Tarea 5.1: Configurar dependencias del viewer

**Archivo:** `crates/workshop-viewer/Cargo.toml`

**Dependencias:**
```toml
[dependencies]
workshop-common = { path = "../workshop-common" }
dioxus = { version = "0.5", features = ["desktop"] }
dioxus-desktop = "0.5"
dioxus-router = "0.5"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rfd = "0.15"
webbrowser = "1.2"
image = "0.25"
```

---

### Tarea 5.2: Crear theme/mod.rs

**Descripción:** Implementar ThemeProvider centralizado.

**Criterios de aceptación:**
- [ ] ThemeProvider provee tokens
- [ ] CSS variables se sincronizan
- [ ] Workshop config se carga
- [ ] toggle_mode funciona

---

### Tarea 5.3: Crear theme/tokens.rs

**Descripción:** Definir DesignTokens desde TOML.

**Criterios de aceptación:**
- [ ] Struct deserialize desde TOML
- [ ] Métodos dark() y light()
- [ ] to_css_variables() genera CSS

---

### Tarea 5.4: Crear index.css

**Descripción:** Crear CSS base con variables.

**Criterios de aceptación:**
- [ ] Variables CSS en :root
- [ ] Clases utilitarias
- [ ] Componentes base (btn, card, input, badge, modal)

---

### Tarea 5.5: Crear componentes atoms

**Descripción:** Button, Input, Badge, Icon, Spinner.

**Archivos:**
- `components/atoms/button.rs`
- `components/atoms/input.rs`
- `components/atoms/badge.rs`
- `components/atoms/icon.rs`
- `components/atoms/spinner.rs`

**Criterios de aceptación:**
- [ ] Button con 7 variantes
- [ ] Input con label y error
- [ ] Badge con 5 colores
- [ ] Icon parametrizable
- [ ] Spinner animado

---

### Tarea 5.6: Crear componentes molecules

**Descripción:** Card, Modal, FormGroup, Tooltip.

**Archivos:**
- `components/molecules/card.rs`
- `components/molecules/modal.rs`
- `components/molecules/confirm_modal.rs`
- `components/molecules/form_group.rs`
- `components/molecules/tooltip.rs`

**Criterios de aceptación:**
- [ ] Card con header/body/footer
- [ ] Modal con show/close
- [ ] ConfirmModal con callbacks
- [ ] FormGroup con validación

---

### Tarea 5.7: Crear componentes organisms

**Descripción:** Header, Sidebar, DataTable.

**Archivos:**
- `components/organisms/header.rs`
- `components/organisms/sidebar.rs`
- `components/organisms/data_table.rs`

**Criterios de aceptación:**
- [ ] Header con user menu
- [ ] Sidebar con logo del taller
- [ ] DataTable con paginación y búsqueda

---

### Tarea 5.8: Crear icons.rs

**Descripción:** 27+ SVG Heroicons inline.

**Criterios de aceptación:**
- [ ] Iconos de navegación
- [ ] Iconos de acción (CRUD)
- [ ] Iconos de estado
- [ ] Logo del taller

---

### Tarea 5.9: Crear páginas del viewer

**Páginas:**
1. home.rs (Dashboard)
2. login.rs
3. setup.rs (Primer admin)
4. license.rs (Activación)
5. products.rs
6. product_detail.rs
7. sales.rs
8. new_sale.rs
9. repairs.rs
10. new_repair.rs
11. suppliers.rs
12. reports.rs
13. client_history.rs
14. users.rs

**Criterios de aceptación:**
- [ ] Todas las páginas funcionan
- [ ] Navegación entre páginas
- [ ] Loading states
- [ ] Error handling

---

### Tarea 5.10: Crear api.rs (cliente HTTP)

**Descripción:** ApiClient centralizado.

**Criterios de aceptación:**
- [ ] Headers automáticos (Authorization)
- [ ] Timeouts configurables
- [ ] Error handling
- [ ] Base URL configurable

---

## FASE 6: Packaging y Distribución (Semana 11)

### Tarea 6.1: Crear scripts/bump.ps1

**Descripción:** Script de version bump.

**Criterios de aceptación:**
- [ ] Valida formato X.Y.Z
- [ ] Actualiza Cargo.toml
- [ ] Verifica build
- [ ] Crea commit y tag

---

### Tarea 6.2: Crear wix/main.wxs (Server)

**Descripción:** MSI para Windows server.

**Criterios de aceptación:**
- [ ] Instala en Program Files
- [ ] Datos en ProgramData
- [ ] Shortcut en Menú Inicio
- [ ] Hardening de ACLs

---

### Tarea 6.3: Crear wix/main.wxs (Viewer)

**Descripción:** MSI para Windows viewer.

**Criterios de aceptación:**
- [ ] Instala en Program Files
- [ ] Shortcut en Menú Inicio y Escritorio

---

### Tarea 6.4: Crear debian/postinst

**Descripción:** Scripts para .deb Linux.

**Criterios de aceptación:**
- [ ] Crea usuario del sistema
- [ ] Crea directorio de datos
- [ ] Instala servicio systemd

---

### Tarea 6.5: Crear debian/workshop-server.service

**Descripción:** Systemd unit file.

**Criterios de aceptación:**
- [ ] Servicio arranca automáticamente
- [ ] Reinicia en caso de fallo
- [ ] Logs via journalctl

---

## FASE 7: CI/CD (Semana 12)

### Tarea 7.1: Crear .github/workflows/ci.yml

**Descripción:** Pipeline de integración continua.

**Criterios de aceptación:**
- [ ] fmt check
- [ ] clippy check
- [ ] build
- [ ] test
- [ ] cargo audit

---

### Tarea 7.2: Crear .github/workflows/release.yml

**Descripción:** Pipeline de release automático.

**Criterios de aceptación:**
- [ ] Build MSI Windows
- [ ] Build .deb Linux x64
- [ ] Build .deb Linux ARM64
- [ ] Publica GitHub Release

---

## FASE 8: Testing y Documentación (Semana 13)

### Tarea 8.1: Crear tests de integración

**Descripción:** Tests completos del server.

**Criterios de aceptación:**
- [ ] Tests de RBAC
- [ ] Tests de CRUD
- [ ] Tests de validación
- [ ] Tests de seguridad

---

### Tarea 8.2: Crear test_seguridad.ps1

**Descripción:** Suite de tests de seguridad.

**Criterios de aceptación:**
- [ ] Health check + TLS
- [ ] Login con credenciales
- [ ] Bloqueo de registro
- [ ] Sin auth en rutas protegidas
- [ ] Roles
- [ ] Cifrado en DB
- [ ] Redacción en audit

---

### Tarea 8.3: Crear README.md

**Descripción:** Documentación principal.

**Criterios de aceptación:**
- [ ] Arquitectura documentada
- [ ] Instalación documentada
- [ ] API documentada
- [ ] Seguridad documentada

---

### Tarea 8.4: Crear ARQUITECTURA.md

**Descripción:** Documentación técnica completa.

**Criterios de aceptación:**
- [ ] Diagramas de arquitectura
- [ ] Flujos de datos
- [ ] Modelo de datos
- [ ] Decisiones de diseño

---

### Tarea 8.5: Crear GUIA_USUARIO.md

**Descripción:** Guía para el usuario final.

**Criterios de aceptación:**
- [ ] Instalación paso a paso
- [ ] Configuración inicial
- [ ] Uso de cada módulo
- [ ] Solución de problemas

---

## Estimación de Tiempo

| Fase | Semanas | Dependencias |
|------|---------|--------------|
| Fase 1: Setup | 1 | Ninguna |
| Fase 2: Domain | 1 | Fase 1 |
| Fase 3: Server | 2 | Fase 2 |
| Fase 4: CRUD | 3 | Fase 3 |
| Fase 5: Viewer | 3 | Fase 4 |
| Fase 6: Packaging | 1 | Fase 5 |
| Fase 7: CI/CD | 1 | Fase 6 |
| Fase 8: Testing | 1 | Fase 7 |
| **Total** | **13 semanas** | |

---

## Dependencias entre Fases

```
Fase 1 (Setup)
    │
    ▼
Fase 2 (Domain)
    │
    ▼
Fase 3 (Server Bootstrap)
    │
    ▼
Fase 4 (CRUD Endpoints)
    │
    ▼
Fase 5 (Viewer Desktop)
    │
    ▼
Fase 6 (Packaging)
    │
    ▼
Fase 7 (CI/CD)
    │
    ▼
Fase 8 (Testing)
```

---

## Stack Tecnológico Final

| Categoría | Tecnología |
|-----------|------------|
| Lenguaje | Rust (edition 2021) |
| Backend | Axum 0.7 + Tokio |
| DB | PostgreSQL embebido (postgresql_embedded) |
| Frontend | Dioxus Desktop 0.5 |
| Auth | JWT HS256 + Argon2id |
| Crypto | AES-256-GCM |
| License | Ed25519 (ed25519-dalek) |
| Ofuscación | obfuse-rs + strip + lto |
| Packaging | WiX (MSI) + cargo-deb (.deb) |
| CI/CD | GitHub Actions |
| Red | Tailscale |
