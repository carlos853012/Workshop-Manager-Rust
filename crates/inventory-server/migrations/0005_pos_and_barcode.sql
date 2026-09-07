-- POS y códigos de barras
-- Agregar workshop_id a products (nullable primero para DBs existentes)
ALTER TABLE products ADD COLUMN workshop_id UUID REFERENCES workshops(id);

-- Agregar barcode a products
ALTER TABLE products ADD COLUMN barcode VARCHAR(50);

-- Índice único por taller para barcode
CREATE UNIQUE INDEX idx_products_workshop_barcode ON products(workshop_id, barcode) WHERE barcode IS NOT NULL;

-- Índice único por taller para sku
CREATE UNIQUE INDEX idx_products_workshop_sku ON products(workshop_id, sku) WHERE sku IS NOT NULL;

-- Agregar workshop_id a sales (nullable primero)
ALTER TABLE sales ADD COLUMN workshop_id UUID REFERENCES workshops(id);

-- Agregar workshop_id a suppliers (nullable primero)
ALTER TABLE suppliers ADD COLUMN workshop_id UUID REFERENCES workshops(id);

-- Agregar workshop_id a repairs (nullable primero)
ALTER TABLE repairs ADD COLUMN workshop_id UUID REFERENCES workshops(id);

-- Agregar campos de impuestos a sales
ALTER TABLE sales ADD COLUMN subtotal NUMERIC(19,4) NOT NULL DEFAULT 0;
ALTER TABLE sales ADD COLUMN discount_amount NUMERIC(19,4) NOT NULL DEFAULT 0;
ALTER TABLE sales ADD COLUMN taxable_amount NUMERIC(19,4) NOT NULL DEFAULT 0;
ALTER TABLE sales ADD COLUMN tax_amount NUMERIC(19,4) NOT NULL DEFAULT 0;

-- Poblar workshop_id existente en products, sales, suppliers, repairs
UPDATE products SET workshop_id = (SELECT id FROM workshops LIMIT 1) WHERE workshop_id IS NULL;
UPDATE sales SET workshop_id = (SELECT id FROM workshops LIMIT 1) WHERE workshop_id IS NULL;
UPDATE suppliers SET workshop_id = (SELECT id FROM workshops LIMIT 1) WHERE workshop_id IS NULL;
UPDATE repairs SET workshop_id = (SELECT id FROM workshops LIMIT 1) WHERE workshop_id IS NULL;

-- Ahora hacer NOT NULL después del backfill
ALTER TABLE products ALTER COLUMN workshop_id SET NOT NULL;
ALTER TABLE sales ALTER COLUMN workshop_id SET NOT NULL;
ALTER TABLE suppliers ALTER COLUMN workshop_id SET NOT NULL;
ALTER TABLE repairs ALTER COLUMN workshop_id SET NOT NULL;

-- Índices para consultas por taller
CREATE INDEX idx_sales_workshop_id ON sales(workshop_id);
CREATE INDEX idx_suppliers_workshop_id ON suppliers(workshop_id);
CREATE INDEX idx_repairs_workshop_id ON repairs(workshop_id);
