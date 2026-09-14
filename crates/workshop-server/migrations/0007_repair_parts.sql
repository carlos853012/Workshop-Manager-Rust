-- Tabla para rastrear piezas/insumos usados en reparaciones
CREATE TABLE repair_parts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repair_id UUID NOT NULL REFERENCES repairs(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    quantity NUMERIC(10,2) NOT NULL DEFAULT 1,
    unit_cost NUMERIC(19,4),
    total_cost NUMERIC(19,4),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_repair_parts_repair_id ON repair_parts(repair_id);