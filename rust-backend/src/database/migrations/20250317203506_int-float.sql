-- Add migration script here
ALTER TABLE animes
ALTER COLUMN index_number TYPE FLOAT USING index_number::FLOAT;