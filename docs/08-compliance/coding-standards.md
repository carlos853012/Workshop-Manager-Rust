# Coding Standards — WorkshopManager

**Version:** 0.1.0
**Status:** Phase 3 of 8 Complete
**Last Updated:** 2026-09-13
**Audience:** Developers, code reviewers

---

## 1. Rust Coding Standards

### 1.1 Rust API Guidelines

WorkshopManager follows the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

| Guideline | Implementation | Enforcement |
|-----------|---------------|-------------|
| Naming | `snake_case` for functions/variables, `PascalCase` for types | `cargo fmt` |
| Documentation | Public items documented with `///` | `cargo clippy` |
| Error handling | `Result<T, E>` for fallible operations | Constitutional rule |
| Type safety | Newtype pattern, strongly typed enums | Code review |
| Lifetime management | Minimal lifetimes, prefer owned types | Code review |

### 1.2 Code Style

```rust
// GOOD: Clear, idiomatic Rust
pub async fn create_product(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateProductRequest>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    let product = sqlx::query_as!(
        Product,
        r#"
        INSERT INTO products (id, name, price, workshop_id)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        Uuid::new_v4(),
        request.name,
        request.price,
        user.workshop_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse::success(product)))
}

// BAD: Avoid these patterns
pub fn create_product(state: &AppState, name: &str, price: f64) -> Product {
    // No error handling
    // No authentication
    // Raw SQL string concatenation
    let query = format!("INSERT INTO products ... {}", name); // SQL injection!
    Product { name: name.to_string(), price }
}
```

### 1.3 Module Organization

```
crates/
├── inventory-common/
│   ├── src/
│   │   ├── lib.rs          # Module declarations
│   │   ├── types.rs        # Domain types
│   │   ├── enums.rs        # Enumerations
│   │   ├── dto.rs          # Data transfer objects
│   │   └── ...
├── inventory-server/
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── routes/         # Route handlers
│   │   ├── middleware.rs    # Auth middleware
│   │   ├── auth.rs         # Authentication
│   │   ├── crypto.rs       # Encryption
│   │   ├── audit.rs        # Audit logging
│   │   └── ...
└── inventory-viewer/
    ├── src/
    │   ├── main.rs         # Entry point
    │   ├── components/     # Atomic Design components
    │   │   ├── atoms/
    │   │   ├── molecules/
    │   │   └── organisms/
    │   ├── pages/          # Page components
    │   └── api.rs          # API client
```

---

## 2. Security Coding Practices

### 2.1 Constitutional Rules

All code MUST comply with `CONSTITUCION.md`:

| Rule | Description | Enforcement |
|------|-------------|-------------|
| No `unsafe` | Zero tolerance for unsafe code | Code review |
| No `unwrap()` | Use `Result<T, E>` instead | `cargo clippy` |
| No `expect()` | Use proper error handling | `cargo clippy` |
| No `panic!()` | Use `Result<T, E>` instead | `cargo clippy` |
| Parameterized SQL | Never concatenate strings | Code review |
| Encrypt credentials | Use `crypto::encrypt_opt`/`decrypt_opt` | Code review |
| Validate inputs | Before database insertion | Code review |

### 2.2 Input Validation

```rust
// Email validation
fn validate_email(email: &str) -> Result<(), ValidationError> {
    let regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")
        .map_err(|_| ValidationError::InvalidFormat)?;
    
    if email.len() > 200 {
        return Err(ValidationError::TooLong);
    }
    
    if !regex.is_match(email) {
        return Err(ValidationError::InvalidFormat);
    }
    
    Ok(())
}

// Password validation
fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::TooShort);
    }
    
    // No maximum length enforced (OWASP recommendation)
    
    Ok(())
}

// UUID validation
fn validate_uuid(uuid: &str) -> Result<Uuid, ValidationError> {
    Uuid::parse_str(uuid).map_err(|_| ValidationError::InvalidFormat)
}
```

### 2.3 SQL Injection Prevention

```rust
// GOOD: Parameterized query
let user = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE email = $1 AND workshop_id = $2",
    email,
    workshop_id
)
.fetch_optional(&state.db)
.await?;

// BAD: String concatenation (NEVER DO THIS)
let query = format!("SELECT * FROM users WHERE email = '{}'", email);
// This is vulnerable to SQL injection!
```

---

## 3. Error Handling Standards

### 3.1 Error Types

```rust
// Define domain-specific errors
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

// Convert to HTTP responses
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
            AppError::Auth(e) => (StatusCode::UNAUTHORIZED, e),
            AppError::Validation(e) => (StatusCode::UNPROCESSABLE_ENTITY, e),
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, e),
            AppError::Internal(e) => {
                tracing::error!("Internal error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
        };
        
        (status, Json(ApiResponse::error(message))).into_response()
    }
}
```

### 3.2 Error Handling Rules

| Rule | Description |
|------|-------------|
| Never leak DB errors | Map to generic "Internal error" |
| Never panic | Use `Result<T, E>` everywhere |
| Log all errors | Use `tracing::error!` |
| Return meaningful messages | Help users understand what went wrong |
| Use `?` operator | Propagate errors cleanly |

---

## 4. Logging Standards

### 4.1 Log Levels

| Level | Usage | Example |
|-------|-------|---------|
| `ERROR` | System errors, failures | Database connection failed |
| `WARN` | Unexpected conditions | Rate limit exceeded |
| `INFO` | Business events | User logged in, sale created |
| `DEBUG` | Development debugging | SQL query executed |
| `TRACE` | Verbose debugging | Request/response details |

### 4.2 Log Format

```rust
// Structured logging with tracing
tracing::info!(
    user_id = %user.id,
    action = "login",
    email = %user.email,
    "User logged in successfully"
);

// Error logging with context
tracing::error!(
    error = %e,
    user_id = %user.id,
    endpoint = "POST /api/products",
    "Failed to create product"
);
```

### 4.3 Sensitive Data Redaction

```rust
// NEVER log sensitive data
tracing::info!(email = %user.email, "User logged in"); // OK
tracing::info!(password = %password, "Login attempt"); // NEVER!

// Redact in audit logs
fn redact_sensitive(value: &str) -> String {
    if value.contains("password") || value.contains("secret") {
        "[REDACTED]".to_string()
    } else {
        value.to_string()
    }
}
```

---

## 5. Documentation Standards

### 5.1 Rustdoc Comments

```rust
/// Creates a new product in the database.
///
/// # Arguments
///
/// * `state` - Application state with database pool
/// * `user` - Authenticated user with admin role
/// * `request` - Product creation data
///
/// # Returns
///
/// Returns `Ok(Product)` on success, `Err(AppError)` on failure.
///
/// # Errors
///
/// * `AppError::Database` - If database operation fails
/// * `AppError::Validation` - If input validation fails
///
/// # Examples
///
/// ```rust
/// let product = create_product(state, user, request).await?;
/// ```
pub async fn create_product(
    state: State<AppState>,
    user: Extension<AuthenticatedUser>,
    request: Json<CreateProductRequest>,
) -> Result<Json<ApiResponse<Product>>, AppError> {
    // Implementation
}
```

### 5.2 Module Documentation

```rust
//! # Authentication Module
//!
//! This module handles user authentication, including:
//! - Password hashing with Argon2id
//! - JWT token creation and validation
//! - Rate limiting for login attempts
//!
//! # Security
//!
//! All passwords are hashed before storage using Argon2id with:
//! - 64 MB memory cost
//! - 3 iterations
//! - 4 parallelism
//!
//! JWT tokens expire after 8 hours and cannot be revoked (stateless design).
```

### 5.3 README Requirements

Each crate should have a README with:
- Purpose and description
- Installation instructions
- Usage examples
- API reference link
- License information

---

## 6. Naming Conventions

### 6.1 Rust Naming

| Element | Convention | Example |
|---------|-----------|---------|
| Functions | `snake_case` | `create_product` |
| Variables | `snake_case` | `user_id` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_RETRY_COUNT` |
| Types | `PascalCase` | `Product`, `CreateProductRequest` |
| Enums | `PascalCase` | `UserRole`, `RepairStatus` |
| Enum variants | `PascalCase` | `UserRole::Admin` |
| Modules | `snake_case` | `auth`, `routes` |
| Files | `snake_case` | `auth.rs`, `routes.rs` |

### 6.2 Database Naming

| Element | Convention | Example |
|---------|-----------|---------|
| Tables | `snake_case` (plural) | `users`, `products` |
| Columns | `snake_case` | `created_at`, `workshop_id` |
| Primary keys | `id` | `id` |
| Foreign keys | `<table>_id` | `user_id`, `workshop_id` |
| Indexes | `idx_<table>_<column>` | `idx_users_email` |

### 6.3 API Naming

| Element | Convention | Example |
|---------|-----------|---------|
| Endpoints | `kebab-case` | `/api/device-keys` |
| Query params | `snake_case` | `?page_size=20` |
| JSON fields | `snake_case` | `"workshop_id"` |
| Headers | `PascalCase-Kebab` | `X-WorkshopManager-Key` |

---

## 7. File Organization

### 7.1 Server File Structure

```
crates/inventory-server/src/
├── main.rs                 # Entry point, server setup
├── config.rs               # Configuration handling
├── error.rs                # Error types
├── auth.rs                 # Authentication logic
├── crypto.rs               # Encryption/decryption
├── audit.rs                # Audit logging
├── backup.rs               # Backup scheduler
├── device_key.rs           # Device key management
├── rate_limiter.rs         # Rate limiting
├── tls.rs                  # TLS certificate handling
├── middleware.rs            # Auth middleware
├── routes/
│   ├── mod.rs              # Route declarations
│   ├── auth.rs             # Auth routes
│   ├── products.rs         # Product routes
│   ├── sales.rs            # Sale routes
│   ├── repairs.rs          # Repair routes
│   ├── suppliers.rs        # Supplier routes
│   ├── reports.rs          # Report routes
│   ├── users.rs            # User management routes
│   └── device_keys.rs      # Device key routes
└── migrations/             # SQL migrations
```

### 7.2 Viewer File Structure

```
crates/inventory-viewer/src/
├── main.rs                 # Entry point
├── api.rs                  # API client
├── auth.rs                 # Auth context
├── i18n.rs                 # Internationalization
├── components/
│   ├── atoms/              # Basic UI elements
│   │   ├── badge.rs
│   │   ├── button.rs
│   │   ├── icon.rs
│   │   ├── input.rs
│   │   └── spinner.rs
│   ├── molecules/          # Combined elements
│   │   ├── card.rs
│   │   ├── confirm_modal.rs
│   │   ├── form_group.rs
│   │   ├── modal.rs
│   │   └── tooltip.rs
│   └── organisms/          # Complex components
│       ├── connection_settings.rs
│       ├── data_table.rs
│       └── header.rs
└── pages/                  # Page components
    ├── home.rs
    ├── login.rs
    ├── setup.rs
    ├── products.rs
    ├── sales.rs
    ├── pos.rs
    ├── repairs.rs
    ├── suppliers.rs
    ├── reports.rs
    ├── users.rs
    └── device_keys.rs
```

---

## 8. Code Review Checklist

### 8.1 Security Review

- [ ] No `unsafe` code
- [ ] No `unwrap()`, `expect()`, or `panic!()`
- [ ] Parameterized SQL only
- [ ] Input validation before DB operations
- [ ] Sensitive data encrypted
- [ ] No secrets in logs
- [ ] Proper error handling

### 8.2 Quality Review

- [ ] Follows naming conventions
- [ ] Proper documentation
- [ ] Tests included
- [ ] No dead code
- [ ] Performance considerations
- [ ] Memory efficiency

### 8.3 Compliance Review

- [ ] Constitutional rules followed
- [ ] Audit logging implemented
- [ ] RBAC enforced
- [ ] Error messages sanitized
- [ ] No sensitive data exposure

---

## 9. Cross-References

| Document | Description |
|----------|-------------|
| [CONSTITUCION.md](../../CONSTITUCION.md) | Constitutional rules |
| [Conventions](../07-developer-guide/conventions.md) | Development conventions |
| [Contributing](../07-developer-guide/contributing.md) | Contribution guidelines |
| [Project Structure](../07-developer-guide/project-structure.md) | Code organization |
| [OWASP Top 10](./owasp-top10.md) | Security compliance |
