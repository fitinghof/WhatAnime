-- Add migration script here
ALTER TABLE animes
ALTER COLUMN index_number TYPE REAL USING index_number::REAL;