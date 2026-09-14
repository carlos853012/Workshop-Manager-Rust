-- Add product_id FK to repair_parts to link parts with inventory products
ALTER TABLE repair_parts
  ADD COLUMN product_id UUID REFERENCES products(id) ON DELETE SET NULL;

CREATE INDEX idx_repair_parts_product_id ON repair_parts(product_id);
