-- Add up migration script here
-- /migrations/TIMESTAMP_create_initial_schema.sql

-- Включаем расширение для UUID, если его еще нет
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Создаем кастомный тип для ролей пользователей
CREATE TYPE user_role AS ENUM ('User', 'Moderator', 'Architect', 'Admin', 'Creator');

-- Таблица пользователей
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    public_key TEXT,
    encrypted_private_key TEXT,
    role user_role NOT NULL DEFAULT 'User',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Таблица локаций
CREATE TABLE locations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    image_url VARCHAR(255),
    security_level INT NOT NULL DEFAULT 0,
    creator_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Таблица для хранения состояния игрока
CREATE TABLE players (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    current_location_id UUID REFERENCES locations(id) ON DELETE SET NULL,
    access_level INT NOT NULL DEFAULT 0,
    inventory JSONB
);

-- Функция-триггер для автоматического обновления поля updated_at
CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Применяем триггер к таблицам
CREATE TRIGGER set_timestamp
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp();

CREATE TRIGGER set_timestamp
BEFORE UPDATE ON locations
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp();

-- Вставляем одну стартовую локацию, чтобы миру было с чего начаться
INSERT INTO locations (id, name, description, security_level)
VALUES ('a1b2c3d4-e5f6-7890-1234-567890abcdef', 'УРОВЕНЬ 0 / ШЛЮЗ 001', 'Холодный свет аварийных ламп отражается от влажного полиметалла. Воздух пахнет озоном и старой пылью.', 0);