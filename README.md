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
│  workshop-viewer   │ ◄────────────────►  │  workshop-server    │
│  Dioxus Desktop     │    REST API         │  Axum 0.7 + Tokio    │
│  (WebView2)         │                     │  PostgreSQL embebido  │
└─────────────────────┘                     └──────────────────────┘
        │                                           │
        └──────────► workshop-common ◄─────────────┘
                     (tipos compartidos)
```

**Crate `workshop-common`**: tipos de dominio, DTOs, enums, sistema de licencias.

**Crate `workshop-server`**: backend HTTP con autenticación JWT, cifrado AES-256-GCM, auditoría, backup automático, y PostgreSQL embebido (sin instalación externa).

**Crate `workshop-viewer`**: cliente de escritorio con Dioxus, componentes Atomic Design, temas claro/oscuro.

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
cargo build -p workshop-server
cargo build -p workshop-viewer

# Verificar sin generar binarios
cargo check -p workshop-viewer

# Release optimizado
cargo build --release --workspace
```

## Ejecución

```powershell
# Server (escucha en https://0.0.0.0:8443)
cargo run -p workshop-server

# Viewer (abre ventana de escritorio)
cargo run -p workshop-viewer
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
│   ├── workshop-common/          # Tipos compartidos
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── dto.rs             # Data Transfer Objects
│   │       ├── features.rs        # Features por tier de licencia
│   │       ├── license.rs         # Verificación Ed25519
│   │       ├── money.rs           # Lógica monetaria
│   │       └── patente.rs         # Validación patentes
│   ├── workshop-server/          # Backend
│   │   ├── migrations/            # Migraciones SQL (10 archivos)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── auth.rs            # JWT + Argon2id
│   │       ├── crypto.rs          # AES-256-GCM
│   │       ├── tls.rs             # Certificados autofirmados
│   │       ├── audit.rs           # Log de cambios
│   │       ├── backup.rs          # Backup automático
│   │       ├── db_manager.rs      # PostgreSQL embebido
│   │       └── routes/            # Endpoints REST
│   └── workshop-viewer/          # Frontend Desktop
│       ├── index.css              # Estilos (inlinados)
│       ├── assets/                # Tokens de tema
│       └── src/
│           ├── main.rs
│           ├── api.rs             # Cliente HTTP
│           ├── pages/             # 13 páginas
│           └── components/        # Atomic Design
│               ├── atoms/         # Button, Input, Icon, Spinner
│               ├── molecules/     # Card, Modal, ConfirmModal
│               └── organisms/     # Header, DataTable, ServerSettings
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
cargo build --release -p workshop-server -p workshop-viewer
cargo wix -p workshop-server --nocapture
cargo wix -p workshop-viewer --nocapture

# Linux: .deb
cargo deb -p workshop-server
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
