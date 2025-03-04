
ALTER TABLE animes
    ADD COLUMN song_group_id INTEGER NOT NULL DEFAULT 0;

CREATE INDEX idx_group_id ON animes(song_group_id);

CREATE TABLE IF NOT EXISTS song_groups (
    group_id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- Identifies a single anisong song 
    song_title TEXT NOT NULL, -- The name of the song in anisong_db
    artist_ids INTEGER[] NOT NULL -- All the artists that contributed to the song according to anisongdb
);

CREATE INDEX idx_song_title ON song_groups(song_title);
CREATE INDEX idx_artist_ids ON song_groups(artist_ids);

CREATE TABLE IF NOT EXISTS song_group_links (
    spotify_id VARCHAR(22) PRIMARY KEY,
    group_id INTEGER REFERENCES song_groups(group_id) NOT NULL -- Identifies a single anisong song 
);
