# WorkshopManager - Guía para Crear el Proyecto desde Cero

## Prerequisitos

### Herramientas Necesarias

```bash
# Rust (instalar desde https://rustup.rs)
rustup update stable

# Herramientas de build
cargo install cargo-deb
cargo install cargo-wix

# WiX Toolset v3 (Windows, para MSI)
# Descargar desde: https://wixtoolset.org/releases/
# O: choco install wixtoolset

# Git
git init
```

### Estructura de Directorios Inicial

```
workshop-manager/
├── .cargo/
│   └── config.toml
├── .github/
│   └── workflows/
├── .opencode/
│   └── skills/
├── crates/
│   ├── workshop-common/
│   ├── workshop-server/
│   └── workshop-viewer/
├── config/
│   └── themes/
├── scripts/
├── tools/
│   └── keygen/
├── Cargo.toml
├── .gitignore
├── CONSTITUCION.md
├── AGENTS.md
├── COMMANDS.md
├── ARQUITECTURA.md
├── README.md
├── .clinerules
├── .cursorrules
└── .geminirules
```

---

## Paso 1: Crear Workspace Root

### 1.1 Inicializar Git

```bash
mkdir workshop-manager
cd workshop-manager
git init
```

### 1.2 Crear .gitignore

```gitignore
# Rust
/target/
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Datos sensibles
/dev-data/
*.key
*.pem
.env
secrets/

# Build artifacts
*.msi
*.deb
*.exe

# PostgreSQL embebido
/pgdata/
*.db
*.sqlite

# Logs
/logs/
*.log
```

### 1.3 Crear Cargo.toml (Workspace)

```toml
[workspace]
members = [
    "crates/workshop-common",
    "crates/workshop-server",
    "crates/workshop-viewer"
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "Proprietary"
authors = ["Tu Nombre <tu@email.com>"]

[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
opt-level = "z"
```

### 1.4 Crear .cargo/config.toml

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

### 1.5 Verificar que compila

```bash
# Crear estructura mínima
mkdir -p crates/workshop-common/src
mkdir -p crates/workshop-server/src
mkdir -p crates/workshop-viewer/src

# Crear archivos mínimos
echo 'pub fn hello() -> String { "Hello".to_string() }' > crates/workshop-common/src/lib.rs
echo 'fn main() { println!("Server"); }' > crates/workshop-server/src/main.rs
echo 'fn main() { println!("Viewer"); }' > crates/workshop-viewer/src/main.rs

# Crear Cargo.tomls de cada crate
cat > crates/workshop-common/Cargo.toml << 'EOF'
[package]
name = "workshop-common"
version.workspace = true
edition.workspace = true

[dependencies]
EOF

cat > crates/workshop-server/Cargo.toml << 'EOF'
[package]
name = "workshop-server"
version.workspace = true
edition.workspace = true

[[bin]]
name = "workshop-server"
path = "src/main.rs"

[dependencies]
workshop-common = { path = "../workshop-common" }
EOF

cat > crates/workshop-viewer/Cargo.toml << 'EOF'
[package]
name = "workshop-viewer"
version.workspace = true
edition.workspace = true

[[bin]]
name = "workshop-viewer"
path = "src/main.rs"

[dependencies]
workshop-common = { path = "../workshop-common" }
EOF

# Verificar compilación
cargo build --workspace
```

---

## Paso 2: Copiar Skills y Documentación

### 2.1 Copiar Skills de noc-system

```powershell
# Ejecutar desde PowerShell
$source = "C:\Users\carlos\Desktop\noc-system\.opencode\skills"
$dest = ".opencode\skills"

New-Item -ItemType Directory -Force -Path $dest
Copy-Item -Recurse -Force "$source\*" $dest
```

### 2.2 Crear CONSTITUCION.md

Copiar de Equipos-Rust y modificar:

```powershell
Copy-Item "C:\Users\carlos\Desktop\Equipos-Rust\CONSTITUCION.md" ".\CONSTITUCION.md"
```

Luego editar para eliminar sección 16 (OT/SCADA) y agregar reglas de taller.

### 2.3 Crear AGENTS.md

Copiar de Equipos-Rust y adaptar:

```powershell
Copy-Item "C:\Users\carlos\Desktop\Equipos-Rust\AGENTS.md" ".\AGENTS.md"
```

### 2.4 Crear COMMANDS.md

Copiar de Equipos-Rust y adaptar:

```powershell
Copy-Item "C:\Users\carlos\Desktop\Equipos-Rust\COMMANDS.md" ".\COMMANDS.md"
```

### 2.5 Crear archivos de asistentes IA

```powershell
Copy-Item "C:\Users\carlos\Desktop\Equipos-Rust\.clinerules" ".\.clinerules"
Copy-Item "C:\Users\carlos\Desktop\Equipos-Rust\.cursorrules" ".\.cursorrules"
Copy-Item "C:\Users\carlos\Desktop\Equipos-Rust\.geminirules" ".\.geminirules"
```

---

## Paso 3: Configurar workshop-common

### 3.1 Actualizar Cargo.toml

```toml
[package]
name = "workshop-common"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
sqlx = { version = "0.7", features = ["postgres", "chrono", "uuid"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
ed25519-dalek = "2.1"
```

### 3.2 Crear lib.rs

```rust
pub mod features;
pub mod license;
pub mod dto;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ==================== PRODUCTOS ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub sku: Option<String>,
    pub price: f64,
    pub cost: f64,
    pub stock: i32,
    pub min_stock: i32,
    pub location: Option<String>,
    pub supplier_id: Option<uuid::Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== VENTAS ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Sale {
    pub id: uuid::Uuid,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub total: f64,
    pub payment_method: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SaleItem {
    pub id: uuid::Uuid,
    pub sale_id: uuid::Uuid,
    pub product_id: uuid::Uuid,
    pub product_name: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub total: f64,
}

// ==================== REPARACIONES ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Repair {
    pub id: uuid::Uuid,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub motorcycle: Option<String>,
    pub license_plate: Option<String>,
    pub description: Option<String>,
    pub diagnosis: Option<String>,
    pub technician_id: Option<uuid::Uuid>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
    pub priority: String,
    pub status: String,
    pub estimated_cost: Option<f64>,
    pub final_cost: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RepairUpdate {
    pub id: uuid::Uuid,
    pub repair_id: uuid::Uuid,
    pub status: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: DateTime<Utc>,
}

// ==================== PROVEEDORES ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Supplier {
    pub id: uuid::Uuid,
    pub name: String,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub tax_id: Option<String>,
    pub payment_terms: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== USUARIOS ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub display_name: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// ==================== AUDITORÍA ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: i32,
    pub user_id: Option<uuid::Uuid>,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<uuid::Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ==================== ENUMS ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentMethod {
    #[serde(rename = "cash")]
    Cash,
    #[serde(rename = "card")]
    Card,
    #[serde(rename = "transfer")]
    Transfer,
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "cash"),
            PaymentMethod::Card => write!(f, "card"),
            PaymentMethod::Transfer => write!(f, "transfer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepairStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl std::fmt::Display for RepairStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepairStatus::Pending => write!(f, "pending"),
            RepairStatus::InProgress => write!(f, "in_progress"),
            RepairStatus::Completed => write!(f, "completed"),
            RepairStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    #[serde(rename = "high")]
    High,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "low")]
    Low,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::High => write!(f, "high"),
            Priority::Medium => write!(f, "medium"),
            Priority::Low => write!(f, "low"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "mechanic")]
    Mechanic,
    #[serde(rename = "seller")]
    Seller,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Mechanic => write!(f, "mechanic"),
            UserRole::Seller => write!(f, "seller"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_method_display() {
        assert_eq!(PaymentMethod::Cash.to_string(), "cash");
        assert_eq!(PaymentMethod::Card.to_string(), "card");
        assert_eq!(PaymentMethod::Transfer.to_string(), "transfer");
    }

    #[test]
    fn test_repair_status_display() {
        assert_eq!(RepairStatus::Pending.to_string(), "pending");
        assert_eq!(RepairStatus::InProgress.to_string(), "in_progress");
        assert_eq!(RepairStatus::Completed.to_string(), "completed");
        assert_eq!(RepairStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(Priority::High.to_string(), "high");
        assert_eq!(Priority::Medium.to_string(), "medium");
        assert_eq!(Priority::Low.to_string(), "low");
    }

    #[test]
    fn test_user_role_display() {
        assert_eq!(UserRole::Admin.to_string(), "admin");
        assert_eq!(UserRole::Mechanic.to_string(), "mechanic");
        assert_eq!(UserRole::Seller.to_string(), "seller");
    }
}
```

### 3.3 Crear features.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Feature {
    // Base
    Inventory,
    Sales,
    Repairs,
    Suppliers,
    Dashboard,
    Auth,
    AuditLog,
    MultiViewer,
    // DLC: Reports
    PdfReports,
    ClientHistory,
    ExcelExport,
    // DLC: Advanced
    AdvancedAnalytics,
    AutoBackup,
    MultiWorkshop,
    // DLC: API
    RestApi,
    Webhooks,
    Integrations,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LicenseTier {
    Base,
    Reports,
    Advanced,
    Api,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub key: String,
    pub tier: LicenseTier,
    pub features: Vec<Feature>,
    pub purchased_at: chrono::DateTime<chrono::Utc>,
    pub hardware_hash: Option<String>,
}

impl License {
    pub fn has_feature(&self, feature: &Feature) -> bool {
        self.features.contains(feature)
    }

    pub fn is_valid(&self) -> bool {
        !self.key.is_empty() && !self.features.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_has_feature() {
        let license = License {
            key: "TEST-1234".to_string(),
            tier: LicenseTier::Base,
            features: vec![Feature::Inventory, Feature::Sales],
            purchased_at: chrono::Utc::now(),
            hardware_hash: None,
        };

        assert!(license.has_feature(&Feature::Inventory));
        assert!(license.has_feature(&Feature::Sales));
        assert!(!license.has_feature(&Feature::PdfReports));
    }

    #[test]
    fn test_license_is_valid() {
        let valid = License {
            key: "TEST-1234".to_string(),
            tier: LicenseTier::Base,
            features: vec![Feature::Inventory],
            purchased_at: chrono::Utc::now(),
            hardware_hash: None,
        };
        assert!(valid.is_valid());

        let invalid = License {
            key: String::new(),
            tier: LicenseTier::Base,
            features: vec![],
            purchased_at: chrono::Utc::now(),
            hardware_hash: None,
        };
        assert!(!invalid.is_valid());
    }
}
```

### 3.4 Crear license.rs

```rust
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use sha2::{Sha256, Digest};
use crate::{License, features::Feature};

pub fn extract_hardware_hash(cpu_id: &str, motherboard: &str, disk: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cpu_id.as_bytes());
    hasher.update(motherboard.as_bytes());
    hasher.update(disk.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn validate_hardware(license: &License) -> bool {
    match &license.hardware_hash {
        Some(_) => true, // En producción, comparar con hardware actual
        None => true,    // Sin binding de hardware = válido
    }
}

pub fn verify_license_signature(license: &License, public_key: &VerifyingKey) -> bool {
    // En producción, verificar firma Ed25519
    // Por ahora, retornar true para desarrollo
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hardware_hash() {
        let hash1 = extract_hardware_hash("cpu1", "mb1", "disk1");
        let hash2 = extract_hardware_hash("cpu1", "mb1", "disk1");
        let hash3 = extract_hardware_hash("cpu2", "mb1", "disk1");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_validate_hardware_no_binding() {
        let license = License {
            key: "TEST".to_string(),
            tier: crate::features::LicenseTier::Base,
            features: vec![Feature::Inventory],
            purchased_at: chrono::Utc::now(),
            hardware_hash: None,
        };
        assert!(validate_hardware(&license));
    }
}
```

### 3.5 Crear dto.rs

```rust
use serde::{Deserialize, Serialize};

// ==================== REQUESTS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub sku: Option<String>,
    pub price: f64,
    pub cost: f64,
    pub stock: i32,
    pub min_stock: i32,
    pub location: Option<String>,
    pub supplier_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSaleRequest {
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub payment_method: String,
    pub items: Vec<SaleItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItemRequest {
    pub product_id: uuid::Uuid,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepairRequest {
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub motorcycle: Option<String>,
    pub license_plate: Option<String>,
    pub description: Option<String>,
    pub priority: String,
    pub estimated_cost: Option<f64>,
}

// ==================== RESPONSES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: crate::User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardResponse {
    pub total_products: i64,
    pub low_stock: i64,
    pub total_sales: i64,
    pub total_revenue: f64,
    pub pending_repairs: i64,
    pub completed_repairs: i64,
    pub average_sale: f64,
}
```

### 3.6 Verificar compilación

```bash
cargo build -p workshop-common
cargo test -p workshop-common
```

---

## Paso 4: Configurar workshop-server

### 4.1 Actualizar Cargo.toml

```toml
[package]
name = "workshop-server"
version.workspace = true
edition.workspace = true

[[bin]]
name = "workshop-server"
path = "src/main.rs"

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
rcgen = "0.13"
axum-server = { version = "0.7", features = ["tls-rustls"] }
rustls = "0.23"
```

### 4.2 Crear main.rs básico

```rust
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod state;
mod config;
mod secrets;
mod crypto;
mod auth;
mod error;
mod middleware;
mod rate_limiter;
mod audit;
mod schema;
mod db_manager;
mod tls;
mod backup;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Init tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .init();

    tracing::info!("Starting WorkshopManager Server...");

    // 2. Load config
    let config = config::load_config()?;
    tracing::info!("Config loaded");

    // 3. Init secrets
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("WorkshopManager")
        .join("data");
    std::fs::create_dir_all(&data_dir)?;
    let secrets = secrets::init_secrets(&data_dir)?;
    tracing::info!("Secrets initialized");

    // 4. Init crypto
    crypto::init(&data_dir)?;
    tracing::info!("Crypto initialized");

    // 5. Start PostgreSQL embedded
    let db_url = db_manager::setup(&data_dir).await?;
    tracing::info!("PostgreSQL started");

    // 6. Run migrations
    let pool = db_manager::create_pool(&db_url).await?;
    schema::run_migrations(&pool).await?;
    tracing::info!("Migrations completed");

    // 7. Create AppState
    let state = state::AppState::new(pool, secrets, config);

    // 8. Build router
    let app = Router::new()
        .route("/health", get(health))
        .nest("/api", routes::api_routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 9. Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
    tracing::info!("Server listening on {}", addr);

    axum::serve(
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
    )
    .await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}
```

### 4.3 Verificar compilación (sin dependencias faltantes)

```bash
cargo build -p workshop-server
```

---

## Paso 5: Configurar workshop-viewer

### 5.1 Actualizar Cargo.toml

```toml
[package]
name = "workshop-viewer"
version.workspace = true
edition.workspace = true

[[bin]]
name = "workshop-viewer"
path = "src/main.rs"

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

### 5.2 Crear main.rs básico

```rust
use dioxus::prelude::*;

mod theme;
mod api;
mod icons;
mod layout;
mod pages;
mod components;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {include_str!("../index.css")} }
        Router::<Route> {}
    }
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/login")]
    Login {},
    #[route("/products")]
    Products {},
    #[route("/sales")]
    Sales {},
    #[route("/repairs")]
    Repairs {},
    #[route("/suppliers")]
    Suppliers {},
}

#[component]
fn Home() -> Element {
    rsx! {
        h1 { "WorkshopManager" }
    }
}

// Stub routes
#[component]
fn Login() -> Element { rsx! { h1 { "Login" } } }
#[component]
fn Products() -> Element { rsx! { h1 { "Products" } } }
#[component]
fn Sales() -> Element { rsx! { h1 { "Sales" } } }
#[component]
fn Repairs() -> Element { rsx! { h1 { "Repairs" } } }
#[component]
fn Suppliers() -> Element { rsx! { h1 { "Suppliers" } } }
```

### 5.3 Crear index.css básico

```css
:root {
    --primary: #3b82f6;
    --primary-hover: #2563eb;
    --bg: #f8fafc;
    --panel: #ffffff;
    --text: #1e293b;
    --text-secondary: #64748b;
    --border: #e2e8f0;
    --success: #10b981;
    --warning: #f59e0b;
    --error: #ef4444;
    --radius-md: 0.5rem;
    --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1);
}

* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: system-ui, sans-serif; background: var(--bg); color: var(--text); }
```

### 5.4 Verificar compilación

```bash
cargo build -p workshop-viewer
```

---

## Paso 6: Verificar Workspace Completo

```bash
# Compilar todo
cargo build --workspace

# Linting
cargo clippy --workspace -- -D warnings

# Formatting
cargo fmt --all --check

# Tests
cargo test --workspace
```

---

## Paso 7: Crear Primer Commit

```bash
git add .
git commit -m "feat: initial workspace setup with 3 crates"
```

---

## Estructura Final Esperada

```
workshop-manager/
├── .cargo/config.toml
├── .github/workflows/
├── .opencode/skills/ (9 skills)
├── .gitignore
├── .clinerules
├── .cursorrules
├── .geminirules
├── crates/
│   ├── workshop-common/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── features.rs
│   │       ├── license.rs
│   │       └── dto.rs
│   ├── workshop-server/
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── workshop-viewer/
│       ├── Cargo.toml
│       ├── index.css
│       └── src/main.rs
├── config/
│   └── themes/
├── scripts/
├── tools/keygen/
├── Cargo.toml
├── CONSTITUCION.md
├── AGENTS.md
├── COMMANDS.md
├── ARQUITECTURA.md
└── README.md
```

---

## Siguientes Pasos

Una vez completada esta guía, seguir el **PLAN_DESARROLLO.md** para las fases 2-8.
