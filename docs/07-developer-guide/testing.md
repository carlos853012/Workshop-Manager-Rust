# Testing Strategy

This document describes the testing approach, patterns, and best practices for WorkshopManager.

---

## Test Organization

Tests are **inline unit tests** within source files, following Rust convention:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(result, expected);
    }
}
```

**Current coverage**: 82+ tests across 3 crates.

### Test Distribution

| Crate | Focus Areas |
|-------|------------|
| `workshop-common` | Money calculations, patent validation, DTO serialization, license verification |
| `workshop-server` | Auth, crypto, error handling, middleware, route logic |
| `workshop-viewer` | Component rendering, state management (planned) |

---

## Running Tests

```powershell
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p workshop-server
cargo test -p workshop-common

# Run specific test
cargo test test_name

# Run tests with output
cargo test --workspace -- --nocapture

# Run tests matching pattern
cargo test --workspace -- auth
```

---

## Writing New Tests

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_input_returns_ok() {
        let result = validate("valid");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_input_returns_err() {
        let result = validate("");
        assert!(result.is_err());
    }
}
```

### Naming Convention

```
test_<function>_<scenario>_<expected>
```

Examples:
- `test_validate_empty_name_returns_err`
- `test_calculate_total_with_discount`
- `test_auth_valid_token_returns_user`

### Test Patterns by Module

#### Money Calculations

```rust
#[test]
fn test_calculate_total_with_items() {
    let items = vec![
        SaleItem { quantity: 2, unit_price: dec!(10.00), ..Default::default() },
        SaleItem { quantity: 1, unit_price: dec!(25.50), ..Default::default() },
    ];
    let total = calculate_total(&items);
    assert_eq!(total, dec!(45.50));
}
```

#### Auth / Crypto

```rust
#[test]
fn test_jwt_roundtrip() {
    let secret = b"test-secret";
    let token = generate_token("user-id", secret).unwrap();
    let claims = verify_token(&token, secret).unwrap();
    assert_eq!(claims.user_id, "user-id");
}

#[test]
fn test_password_hash_roundtrip() {
    let password = "secure-password";
    let hash = hash_password(password).unwrap();
    assert!(verify_password(password, &hash).unwrap());
}
```

#### Error Handling

```rust
#[test]
fn test_sanitize_db_error_duplicate_key() {
    let msg = sanitize_db_error("duplicate key value violates unique constraint");
    assert_eq!(msg, "El recurso ya existe");
}

#[test]
fn test_sanitize_db_error_generic() {
    let msg = sanitize_db_error("something unexpected");
    assert_eq!(msg, "Error interno del servidor");
}
```

#### Input Validation

```rust
#[test]
fn test_validate_product_name_too_long() {
    let name = "x".repeat(201);
    let result = validate_product_name(&name);
    assert!(result.is_err());
}

#[test]
fn test_validate_email_invalid_format() {
    let result = validate_email("not-an-email");
    assert!(result.is_err());
}
```

---

## Mocking Strategies

### Database Mocking

For unit tests that need DB interaction without a real database:

```rust
// Use trait-based abstraction
trait Database {
    async fn get_product(&self, id: Uuid) -> Result<Product, AppError>;
}

// Mock implementation for tests
struct MockDb {
    products: HashMap<Uuid, Product>,
}

impl Database for MockDb {
    async fn get_product(&self, id: Uuid) -> Result<Product, AppError> {
        self.products.get(&id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Not found".into()))
    }
}
```

### API Response Mocking

```rust
// Mock HTTP responses for viewer tests
fn mock_api_response(data: serde_json::Value) -> Response {
    Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(data.to_string())
        .unwrap()
}
```

---

## Integration Test Plans

Integration tests are planned for future implementation:

### Server Integration Tests

```
tests/
├── auth_flows.rs         # Login → Token → Refresh → Logout
├── product_crud.rs       # Create → Read → Update → Delete
├── sales_flow.rs         # Sale creation with stock deduction
├── repair_lifecycle.rs   # Repair status transitions
├── multi_tenant.rs       # Workshop isolation verification
└── rate_limiting.rs      # Rate limiter behavior
```

### Viewer Integration Tests

```
tests/
├── navigation.rs         # Route transitions
├── form_submission.rs    # Form → API → State update
└── error_handling.rs     # Error display flows
```

---

## Test Data Management

### Fixtures

Create reusable test data:

```rust
fn make_test_product() -> Product {
    Product {
        id: Uuid::new_v4(),
        name: "Test Product".into(),
        price: dec!(29.99),
        cost: dec!(15.00),
        stock: 100,
        min_stock: 10,
        workshop_id: Uuid::new_v4(),
        ..Default::default()
    }
}
```

### Test Constants

```rust
const TEST_WORKSHOP_ID: &str = "550e8400-e29b-41d4-a716-446655440000";
const TEST_USER_EMAIL: &str = "test@workshop.com";
```

### Cleanup

- Tests must not leave data in shared state
- Use unique IDs per test to avoid conflicts
- Transaction rollback for DB tests (when implemented)

---

## Coverage Metrics

| Area | Target | Current |
|------|--------|---------|
| Unit tests | 80%+ functions | ~60% |
| Edge cases | All validation paths | Partial |
| Error paths | All AppError variants | Most covered |

---

## CI Testing

The CI pipeline runs on every push:

```yaml
- cargo fmt --all --check
- cargo clippy --workspace -- -D warnings
- cargo test --workspace
```

All checks must pass before merging.
