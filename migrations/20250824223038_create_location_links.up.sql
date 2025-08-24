-- Add up migration script here
-- /migrations/TIMESTAMP_create_location_links.up.sql
CREATE TABLE location_links (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_location_id UUID NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    target_location_id UUID NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    link_text VARCHAR(255) NOT NULL,
    required_access_level INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Создадим вторую локацию для теста
-- /migrations/TIMESTAMP_create_location_links.up.sql
-- ... (создание таблицы location_links) ...

-- Создадим вторую локацию для теста с валидным UUID
INSERT INTO locations (id, name, description, security_level)
VALUES ('550e8400-e29b-41d4-a716-446655440000', 'ДАТА-ХАБ 01', 'Серверные стойки уходят ввысь, их индикаторы медленно пульсируют в темноте. В центре зала стоит главный терминал.', 1);

-- Создадим связь от Шлюза к Дата-хабу и обратно, используя валидные UUID
INSERT INTO location_links (source_location_id, target_location_id, link_text, required_access_level) VALUES
('a1b2c3d4-e5f6-7890-1234-567890abcdef', '550e8400-e29b-41d4-a716-446655440000', '> Войти в дата-хаб [УР. БЕЗОПАСНОСТИ: 1]', 0),
('550e8400-e29b-41d4-a716-446655440000', 'a1b2c3d4-e5f6-7890-1234-567890abcdef', '> Вернуться к шлюзу 001', 0);