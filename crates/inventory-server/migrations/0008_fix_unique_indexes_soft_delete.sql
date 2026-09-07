-- Fix: índices únicos que ignoraban soft-delete
-- Eliminar índices antiguos que no excluían productos eliminados
DROP INDEX IF EXISTS idx_products_workshop_barcode;
DROP INDEX IF EXISTS idx_products_workshop_sku;

-- Recrear con filtro que excluye productos soft-deleted
CREATE UNIQUE INDEX idx_products_workshop_barcode ON products(workshop_id, barcode)
    WHERE barcode IS NOT NULL AND status != 'deleted';

CREATE UNIQUE INDEX idx_products_workshop_sku ON products(workshop_id, sku)
    WHERE sku IS NOT NULL AND status != 'deleted';
