-- Initial schema for WorkshopManager

-- Custom enums (must match inventory-common sqlx::Type definitions)
CREATE TYPE payment_method AS ENUM ('cash', 'card', 'transfer');
CREATE TYPE repair_status AS ENUM ('pending', 'in_progress', 'completed', 'cancelled');
CREATE TYPE priority AS ENUM ('high', 'medium', 'low');
CREATE TYPE user_role AS ENUM ('admin', 'mechanic', 'seller');

-- Suppliers table (no dependencies)
CREATE TABLE suppliers (
    id UUID PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    contact_person VARCHAR(200),
    email VARCHAR(200),
    phone VARCHAR(20),
    address TEXT,
    tax_id VARCHAR(50),
    payment_terms VARCHAR(200),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Users table (no dependencies)
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(200) NOT NULL UNIQUE,
    display_name VARCHAR(200),
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Products table (depends on suppliers)
CREATE TABLE products (
    id UUID PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    brand VARCHAR(100),
    model VARCHAR(100),
    sku VARCHAR(100),
    price NUMERIC(19,4) NOT NULL,
    cost NUMERIC(19,4) NOT NULL,
    stock INTEGER NOT NULL DEFAULT 0,
    min_stock INTEGER NOT NULL DEFAULT 0,
    location VARCHAR(200),
    supplier_id UUID REFERENCES suppliers(id),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_products_name ON products(name);
CREATE INDEX idx_products_sku ON products(sku);

-- Sales table (no dependencies)
CREATE TABLE sales (
    id UUID PRIMARY KEY,
    customer_name VARCHAR(200),
    customer_email VARCHAR(200),
    customer_phone VARCHAR(20),
    total NUMERIC(19,4) NOT NULL,
    payment_method payment_method NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'completed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sales_created_at ON sales(created_at);

-- Sale items table (depends on sales and products)
CREATE TABLE sale_items (
    id UUID PRIMARY KEY,
    sale_id UUID NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    product_name VARCHAR(200),
    quantity INTEGER NOT NULL,
    unit_price NUMERIC(19,4) NOT NULL,
    total NUMERIC(19,4) NOT NULL
);

CREATE INDEX idx_sale_items_sale_id ON sale_items(sale_id);

-- Repairs table (depends on users)
CREATE TABLE repairs (
    id UUID PRIMARY KEY,
    customer_name VARCHAR(200),
    customer_email VARCHAR(200),
    customer_phone VARCHAR(20),
    motorcycle VARCHAR(200),
    license_plate VARCHAR(50),
    description TEXT,
    diagnosis TEXT,
    technician_id UUID REFERENCES users(id),
    estimated_delivery DATE,
    priority priority NOT NULL,
    status repair_status NOT NULL,
    estimated_cost NUMERIC(19,4),
    final_cost NUMERIC(19,4),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_repairs_status ON repairs(status);
CREATE INDEX idx_repairs_license_plate ON repairs(license_plate);
CREATE INDEX idx_repairs_created_at ON repairs(created_at);

-- Repair updates table (depends on repairs and users)
CREATE TABLE repair_updates (
    id UUID PRIMARY KEY,
    repair_id UUID NOT NULL REFERENCES repairs(id) ON DELETE CASCADE,
    status repair_status,
    description TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_repair_updates_repair_id ON repair_updates(repair_id);

-- Audit log table (depends on users)
CREATE TABLE audit_log (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    entity_type VARCHAR(100),
    entity_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at);
