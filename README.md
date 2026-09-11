# WorkshopManager

Sistema de gestión integral para talleres de motocicletas. Inventario, ventas, reparaciones, proveedores y reportes — todo en una aplicación de escritorio con backend embebido.

## Características

- **Inventario**: productos con stock, precios, categorías y códigos de barras
- **Ventas**: POS con medios de pago, IVA y descuentos
- **Reparaciones**: órdenes de trabajo con estado, prioridad y partes
- **Proveedores**: gestión y vinculación con productos
- **Reportes**: dashboard analítico, historial de clientes, ventas por período
- **Multi-taller**: aislamiento de datos por taller con roles (Admin/Mechanic/Seller)
- **Seguridad**: JWT + Argon2id, AES-256-GCM, TLS, auditoría de cambios
- **Licenciamiento offline**: Ed25519 hardware-bound, sin conexión a internet
- **Distribución**: MSI (Windows) + .deb (Linux amd64/arm64)

## Arquitectura

```
┌─────────────────────┐     HTTPS/TLS      ┌──────────────────────┐
│  inventory-viewer   │ ◄────────────────►  │  inventory-server    │
│  Dioxus Desktop     │    REST API         │  Axum 0.7 + Tokio    │
│  (WebView2)         │                     │  PostgreSQL embebido  │
└─────────────────────┘                     └──────────────────────┘
        │                                           │
        └──────────► inventory-common ◄─────────────┘
                     (tipos compartidos)
```

**Crate `inventory-common`**: tipos de dominio, DTOs, enums, sistema de licencias.

**Crate `inventory-server`**: backend HTTP con autenticación JWT, cifrado AES-256-GCM, auditoría, backup automático, y PostgreSQL embebido (sin instalación externa).

**Crate `inventory-viewer`**: cliente de escritorio con Dioxus, componentes Atomic Design, temas claro/oscuro.

## Requisitos previos

- [Rust](https://rustup.rs/) (stable, edition 2021)
- **Windows**: WebView2 Runtime (incluido en Windows 10/11)
- **Linux**: libwebkit2gtk-4.1-dev
- **Packaging**: [WiX Toolset v3](https://wixtoolset.org/docs/wix3/) (MSI), [cargo-deb](https://github.com/kornelski/cargo-deb) (.deb)

## Compilación

```powershell
# Compilar todo el workspace
cargo build --workspace

# Compilar solo un crate
cargo build -p inventory-server
cargo build -p inventory-viewer

# Verificar sin generar binarios
cargo check -p inventory-viewer

# Release optimizado
cargo build --release --workspace
```

## Ejecución

```powershell
# Server (escucha en https://0.0.0.0:8443)
cargo run -p inventory-server

# Viewer (abre ventana de escritorio)
cargo run -p inventory-viewer
```

El server genera automáticamente PostgreSQL embebido, certificados TLS, y credenciales en el primer arranque. Los datos se almacenan en el directorio de datos local de la plataforma.

## Testing y linting

```powershell
# Tests
cargo test --workspace

# Linting
cargo clippy --workspace -- -D warnings

# Formato
cargo fmt --all --check
```

## Estructura del proyecto

```
workshop-manager/
├── Cargo.toml                     # Workspace root
├── config/
│   ├── server.toml                # Config del server (gitignored)
│   └── viewer.toml                # Config del viewer (gitignored)
├── crates/
│   ├── inventory-common/          # Tipos compartidos
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── dto.rs             # Data Transfer Objects
│   │       ├── features.rs        # Features por tier de licencia
│   │       ├── license.rs         # Verificación Ed25519
│   │       ├── money.rs           # Lógica monetaria
│   │       └── patente.rs         # Validación patentes
│   ├── inventory-server/          # Backend
│   │   ├── migrations/            # Migraciones SQL (8 archivos)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── auth.rs            # JWT + Argon2id
│   │       ├── crypto.rs          # AES-256-GCM
│   │       ├── tls.rs             # Certificados autofirmados
│   │       ├── audit.rs           # Log de cambios
│   │       ├── backup.rs          # Backup automático
│   │       ├── db_manager.rs      # PostgreSQL embebido
│   │       └── routes/            # Endpoints REST
│   └── inventory-viewer/          # Frontend Desktop
│       ├── index.css              # Estilos (inlinados)
│       ├── assets/                # Tokens de tema
│       └── src/
│           ├── main.rs
│           ├── api.rs             # Cliente HTTP
│           ├── pages/             # 13 páginas
│           └── components/        # Atomic Design
│               ├── atoms/         # Button, Input, Badge, Icon, Spinner
│               ├── molecules/     # Card, Modal, FormGroup, Tooltip
│               └── organisms/     # Header, DataTable, ConnectionSettings
├── scripts/
│   └── bump.ps1                   # Versionado semántico
└── .github/workflows/
    ├── ci.yml                     # Build + lint + test
    └── release.yml                # Build multi-plataforma + publish
```

## Distribución

```powershell
# Versionado (actualiza Cargo.toml, commitea, taggea)
.\scripts\bump.ps1 0.2.0

# Windows: MSI
cargo build --release -p inventory-server -p inventory-viewer
cargo wix -p inventory-server --nocapture
cargo wix -p inventory-viewer --nocapture

# Linux: .deb
cargo deb -p inventory-server
```

Los workflows de CI/CD replican estos pasos: `ci.yml` en cada push, `release.yml` al taggear `v*` (genera 2 MSIs + 2 .deb + 2 binarios raw).

## Licencia

Proprietary — Ver `Cargo.toml` workspace para detalles.

## Documentación interna

- `PLAN_DESARROLLO.md` — Plan de desarrollo en 8 fases
- `CONSTITUCION.md` — Reglas de ingeniería del proyecto
- `AUDITORIA.md` — Auditoría de seguridad y calidad
- `COMMANDS.md` — Comandos de build, deploy y troubleshooting
- `AGENTS.md` — Guía para asistentes de IA
