-- Add migration script here
ALTER TABLE animes DROP COLUMN spotify_id;

-- Should allow adding more entries to the database, even though they might not be binded to spotify ids yet.
ALTER TABLE animes ALTER COLUMN song_group_id DROP NOT NULL;
ALTER TABLE animes ALTER COLUMN song_group_id DROP DEFAULT;

-- Last updated
ALTER TABLE animes ADD COLUMN last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()