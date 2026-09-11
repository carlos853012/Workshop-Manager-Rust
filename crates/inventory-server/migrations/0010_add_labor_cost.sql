-- Add labor_cost column to repairs for calculating total repair cost (labor + parts)
ALTER TABLE repairs ADD COLUMN labor_cost NUMERIC(19,4);
