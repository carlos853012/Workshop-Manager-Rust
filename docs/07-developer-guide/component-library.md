# Component Library

Atomic Design component reference for the Dioxus Desktop viewer.

---

## Design System Overview

```
atoms/       → Basic UI primitives (Button, Input, Icon, Spinner, Badge)
molecules/   → Composed atom combinations (Card, Modal, ConfirmModal)
organisms/   → Complex UI sections (DataTable, Header, form modals)
pages/       → Full route components
```

---

## Atoms

### Button

Interactive button with variants.

```rust
#[derive(Props, PartialEq)]
pub struct ButtonProps {
    pub label: &'static str,
    pub variant: ButtonVariant,
    pub onclick: EventHandler<()>,
    #[prop(default = false)]
    pub disabled: bool,
}

pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}
```

**Usage**:
```rust
Button {
    label: "Save",
    variant: ButtonVariant::Primary,
    onclick: |_| save.call(()),
}
```

**Rules**:
- Cancel actions → `ButtonVariant::Ghost`
- Destructive actions → `ButtonVariant::Danger`
- Primary actions → `ButtonVariant::Primary`

### Input

Text input with label and validation.

```rust
#[derive(Props, PartialEq)]
pub struct InputProps {
    pub label: &'static str,
    pub value: Signal<String>,
    #[prop(default = "")]
    pub placeholder: &'static str,
    #[prop(default = false)]
    pub input_type: bool, // true = password
}
```

### Icon

SVG icon component.

```rust
#[derive(Props, PartialEq)]
pub struct IconProps {
    pub name: &'static str,
    #[prop(default = 16)]
    pub size: u32,
}
```

### Spinner

Loading indicator.

```rust
// No props needed
rsx! { Spinner {} }
```

### Badge

Status indicator with color.

```rust
#[derive(Props, PartialEq)]
pub struct BadgeProps {
    pub label: &'static str,
    pub variant: BadgeVariant,
}
```

---

## Molecules

### Card

Content container with optional header.

```rust
#[derive(Props, PartialEq)]
pub struct CardProps {
    pub title: Option<&'static str>,
    pub children: Element,
}
```

**Usage**:
```rust
Card { title: "Product Details",
    div { class: "card-body",
        // Content
    }
}
```

### Modal

Dialog overlay with title and close button.

```rust
#[derive(Props, PartialEq)]
pub struct ModalProps {
    pub title: &'static str,
    pub children: Element,
    pub on_close: EventHandler<()>,
}
```

**Usage** (always with footer):
```rust
Modal { title: "Edit Item",
    div { class: "modal-body",
        // Form content
    }
    div { class: "modal-footer",
        Button { label: "Cancel", variant: ButtonVariant::Ghost, onclick: close }
        Button { label: "Save", variant: ButtonVariant::Primary, onclick: save }
    }
}
```

### ConfirmModal

Confirmation dialog with message.

```rust
#[derive(Props, PartialEq)]
pub struct ConfirmModalProps {
    pub title: &'static str,
    pub message: &'static str,
    pub on_confirm: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
}
```

---

## Organisms

### DataTable

Reusable data table with sorting and pagination.

```rust
#[derive(Props, PartialEq)]
pub struct DataTableProps<T: PartialEq> {
    pub columns: Vec<Column<T>>,
    pub data: Vec<T>,
    pub on_sort: Option<EventHandler<String>>,
}
```

**Structure**:
```rust
div { class: "data-table-wrapper",
    table { class: "data-table",
        thead { /* Column headers */ }
        tbody { /* Data rows */ }
    }
}
```

### Header

App header with navigation and user menu.

```rust
// No external props — reads from app state
rsx! { Header {} }
```

### ServerSettings

Server configuration form (admin only).

### UserFormModal

Create/edit user dialog.

### SaleDetailModal

Sale detail view with items list.

### RepairDetailModal

Repair detail view with status history.

### StockEntryModal

Stock adjustment dialog.

### ProductFormModal

Create/edit product dialog.

### SupplierFormModal

Create/edit supplier dialog.

### CertificateDetailModal

Service certificate detail view.

### Tabs (organisms/tabs/)

Tabbed interface container.

---

## CSS Tokens

### Theme System

Themes are defined in TOML files and inlined at compile time:

```
viewer/src/theme/
├── mod.rs          # Theme loading
├── tokens.rs       # Token parsing
└── tokens-light.toml
└── tokens-dark.toml
```

**Important**: CSS is inlined via `include_str!("../index.css")`. Rebuild the viewer to see CSS changes.

### Available Tokens

| Category | Tokens |
|----------|--------|
| Colors | `--color-primary`, `--color-secondary`, `--color-success`, `--color-danger`, `--color-warning` |
| Background | `--bg-primary`, `--bg-secondary`, `--bg-card`, `--bg-modal` |
| Text | `--text-primary`, `--text-secondary`, `--text-muted` |
| Spacing | `--space-1` through `--space-8` |
| Border | `--border-color`, `--border-radius` |
| Shadow | `--shadow-sm`, `--shadow-md`, `--shadow-lg` |

### Usage in CSS

```css
.my-component {
    background: var(--bg-card);
    color: var(--text-primary);
    padding: var(--space-4);
    border: 1px solid var(--border-color);
    border-radius: var(--border-radius);
    box-shadow: var(--shadow-sm);
}
```

---

## Creating New Components

### Step-by-Step

1. **Determine level**: atom, molecule, or organism?
2. **Create file** in appropriate `components/{level}/` directory
3. **Define props** struct with `#[derive(Props, PartialEq)]`
4. **Implement** the component function
5. **Export** in `mod.rs`
6. **Add CSS** using tokens (update `index.css` and rebuild)

### Example: New Atom

```rust
// components/atoms/status_indicator.rs
use dioxus::prelude::*;

#[derive(Props, PartialEq)]
pub struct StatusIndicatorProps {
    pub status: Status,
}

pub enum Status {
    Active,
    Inactive,
    Error,
}

#[component]
pub fn StatusIndicator(props: StatusIndicatorProps) -> Element {
    let color_class = match props.status {
        Status::Active => "status-active",
        Status::Inactive => "status-inactive",
        Status::Error => "status-error",
    };

    rsx! {
        span { class: "status-indicator {color_class}" }
    }
}
```

### Example: New Molecule

```rust
// components/molecules/info_card.rs
use dioxus::prelude::*;

#[derive(Props, PartialEq)]
pub struct InfoCardProps {
    pub title: String,
    pub value: String,
    pub icon: Option<&'static str>,
}

#[component]
pub fn InfoCard(props: InfoCardProps) -> Element {
    rsx! {
        div { class: "info-card",
            div { class: "info-card-header",
                if let Some(icon_name) = props.icon {
                    Icon { name: icon_name, size: 20 }
                }
                span { class: "info-card-title", "{props.title}" }
            }
            div { class: "info-card-value", "{props.value}" }
        }
    }
}
```

---

## Props Patterns

### Optional Props

```rust
#[derive(Props, PartialEq)]
pub struct MyProps {
    pub required: String,
    #[prop(default = "default".to_string())]
    pub optional: String,
}
```

### Event Handlers

```rust
#[derive(Props, PartialEq)]
pub struct MyProps {
    pub on_change: EventHandler<String>,
    #[prop(default = |_| {})]
    pub on_click: EventHandler<()>,
}
```

### Children

```rust
#[derive(Props, PartialEq)]
pub struct MyProps {
    pub children: Element,
}

// Usage
MyComponent {
    div { "Child content" }
}
```

### Signals

```rust
#[derive(Props, PartialEq)]
pub struct MyProps {
    pub value: Signal<String>,
    pub on_change: Signal<String>,
}
```
