# Coding Conventions

This document defines coding standards for all WorkshopManager crates.

---

## Rust Style

### Safety Rules

```rust
// ❌ FORBIDDEN
unsafe { /* anything */ }
let value = option.unwrap();
let value = option.expect("msg");
panic!("error");

// ✅ REQUIRED
match result {
    Ok(val) => val,
    Err(e) => return Err(AppError::Internal(e.to_string())),
}
// or
let value = result.map_err(|e| AppError::Internal(e.to_string()))?;
```

**Exceptions** (document justification in code):
- `unwrap()` in tests only
- `expect()` when failure is impossible by construction
- `unsafe` only with documented invariants and no safe alternative

### Error Handling

Use the `AppError` enum for all server errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Too many requests")]
    TooManyRequests,
}
```

Map errors properly:

```rust
// Database errors → sanitized messages
AppError::Internal(sqlx_error.to_string())

// Validation errors
AppError::Validation("Name must be 1-200 characters".into())

// Not found
AppError::NotFound("Product not found".into())
```

Never expose internal details to clients. Use `sanitize_db_error()` for DB errors.

---

## SQL Parameterization

**Never concatenate strings into SQL queries.**

```rust
// ❌ SQL INJECTION VULNERABILITY
format!("SELECT * FROM products WHERE name = '{}'", user_input)

// ✅ Parameterized query
sqlx::query_as!(
    Product,
    "SELECT * FROM products WHERE name = $1 AND workshop_id = $2",
    user_input,
    workshop_id
)
```

All queries must use `sqlx::query!` or `sqlx::query_as!` macros for compile-time checked SQL.

---

## Frontend Patterns

### Atomic Design Hierarchy

```
atoms/ → molecules/ → organisms/ → pages/
```

**Atoms** — Single-purpose UI primitives:
- `Button`, `Input`, `Icon`, `Spinner`, `Badge`

**Molecules** — Combinations of atoms:
- `Card`, `Modal`, `ConfirmModal`, `FormGroup`, `Tooltip`

**Organisms** — Complex sections with logic:
- `DataTable`, `Header`, `ServerSettings`, form modals

**Rule**: Never create modals inline in pages. Always extract to `organisms/`.

### CSS Token Usage

```css
/* ❌ HARDCODED */
.button { background: #3b82f6; padding: 8px; }

/* ✅ USING TOKENS */
.button { background: var(--color-primary); padding: var(--space-2); }
```

All colors, spacing, typography, borders, and shadows must use CSS variables from `tokens-*.toml`.

### Table Pattern

Every table MUST follow this structure:

```rust
rsx! {
    div { class: "data-table-wrapper",
        table { class: "data-table",
            thead { /* headers */ }
            tbody { /* rows */ }
        }
    }
}
```

CSS handles horizontal scrolling via `overflow-x: auto` on the wrapper.

### Modal Pattern

Every modal MUST use the `Modal` molecule with a standard footer:

```rust
rsx! {
    Modal { title: "Edit Product",
        // Modal body content
        div { class: "modal-footer",
            Button {
                label: "Cancel",
                variant: ButtonVariant::Ghost,
                onclick: |_| close_modal.call(()),
            }
            Button {
                label: "Save",
                variant: ButtonVariant::Primary,
                onclick: |_| save.call(()),
            }
        }
    }
}
```

### Loading States

```rust
// ✅ Consistent empty/loading state
div { class: "empty-state",
    Spinner {}
}
```

### Error Display

All API errors MUST surface to the user:

```rust
// ❌ Silent discard
let _ = api_call().await;

// ✅ Show error to user
match api_call().await {
    Ok(data) => set_data(data),
    Err(e) => alert.set(e.to_string()),
}
```

---

## Signal Passing

Use explicit props, never implicit context for component data:

```rust
#[derive(Props, PartialEq)]
struct MyProps {
    product_id: uuid::Uuid,
    on_close: EventHandler<()>,
}

#[component]
fn MyComponent(props: MyProps) -> Element {
    // Access props.product_id, props.on_close
}
```

---

## i18n Rules

| Context | Language | Location |
|---------|----------|----------|
| Enum `Display` traits | English | `inventory-common` |
| API responses / DB values | English | Server-side |
| UI labels | Spanish | `i18n.rs` |

```rust
// inventory-common: Display in English
impl fmt::Display for RepairStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::InProgress => write!(f, "InProgress"),
            // ...
        }
    }
}

// inventory-viewer: UI translations in i18n.rs
pub fn translate_repair_status(status: &RepairStatus) -> &'static str {
    match status {
        RepairStatus::Pending => "Pendiente",
        RepairStatus::InProgress => "En Progreso",
        // ...
    }
}
```

---

## Git Conventions

### Commit Messages

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`

**Examples**:
```
feat(products): add barcode scanning in POS
fix(auth): prevent token refresh race condition
refactor(server): extract pagination module
docs(api): update repair endpoints
```

### Branch Naming

```
feat/feature-name
fix/issue-description
refactor/module-name
release/v0.2.0
```

---

## Responsive Design

All layouts MUST include mobile rules:

```css
@media (max-width: 768px) {
    .modal { min-width: 90vw; }
    .data-table-wrapper { overflow-x: auto; }
}
```

---

## Performance

- Avoid unnecessary `.clone()` — use references when possible
- Use `Cow<'_, str>` for strings that may or may not need allocation
- Profile before optimizing — correct first, fast second
