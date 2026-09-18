-- ============================================================
-- WorkshopManager — Seed de datos de prueba
-- Taller de motos chileno con datos realistas
-- ============================================================
-- USO:
--   psql -U postgres -d workshop_manager -f seed.sql
--
-- NOTAS:
--   - Password hash: Admin12345 (Argon2id)
--   - Precios incluyen IVA (19% Chile)
--   - UUIDs fijos para reproducibilidad
-- ============================================================

-- Workshop
INSERT INTO workshops (id, name, address, city, created_at, updated_at)
VALUES (
    'a0000000-0000-0000-0000-000000000001',
    'Taller MotoSur',
    'Av. San Martín 1234',
    'Santiago',
    NOW(), NOW()
) ON CONFLICT (id) DO NOTHING;

-- Usuarios
INSERT INTO users (id, email, display_name, password_hash, role, status, workshop_id, created_at)
VALUES
    ('b0000000-0000-0000-0000-000000000001', 'admin@taller.cl', 'Carlos Reyes',
     '$argon2id$v=19$m=65536,t=3,p=4$+qrsNODbN9DJXRHQGkj8pw$F3A8YucDkkFjOOaG/DJ7kaJhSeyvPCrd+tWUPoVvJ64',
     'admin', 'active', 'a0000000-0000-0000-0000-000000000001', NOW()),
    ('b0000000-0000-0000-0000-000000000002', 'vendedor@taller.cl', 'María López',
     '$argon2id$v=19$m=65536,t=3,p=4$+qrsNODbN9DJXRHQGkj8pw$F3A8YucDkkFjOOaG/DJ7kaJhSeyvPCrd+tWUPoVvJ64',
     'seller', 'active', 'a0000000-0000-0000-0000-000000000001', NOW()),
    ('b0000000-0000-0000-0000-000000000003', 'mecanico@taller.cl', 'Juan Pérez',
     '$argon2id$v=19$m=65536,t=3,p=4$+qrsNODbN9DJXRHQGkj8pw$F3A8YucDkkFjOOaG/DJ7kaJhSeyvPCrd+tWUPoVvJ64',
     'mechanic', 'active', 'a0000000-0000-0000-0000-000000000001', NOW())
ON CONFLICT (id) DO NOTHING;

-- Proveedores
INSERT INTO suppliers (id, name, contact_person, email, phone, address, tax_id, payment_terms, status, workshop_id, created_at, updated_at)
VALUES
    ('c0000000-0000-0000-0000-000000000001', 'ChileMotos SpA', 'Roberto Díaz', 'ventas@chilemotos.cl', '+56912345678', 'Calle Los Aromos 456, Temuco', '76.123.456-7', '30 días', 'active', 'a0000000-0000-0000-0000-000000000001', NOW(), NOW()),
    ('c0000000-0000-0000-0000-000000000002', 'RepuestosBike Ltda', 'Ana Fernández', 'pedidos@repuestosbike.cl', '+56987654321', 'Av. industrial 789, Valparaíso', '76.987.654-3', '15 días', 'active', 'a0000000-0000-0000-0000-000000000001', NOW(), NOW()),
    ('c0000000-0000-0000-0000-000000000003', 'Importadora MotoParts', 'Luis Soto', 'compras@motoparts.cl', '+56911223344', 'Parque Industrial, Quilicura', '76.555.123-4', 'Contado', 'active', 'a0000000-0000-0000-0000-000000000001', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- Productos (precios incluyen IVA 19%)
INSERT INTO products (id, name, description, category, brand, model, sku, price, cost, stock, min_stock, location, supplier_id, status, workshop_id, barcode, created_at, updated_at)
VALUES
    -- Filtros
    ('d0000000-0000-0000-0000-000000000001', 'Filtro de aceite universal', 'Filtro de aceite para motos 4T', 'Filtros', 'NGK', 'OF92', 'FIL-001', 9500, 5200, 45, 10, 'Estante A1', 'c0000000-0000-0000-0000-000000000001', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560001', NOW(), NOW()),
    ('d0000000-0000-0000-0000-000000000002', 'Filtro de aire Honda CG', 'Filtro de aire original Honda', 'Filtros', 'Honda', 'CG150', 'FIL-002', 15000, 8500, 30, 8, 'Estante A1', 'c0000000-0000-0000-0000-000000000001', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560002', NOW(), NOW()),
    -- Frenos
    ('d0000000-0000-0000-0000-000000000003', 'Pastillas de freno delanteras', 'Pastillas cerámica para disco delantero', 'Frenos', 'Brembo', 'SC168', 'FRE-001', 18000, 10500, 25, 5, 'Estante B1', 'c0000000-0000-0000-0000-000000000002', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560003', NOW(), NOW()),
    ('d0000000-0000-0000-0000-000000000004', 'Disco de freno delantero 260mm', 'Disco flotante 260mm', 'Frenos', 'Galfer', 'FD118', 'FRE-002', 35000, 21000, 12, 3, 'Estante B1', 'c0000000-0000-0000-0000-000000000002', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560004', NOW(), NOW()),
    -- Encendido
    ('d0000000-0000-0000-0000-000000000005', 'Bujía iridium NGK', 'Bujía iridium CPR8EIX', 'Encendido', 'NGK', 'CPR8EIX', 'ENC-001', 7500, 4200, 50, 15, 'Estante C1', 'c0000000-0000-0000-0000-000000000001', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560005', NOW(), NOW()),
    ('d0000000-0000-0000-0000-000000000006', 'Bobina de encendido universal', 'Bobina CDI universal 12V', 'Encendido', 'Takasago', 'TS-C123', 'ENC-002', 22000, 13500, 18, 5, 'Estante C1', 'c0000000-0000-0000-0000-000000000001', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560006', NOW(), NOW()),
    -- Transmisión
    ('d0000000-0000-0000-0000-000000000007', 'Cadena 428H 118 eslabones', 'Cadena reforzada 428H', 'Transmisión', 'RK', '428H-118', 'TRA-001', 28000, 16000, 20, 6, 'Estante D1', 'c0000000-0000-0000-0000-000000000002', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560007', NOW(), NOW()),
    ('d0000000-0000-0000-0000-000000000008', 'Piñón delantero 17T', 'Piñón acero cromado 17 dientes', 'Transmisión', 'Sunstar', 'CS-17', 'TRA-002', 12000, 7000, 35, 10, 'Estante D1', 'c0000000-0000-0000-0000-000000000002', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560008', NOW(), NOW()),
    -- Suspension
    ('d0000000-0000-0000-0000-000000000009', 'Aceite dehorquillas 10W', 'Aceite mineral para horquillas 1L', 'Suspensión', 'Motul', 'Fork 10W', 'SUS-001', 8500, 5000, 40, 12, 'Estante E1', 'c0000000-0000-0000-0000-000000000001', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560009', NOW(), NOW()),
    -- Mantención
    ('d0000000-0000-0000-0000-000000000010', 'Aceite motor 4T 10W40 1L', 'Aceite semisintético para motor', 'Mantención', 'Motul', '7100 10W40', 'MAN-001', 9000, 5500, 60, 20, 'Estante F1', 'c0000000-0000-0000-0000-000000000001', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560010', NOW(), NOW()),
    ('d0000000-0000-0000-0000-000000000011', 'Spray limpiador de frenos 500ml', 'Limpiador sin residuos', 'Mantención', 'Würth', 'BRK CLEAN', 'MAN-002', 6500, 3800, 30, 8, 'Estante F1', 'c0000000-0000-0000-0000-000000000003', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560011', NOW(), NOW()),
    -- Neumáticos
    ('d0000000-0000-0000-0000-000000000012', 'Neumático delantero 90/90-17', 'Neumático tubeless radial', 'Neumáticos', 'IRC', 'MB-66', 'NEU-001', 42000, 28000, 8, 2, 'Piso', 'c0000000-0000-0000-0000-000000000002', 'active', 'a0000000-0000-0000-0000-000000000001', '7801234560012', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- VENTAS — 25 ventas de marzo a agosto 2026
-- ============================================================
-- Función auxiliar: calcular IVA de un precio que ya incluye IVA
-- base = precio / 1.19, iva = precio - base

-- === MARZO 2026 ===
INSERT INTO sales (id, customer_name, customer_phone, total, payment_method, status, workshop_id, subtotal, discount_amount, taxable_amount, tax_amount, created_at)
VALUES
    ('e0000000-0000-0000-0000-000000000001', 'Pedro Gutiérrez', '+56911111111', 36000, 'cash', 'completed', 'a0000000-0000-0000-0000-000000000001', 30252.10, 0, 30252.10, 5747.90, '2026-03-05 10:30:00-03'),
    ('e0000000-0000-0000-0000-000000000002', 'Ana Vargas', '+56922222222', 7500, 'card', 'completed', 'a0000000-0000-0000-0000-000000000001', 6302.52, 0, 6302.52, 1197.48, '2026-03-12 14:15:00-03'),
    ('e0000000-0000-0000-0000-000000000003', 'Marcos Silva', '+56933333333', 53000, 'transfer', 'completed', 'a0000000-0000-0000-0000-000000000001', 44537.82, 0, 44537.82, 8462.18, '2026-03-20 09:00:00-03'),
    ('e0000000-0000-0000-0000-000000000004', 'Catalina Rojas', '+56944444444', 22000, 'cash', 'completed', 'a0000000-0000-0000-0000-000000000001', 18487.39, 0, 18487.39, 3512.61, '2026-03-28 16:45:00-03');

-- === ABRIL 2026 ===
INSERT INTO sales (id, customer_name, customer_phone, total, payment_method, status, workshop_id, subtotal, discount_amount, taxable_amount, tax_amount, created_at)
VALUES
    ('e0000000-0000-0000-0000-000000000005', 'Francisco Morales', '+56955555555', 18000, 'card', 'completed', 'a0000000-0000-0000-0000-000000000001', 15126.05, 0, 15126.05, 2873.95, '2026-04-03 11:00:00-04'),
    ('e0000000-0000-0000-0000-000000000006', 'Isabel Fernández', '+56966666666', 9500, 'cash', 'completed', 'a0000000-0000-0000-0000-000000000001', 7983.19, 0, 7983.19, 1516.81, '2026-04-10 15:30:00-04'),
    ('e0000000-0000-0000-0000-000000000007', 'Rodrigo Castro', '+56977777777', 77000, 'transfer', 'completed', 'a0000000-0000-0000-0000-000000000001', 64705.88, 0, 64705.88, 12294.12, '2026-04-18 10:15:00-04'),
    ('e0000000-0000-0000-0000-000000000008', 'Valentina Díaz', '+56988888888', 15000, 'card', 'completed', 'a0000000-0000-0000-0000-000000000001', 12605.04, 0, 12605.04, 2394.96, '2026-04-25 13:45:00-04'),
    ('e0000000-0000-0000-0000-000000000009', 'Andrés Muñoz', '+56999999999', 45000, 'cash', 'completed', 'a0000000-0000-0000-0000-000000000001', 37815.13, 0, 37815.13, 7184.87, '2026-04-30 09:30:00-04');

-- === MAYO 2026 ===
INSERT INTO sales (id, customer_name, customer_phone, total, payment_method, status, workshop_id, subtotal, discount_amount, taxable_amount, tax_amount, created_at)
VALUES
    ('e0000000-0000-0000-0000-000000000010', 'Patricia Herrera', '+56910101010', 28000, 'transfer', 'completed', 'a0000000-0000-0000-0000-000000000001', 23529.41, 0, 23529.41, 4470.59, '2026-05-02 10:00:00-05'),
    ('e0000000-0000-0000-0000-000000000011', 'Sergio Reyes', '+56920202020', 9500, 'cash', 'completed', 'a0000000-0000-0000-0000-000000000001', 7983.19, 0, 7983.19, 1516.81, '2026-05-08 14:30:00-05'),
    ('e0000000-0000-0000-0000-000000000012', 'Claudia Vega', '+56930303030', 64000, 'card', 'completed', 'a0000000-0000-0000-0000-000000000001', 53781.51, 0, 53781.51, 10218.49, '2026-05-15 11:45:00-05'),
    ('e0000000-0000-0000-0000-000000000013', 'Diego Torres', '+56940404040', 35000, 'cash', 'completed', 'a0000000-0000-0000-0000-000000000001', 29411.76, 0, 29411.76, 5588.24, '2026-05-22 16:00:00-05'),
    ('e0000000-0000-0000-0000-000000000014', 'Gabriela Flores', '+56950505050', 12000, 'transfer', 'completed', 'a0000000-0000-0000-0000-000000000001', 10084.03, 0, 10084.03, 1915.97, '2026-05-29 08:30:00-05');

-- === JUNIO 2026 ===
INSERT INTO sales (id, customer_name, customer_phone, total, payment_method, status, workshop_id, subtotal, discount_amount, taxable_amount, tax_amount, created_at)
VALUES
    ('e0000000-0000-0000-0000-000000000015', 'Manuel Soto', '+56960606060', 50500, 'card', 'completed', 'a0000000-0000-0000-0000-000000000001', 42436.97, 0, 42436.97, 8063.03, '2026-06-05 12:15:00-06'),
    ('e0000000-0000-0000-0000-000000000016', 'Lucía Contreras', '+56970707070', 8500, 'cash', 'completed', 'a0000000-0000-0000-0000-000000000001', 7142.86, 0, 7142.86, 1357.14, '2026-06-12 10:45:00-06'),
    ('e0000000-0000-0000-0000-000000000017', 'Joaquín Pizarro', '+56980808080', 42000, 'transfer', 'completed', 'a0000000-0000-0000-0000-000000000001', 35294.12, 0, 35294.12, 6705.88, '2026-06-19 14:00:00-06'),
    ('e0000000-0000-0000-0000-000000000018', 'Fernanda Lagos', '+56990909090', 21500, 'card', 'completed', 'a0000000-0000-0000-0000-000000000001', 18067.23, 0, 18067.23, 3432.77, '2026-06-26 09:15:00-06');

-- === JULIO 2026 ===
INSERT INTO sales (id, customer_name, customer_phone, total, payment_method, status, workshop_id, subtotal, discount_amount, taxable_amount, tax_amount, created_at)
VALUES
    ('e0000000-0000-0000-0000-000000000019', 'Nicolás Espinoza', '+56911112112', 11500, 'cash', 'completed', 'a0000000-0000-0000-0000-000000000001', 9663.87, 0, 9663.87, 1836.13, '2026-07-03 11:30:00-07'),
    ('e0000000-0000-0000-0000-000000000020', 'Camila Ortiz', '+56922213221', 89000, 'transfer', 'completed', 'a0000000-0000-0000-0000-000000000001', 74789.92, 0, 74789.92, 14210.08, '2026-07-10 15:15:00-07'),
    ('e0000000-0000-0000-0000-000000000021', 'Tomás Riquelme', '+56933314331', 6500, 'card', 'completed', 'a0000000-0000-0000-0000-000000000001', 5462.18, 0, 5462.18, 1037.82, '2026-07-17 10:00:00-07'),
    ('e0000000-0000-0000-0000-000000000022', 'Javiera Mena', '+56944415441', 31000, 'cash', 'completed', 'a0000000-0000-0000-0000-000000000001', 26050.42, 0, 26050.42, 4949.58, '2026-07-24 13:45:00-07');

-- === AGOSTO 2026 ===
INSERT INTO sales (id, customer_name, customer_phone, total, payment_method, status, workshop_id, subtotal, discount_amount, taxable_amount, tax_amount, created_at)
VALUES
    ('e0000000-0000-0000-0000-000000000023', 'Sebastián Villalobos', '+56955516551', 16000, 'transfer', 'completed', 'a0000000-0000-0000-0000-000000000001', 13445.38, 0, 13445.38, 2554.62, '2026-08-01 09:30:00-08'),
    ('e0000000-0000-0000-0000-000000000024', 'Paula Sepúlveda', '+56966617661', 70000, 'card', 'completed', 'a0000000-0000-0000-0000-000000000001', 58823.53, 0, 58823.53, 11176.47, '2026-08-10 14:00:00-08'),
    ('e0000000-0000-0000-0000-000000000025', 'Cristóbal Navarro', '+56977718771', 19500, 'cash', 'completed', 'a0000000-0000-0000-0000-000000000001', 16386.55, 0, 16386.55, 3113.45, '2026-08-18 11:15:00-08');

-- ============================================================
-- SALE ITEMS — Artículos de cada venta
-- ============================================================

-- Venta 1: Filtro + Bujía
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000001', 'e0000000-0000-0000-0000-000000000001', 'd0000000-0000-0000-0000-000000000001', 'Filtro de aceite universal', 2, 9500, 19000),
    ('f0000000-0000-0000-0000-000000000002', 'e0000000-0000-0000-0000-000000000001', 'd0000000-0000-0000-0000-000000000005', 'Bujía iridium NGK', 1, 7500, 7500),
    ('f0000000-0000-0000-0000-000000000003', 'e0000000-0000-0000-0000-000000000001', 'd0000000-0000-0000-0000-000000000010', 'Aceite motor 4T 10W40 1L', 1, 9000, 9000);

-- Venta 2: Bujía
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000004', 'e0000000-0000-0000-0000-000000000002', 'd0000000-0000-0000-0000-000000000005', 'Bujía iridium NGK', 1, 7500, 7500);

-- Venta 3: Pastillas + Disco + Spray
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000005', 'e0000000-0000-0000-0000-000000000003', 'd0000000-0000-0000-0000-000000000003', 'Pastillas de freno delanteras', 1, 18000, 18000),
    ('f0000000-0000-0000-0000-000000000006', 'e0000000-0000-0000-0000-000000000003', 'd0000000-0000-0000-0000-000000000004', 'Disco de freno delantero 260mm', 1, 35000, 35000),
    ('f0000000-0000-0000-0000-000000000007', 'e0000000-0000-0000-0000-000000000003', 'd0000000-0000-0000-0000-000000000011', 'Spray limpiador de frenos 500ml', 1, 6500, 6500);

-- Venta 4: Bobina
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000008', 'e0000000-0000-0000-0000-000000000004', 'd0000000-0000-0000-0000-000000000006', 'Bobina de encendido universal', 1, 22000, 22000);

-- Venta 5: Pastillas
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000009', 'e0000000-0000-0000-0000-000000000005', 'd0000000-0000-0000-0000-000000000003', 'Pastillas de freno delanteras', 1, 18000, 18000);

-- Venta 6: Filtro
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000010', 'e0000000-0000-0000-0000-000000000006', 'd0000000-0000-0000-0000-000000000001', 'Filtro de aceite universal', 1, 9500, 9500);

-- Venta 7: Cadena + Piñón + Aceite
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000011', 'e0000000-0000-0000-0000-000000000007', 'd0000000-0000-0000-0000-000000000007', 'Cadena 428H 118 eslabones', 2, 28000, 56000),
    ('f0000000-0000-0000-0000-000000000012', 'e0000000-0000-0000-0000-000000000007', 'd0000000-0000-0000-0000-000000000008', 'Piñón delantero 17T', 1, 12000, 12000),
    ('f0000000-0000-0000-0000-000000000013', 'e0000000-0000-0000-0000-000000000007', 'd0000000-0000-0000-0000-000000000010', 'Aceite motor 4T 10W40 1L', 1, 9000, 9000);

-- Venta 8: Filtro de aire
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000014', 'e0000000-0000-0000-0000-000000000008', 'd0000000-0000-0000-0000-000000000002', 'Filtro de aire Honda CG', 1, 15000, 15000);

-- Venta 9: Aceite horquillas + Aceite motor
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000015', 'e0000000-0000-0000-0000-000000000009', 'd0000000-0000-0000-0000-000000000009', 'Aceite dehorquillas 10W', 1, 8500, 8500),
    ('f0000000-0000-0000-0000-000000000016', 'e0000000-0000-0000-0000-000000000009', 'd0000000-0000-0000-0000-000000000010', 'Aceite motor 4T 10W40 1L', 4, 9000, 36000);

-- Venta 10: Neumático
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000017', 'e0000000-0000-0000-0000-000000000010', 'd0000000-0000-0000-0000-000000000012', 'Neumático delantero 90/90-17', 1, 42000, 42000);

-- Venta 11: Filtro
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000018', 'e0000000-0000-0000-0000-000000000011', 'd0000000-0000-0000-0000-000000000001', 'Filtro de aceite universal', 1, 9500, 9500);

-- Venta 12: Pastillas + Aceite
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000019', 'e0000000-0000-0000-0000-000000000012', 'd0000000-0000-0000-0000-000000000003', 'Pastillas de freno delanteras', 2, 18000, 36000),
    ('f0000000-0000-0000-0000-000000000020', 'e0000000-0000-0000-0000-000000000012', 'd0000000-0000-0000-0000-000000000010', 'Aceite motor 4T 10W40 1L', 3, 9000, 27000);

-- Venta 13: Cadena
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000021', 'e0000000-0000-0000-0000-000000000013', 'd0000000-0000-0000-0000-000000000007', 'Cadena 428H 118 eslabones', 1, 28000, 28000),
    ('f0000000-0000-0000-0000-000000000022', 'e0000000-0000-0000-0000-000000000013', 'd0000000-0000-0000-0000-000000000008', 'Piñón delantero 17T', 1, 12000, 12000);

-- Venta 14: Spray + Aceite
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000023', 'e0000000-0000-0000-0000-000000000014', 'd0000000-0000-0000-0000-000000000011', 'Spray limpiador de frenos 500ml', 1, 6500, 6500),
    ('f0000000-0000-0000-0000-000000000024', 'e0000000-0000-0000-0000-000000000014', 'd0000000-0000-0000-0000-000000000010', 'Aceite motor 4T 10W40 1L', 1, 9000, 9000);

-- Venta 15: Disco + Bujía
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000025', 'e0000000-0000-0000-0000-000000000015', 'd0000000-0000-0000-0000-000000000004', 'Disco de freno delantero 260mm', 1, 35000, 35000),
    ('f0000000-0000-0000-0000-000000000026', 'e0000000-0000-0000-0000-000000000015', 'd0000000-0000-0000-0000-000000000005', 'Bujía iridium NGK', 2, 7500, 15000);

-- Venta 16: Filtro
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000027', 'e0000000-0000-0000-0000-000000000016', 'd0000000-0000-0000-0000-000000000001', 'Filtro de aceite universal', 1, 9500, 9500);

-- Venta 17: Cadena + Aceite
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000028', 'e0000000-0000-0000-0000-000000000017', 'd0000000-0000-0000-0000-000000000007', 'Cadena 428H 118 eslabones', 1, 28000, 28000),
    ('f0000000-0000-0000-0000-000000000029', 'e0000000-0000-0000-0000-000000000017', 'd0000000-0000-0000-0000-000000000010', 'Aceite motor 4T 10W40 1L', 2, 9000, 18000);

-- Venta 18: Neumático
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000030', 'e0000000-0000-0000-0000-000000000018', 'd0000000-0000-0000-0000-000000000012', 'Neumático delantero 90/90-17', 1, 42000, 42000);

-- Venta 19: Pastillas
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000031', 'e0000000-0000-0000-0000-000000000019', 'd0000000-0000-0000-0000-000000000003', 'Pastillas de freno delanteras', 1, 18000, 18000);

-- Venta 20: Filtro + Bujía + Aceite
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000032', 'e0000000-0000-0000-0000-000000000020', 'd0000000-0000-0000-0000-000000000001', 'Filtro de aceite universal', 2, 9500, 19000),
    ('f0000000-0000-0000-0000-000000000033', 'e0000000-0000-0000-0000-000000000020', 'd0000000-0000-0000-0000-000000000005', 'Bujía iridium NGK', 4, 7500, 30000),
    ('f0000000-0000-0000-0000-000000000034', 'e0000000-0000-0000-0000-000000000020', 'd0000000-0000-0000-0000-000000000010', 'Aceite motor 4T 10W40 1L', 4, 9000, 36000);

-- Venta 21: Filtro de aire
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000035', 'e0000000-0000-0000-0000-000000000021', 'd0000000-0000-0000-0000-000000000002', 'Filtro de aire Honda CG', 1, 15000, 15000);

-- Venta 22: Cadena + Piñón
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000036', 'e0000000-0000-0000-0000-000000000022', 'd0000000-0000-0000-0000-000000000007', 'Cadena 428H 118 eslabones', 1, 28000, 28000),
    ('f0000000-0000-0000-0000-000000000037', 'e0000000-0000-0000-0000-000000000022', 'd0000000-0000-0000-0000-000000000008', 'Piñón delantero 17T', 1, 12000, 12000);

-- Venta 23: Aceite motor
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000038', 'e0000000-0000-0000-0000-000000000023', 'd0000000-0000-0000-0000-000000000010', 'Aceite motor 4T 10W40 1L', 2, 9000, 18000);

-- Venta 24: Disco + Pastillas + Spray
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000039', 'e0000000-0000-0000-0000-000000000024', 'd0000000-0000-0000-0000-000000000004', 'Disco de freno delantero 260mm', 1, 35000, 35000),
    ('f0000000-0000-0000-0000-000000000040', 'e0000000-0000-0000-0000-000000000024', 'd0000000-0000-0000-0000-000000000003', 'Pastillas de freno delanteras', 1, 18000, 18000),
    ('f0000000-0000-0000-0000-000000000041', 'e0000000-0000-0000-0000-000000000024', 'd0000000-0000-0000-0000-000000000011', 'Spray limpiador de frenos 500ml', 2, 6500, 13000);

-- Venta 25: Bobina + Filtro
INSERT INTO sale_items (id, sale_id, product_id, product_name, quantity, unit_price, total) VALUES
    ('f0000000-0000-0000-0000-000000000042', 'e0000000-0000-0000-0000-000000000025', 'd0000000-0000-0000-0000-000000000006', 'Bobina de encendido universal', 1, 22000, 22000),
    ('f0000000-0000-0000-0000-000000000043', 'e0000000-0000-0000-0000-000000000025', 'd0000000-0000-0000-0000-000000000001', 'Filtro de aceite universal', 1, 9500, 9500);

-- ============================================================
-- REPARACIONES — 12 reparaciones con distintos estados
-- ============================================================
INSERT INTO repairs (id, customer_name, customer_email, customer_phone, motorcycle, license_plate, description, diagnosis, technician_id, estimated_delivery, priority, status, estimated_cost, final_cost, labor_cost, workshop_id, created_at, updated_at)
VALUES
    -- Completadas
    ('aa000000-0000-0000-0000-000000000001', 'Pedro Gutiérrez', 'pedro@gmail.com', '+56911111111', 'Honda CB190R 2023', 'AB-12-CD', 'Cambio de aceite y filtros', 'Aceite muy oscuro, filtro obstruido', 'b0000000-0000-0000-0000-000000000003', '2026-03-10', 'low', 'completed', 35000, 32000, 12000, 'a0000000-0000-0000-0000-000000000001', '2026-03-05 09:00:00-03', '2026-03-10 16:00:00-03'),
    ('aa000000-0000-0000-0000-000000000002', 'Ana Vargas', 'ana.vargas@hotmail.com', '+56922222222', 'Yamaha FZ25 2022', 'EF-34-GH', 'Reparación de frenos delanteros', 'Pastillas gastadas, disco con surcos', 'b0000000-0000-0000-0000-000000000003', '2026-04-01', 'high', 'completed', 85000, 78500, 25000, 'a0000000-0000-0000-0000-000000000001', '2026-03-25 10:30:00-03', '2026-03-31 18:00:00-03'),
    ('aa000000-0000-0000-0000-000000000003', 'Francisco Morales', 'f.morales@outlook.cl', '+56955555555', 'Suzuki Gixxer 150 2024', 'IJ-56-KL', 'Service general 10.000 km', 'Cambio de aceite, filtros, cadena', 'b0000000-0000-0000-0000-000000000003', '2026-04-15', 'medium', 'completed', 55000, 52000, 18000, 'a0000000-0000-0000-0000-000000000001', '2026-04-08 08:00:00-04', '2026-04-14 14:30:00-04'),
    -- En progreso
    ('aa000000-0000-0000-0000-000000000004', 'Rodrigo Castro', 'r.castro@gmail.com', '+56977777777', 'Kawasaki Ninja 300 2023', 'MN-78-OP', 'Cambio de cadena y piñones', 'Cadena estirada, piñones desgastados', 'b0000000-0000-0000-0000-000000000003', '2026-09-20', 'medium', 'in_progress', 68000, NULL, 15000, 'a0000000-0000-0000-0000-000000000001', '2026-09-10 09:00:00-04', '2026-09-14 11:00:00-04'),
    ('aa000000-0000-0000-0000-000000000005', 'Catalina Rojas', 'c.rojas@yahoo.cl', '+56944444444', 'Honda Wave 110 2022', 'QR-90-ST', 'Reparación de motor - ruido anormal', 'Balancines desajustados, válvulas flojas', 'b0000000-0000-0000-0000-000000000003', '2026-09-25', 'high', 'in_progress', 120000, NULL, 45000, 'a0000000-0000-0000-0000-000000000001', '2026-09-12 14:00:00-04', '2026-09-15 10:30:00-04'),
    ('aa000000-0000-0000-0000-000000000006', 'Diego Torres', 'd.torres@gmail.com', '+56940404040', 'Yamaha Crypton 125 2021', 'UV-12-WX', 'Suspensión delantera - fuga de aceite', 'Retén dañado, horquilla con juego', 'b0000000-0000-0000-0000-000000000003', '2026-09-22', 'medium', 'in_progress', 45000, NULL, 12000, 'a0000000-0000-0000-0000-000000000001', '2026-09-14 11:00:00-04', '2026-09-15 16:00:00-04'),
    -- Pendientes
    ('aa000000-0000-0000-0000-000000000007', 'Patricia Herrera', 'p.herrera@gmail.com', '+56910101010', 'Suzuki Ax100 2020', 'YZ-34-AB', 'Cambio de neumáticos', 'Neumáticos lisos, sin tracción', 'b0000000-0000-0000-0000-000000000003', '2026-09-28', 'low', 'pending', 95000, NULL, NULL, 'a0000000-0000-0000-0000-000000000001', '2026-09-15 08:30:00-04', '2026-09-15 08:30:00-04'),
    ('aa000000-0000-0000-0000-000000000008', 'Manuel Soto', 'm.soto@outlook.cl', '+56960606060', 'Honda CBF 125 2023', 'CD-56-EF', 'Instalación de accesorios', 'Barra de protección + portaequipajes', 'b0000000-0000-0000-0000-000000000003', '2026-09-30', 'low', 'pending', 35000, NULL, NULL, 'a0000000-0000-0000-0000-000000000001', '2026-09-15 10:00:00-04', '2026-09-15 10:00:00-04'),
    -- Canceladas
    ('aa000000-0000-0000-0000-000000000009', 'Gabriela Flores', 'g.flores@gmail.com', '+56950505050', 'Kawasaki Boxer 150 2019', 'GH-78-IJ', 'Pintura completa', 'Cliente canceló por cambio de opinión', NULL, NULL, 'medium', 'cancelled', 180000, NULL, NULL, 'a0000000-0000-0000-0000-000000000001', '2026-08-20 09:00:00-04', '2026-08-25 11:00:00-04'),
    ('aa000000-0000-0000-0000-000000000010', 'Nicolás Espinoza', 'n.espinoza@gmail.com', '+56911112112', 'Yamaha XTZ 250 2022', 'KL-90-MN', 'Cambio de motor completo', 'Presupuesto excedió presupuesto del cliente', 'b0000000-0000-0000-0000-000000000003', '2026-09-01', 'high', 'cancelled', 850000, NULL, NULL, 'a0000000-0000-0000-0000-000000000001', '2026-08-10 14:00:00-04', '2026-08-15 09:30:00-04'),
    -- Completada reciente
    ('aa000000-0000-0000-0000-000000000011', 'Joaquín Pizarro', 'j.pizarro@gmail.com', '+56980808080', 'Honda PCX 160 2024', 'OP-12-QR', 'Service programado 5.000 km', 'Cambio de aceite, filtro, revisión frenos', 'b0000000-0000-0000-0000-000000000003', '2026-09-18', 'low', 'completed', 28000, 25000, 8000, 'a0000000-0000-0000-0000-000000000001', '2026-09-12 08:00:00-04', '2026-09-17 15:00:00-04'),
    -- En progreso reciente
    ('aa000000-0000-0000-0000-000000000012', 'Camila Ortiz', 'c.ortiz@yahoo.cl', '+56922213221', 'Suzuki Burgman 125 2023', 'ST-34-UV', 'Reparación eléctrica - no enciende', 'Batería muerta, alternador con falla', 'b0000000-0000-0000-0000-000000000003', '2026-09-24', 'high', 'in_progress', 65000, NULL, 20000, 'a0000000-0000-0000-0000-000000000001', '2026-09-15 16:00:00-04', '2026-09-16 09:00:00-04');

-- ============================================================
-- REPAIR PARTS — Piezas usadas en reparaciones completadas/en progreso
-- ============================================================
INSERT INTO repair_parts (repair_id, name, quantity, unit_cost, total_cost) VALUES
    -- Rep 1: Service de aceite
    ('aa000000-0000-0000-0000-000000000001', 'Filtro de aceite universal', 1, 5200, 5200),
    ('aa000000-0000-0000-0000-000000000001', 'Aceite motor 4T 10W40 1L', 1, 5500, 5500),
    -- Rep 2: Frenos
    ('aa000000-0000-0000-0000-000000000002', 'Pastillas de freno delanteras', 1, 10500, 10500),
    ('aa000000-0000-0000-0000-000000000002', 'Disco de freno delantero 260mm', 1, 21000, 21000),
    ('aa000000-0000-0000-0000-000000000002', 'Spray limpiador de frenos 500ml', 1, 3800, 3800),
    -- Rep 3: Service general
    ('aa000000-0000-0000-0000-000000000003', 'Aceite motor 4T 10W40 1L', 1, 5500, 5500),
    ('aa000000-0000-0000-0000-000000000003', 'Filtro de aceite universal', 1, 5200, 5200),
    ('aa000000-0000-0000-0000-000000000003', 'Filtro de aire Honda CG', 1, 8500, 8500),
    ('aa000000-0000-0000-0000-000000000003', 'Cadena 428H 118 eslabones', 1, 16000, 16000),
    -- Rep 4: Cadena (en progreso)
    ('aa000000-0000-0000-0000-000000000004', 'Cadena 428H 118 eslabones', 1, 16000, 16000),
    ('aa000000-0000-0000-0000-000000000004', 'Piñón delantero 17T', 1, 7000, 7000),
    -- Rep 11: Service programado
    ('aa000000-0000-0000-0000-000000000011', 'Filtro de aceite universal', 1, 5200, 5200),
    ('aa000000-0000-0000-0000-000000000011', 'Aceite motor 4T 10W40 1L', 1, 5500, 5500);

-- ============================================================
-- REPAIR UPDATES — Historial de cambios de estado
-- ============================================================
INSERT INTO repair_updates (repair_id, status, description, created_by, created_at) VALUES
    -- Rep 1
    ('aa000000-0000-0000-0000-000000000001', 'pending', 'Recepción del vehículo', 'b0000000-0000-0000-0000-000000000001', '2026-03-05 09:00:00-03'),
    ('aa000000-0000-0000-0000-000000000001', 'in_progress', 'Iniciando servicio de aceite', 'b0000000-0000-0000-0000-000000000003', '2026-03-05 10:00:00-03'),
    ('aa000000-0000-0000-0000-000000000001', 'completed', 'Servicio completado, cliente notificado', 'b0000000-0000-0000-0000-000000000003', '2026-03-10 16:00:00-03'),
    -- Rep 2
    ('aa000000-0000-0000-0000-000000000002', 'pending', 'Recepción, diagnóstico de frenos', 'b0000000-0000-0000-0000-000000000001', '2026-03-25 10:30:00-03'),
    ('aa000000-0000-0000-0000-000000000002', 'in_progress', 'Cambiando pastillas y disco', 'b0000000-0000-0000-0000-000000000003', '2026-03-26 09:00:00-03'),
    ('aa000000-0000-0000-0000-000000000002', 'completed', 'Frenos reparados, prueba exitosa', 'b0000000-0000-0000-0000-000000000003', '2026-03-31 18:00:00-03'),
    -- Rep 3
    ('aa000000-0000-0000-0000-000000000003', 'pending', 'Service programado recibido', 'b0000000-0000-0000-0000-000000000001', '2026-04-08 08:00:00-04'),
    ('aa000000-0000-0000-0000-000000000003', 'in_progress', 'Cambiando aceite, filtros y cadena', 'b0000000-0000-0000-0000-000000000003', '2026-04-08 09:30:00-04'),
    ('aa000000-0000-0000-0000-000000000003', 'completed', 'Service completado', 'b0000000-0000-0000-0000-000000000003', '2026-04-14 14:30:00-04'),
    -- Rep 4 (en progreso)
    ('aa000000-0000-0000-0000-000000000004', 'pending', 'Recepción, diagnóstico de transmisión', 'b0000000-0000-0000-0000-000000000001', '2026-09-10 09:00:00-04'),
    ('aa000000-0000-0000-0000-000000000004', 'in_progress', 'Cambiando cadena y piñones', 'b0000000-0000-0000-0000-000000000003', '2026-09-14 11:00:00-04'),
    -- Rep 5 (en progreso)
    ('aa000000-0000-0000-0000-000000000005', 'pending', 'Recepción, ruido anormal en motor', 'b0000000-0000-0000-0000-000000000001', '2026-09-12 14:00:00-04'),
    ('aa000000-0000-0000-0000-000000000005', 'in_progress', 'Diagnóstico: válvulas y balancines', 'b0000000-0000-0000-0000-000000000003', '2026-09-15 10:30:00-04'),
    -- Rep 9 (cancelada)
    ('aa000000-0000-0000-0000-000000000009', 'pending', 'Presupuesto enviado: $180.000', 'b0000000-0000-0000-0000-000000000001', '2026-08-20 09:00:00-04'),
    ('aa000000-0000-0000-0000-000000000009', 'cancelled', 'Cliente canceló, presupuesto demasiado alto', 'b0000000-0000-0000-0000-000000000001', '2026-08-25 11:00:00-04'),
    -- Rep 10 (cancelada)
    ('aa000000-0000-0000-0000-000000000010', 'pending', 'Diagnóstico: motor con falla interna', 'b0000000-0000-0000-0000-000000000003', '2026-08-10 14:00:00-04'),
    ('aa000000-0000-0000-0000-000000000010', 'cancelled', 'Presupuesto excedido, cliente no autoriza', 'b0000000-0000-0000-0000-000000000001', '2026-08-15 09:30:00-04'),
    -- Rep 11 (completada reciente)
    ('aa000000-0000-0000-0000-000000000011', 'pending', 'Service programado 5.000 km', 'b0000000-0000-0000-0000-000000000001', '2026-09-12 08:00:00-04'),
    ('aa000000-0000-0000-0000-000000000011', 'in_progress', 'Cambiando aceite y filtro, revisando frenos', 'b0000000-0000-0000-0000-000000000003', '2026-09-12 09:00:00-04'),
    ('aa000000-0000-0000-0000-000000000011', 'completed', 'Service completado, todo OK', 'b0000000-0000-0000-0000-000000000003', '2026-09-17 15:00:00-04'),
    -- Rep 12 (en progreso reciente)
    ('aa000000-0000-0000-0000-000000000012', 'pending', 'No enciende, batería revisada', 'b0000000-0000-0000-0000-000000000001', '2026-09-15 16:00:00-04'),
    ('aa000000-0000-0000-0000-000000000012', 'in_progress', 'Reparando alternador', 'b0000000-0000-0000-0000-000000000003', '2026-09-16 09:00:00-04');

-- ============================================================
-- FIN DEL SEED
-- ============================================================
SELECT 'Seed completado: 1 workshop, 3 usuarios, 3 proveedores, 12 productos, 25 ventas, 12 reparaciones' AS resultado;
