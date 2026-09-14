CREATE TABLE workshops (
    id UUID PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    address VARCHAR(300) NOT NULL,
    city VARCHAR(120) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE users ADD COLUMN workshop_id UUID;

INSERT INTO workshops (id, name, address, city)
SELECT gen_random_uuid(), 'WorkshopManager', 'Sin dirección', 'Sin ciudad'
WHERE EXISTS (SELECT 1 FROM users);

UPDATE users
SET workshop_id = (SELECT id FROM workshops LIMIT 1)
WHERE workshop_id IS NULL;

ALTER TABLE users
    ALTER COLUMN workshop_id SET NOT NULL,
    ADD CONSTRAINT users_workshop_id_fkey FOREIGN KEY (workshop_id) REFERENCES workshops(id);

CREATE INDEX idx_users_workshop_id ON users(workshop_id);