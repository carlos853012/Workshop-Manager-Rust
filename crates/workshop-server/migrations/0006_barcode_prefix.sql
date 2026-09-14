-- Agregar barcode_prefix a workshops
ALTER TABLE workshops ADD COLUMN barcode_prefix VARCHAR(6);