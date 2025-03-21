-- Add migration script here
CREATE TABLE IF NOT EXISTS new_artists (
    ann_id INTEGER PRIMARY KEY, -- Probably ann_id, some id atleast that is used in anisong db
    names TEXT[] NOT NULL,
    groups_ids INTEGER[], -- Array of artist ann_id
    members INTEGER[] -- Array of artist ann_id
);

CREATE TABLE IF NOT EXISTS artist_links (
    ann_id INTEGER,
    spotify_id VARCHAR(22),
    PRIMARY KEY (ann_id, spotify_id)
);