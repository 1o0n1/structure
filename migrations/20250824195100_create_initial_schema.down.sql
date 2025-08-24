-- Add down migration script here
-- /migrations/TIMESTAMP_create_initial_schema.down.sql

-- Удаляем таблицы в порядке, обратном созданию,
-- чтобы не нарушать внешние ключи (foreign keys).

DROP TABLE IF EXISTS players;
DROP TABLE IF EXISTS locations;
DROP TABLE IF EXISTS users;

-- Удаляем кастомный тип
DROP TYPE IF EXISTS user_role;

-- Удаляем функцию-триггер
DROP FUNCTION IF EXISTS trigger_set_timestamp();