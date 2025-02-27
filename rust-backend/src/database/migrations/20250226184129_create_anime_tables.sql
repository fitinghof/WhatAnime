-- Add migration script here
CREATE TABLE IF NOT EXISTS animes (
    ann_id INTEGER NOT NULL,

    title_eng TEXT NOT NULL,
    title_jpn TEXT NOT NULL,

    -- ex Season 1 or Movie 2
    index_type SMALLINT NOT NULL,
    index_number INTEGER NOT NULL,

    -- ex TV, Movie, TV Special, might be removed in favor of just using index type
    anime_type SMALLINT NOT NULL,

    -- What medium the anime is adapted from, or original, probably
    source VARCHAR(32) NOT NULL,

    -- How many episodes
    episodes INTEGER,

    -- images
    image_url_jpg_small TEXT NOT NULL,
    image_url_jpg_normal TEXT NOT NULL,
    image_url_jpg_big TEXT NOT NULL,

    image_url_webp_small TEXT NOT NULL,
    image_url_webp_normal TEXT NOT NULL,
    image_url_webp_big TEXT NOT NULL,

    -- Trailer
    youtube_id TEXT,
    url TEXT,
    embed_url TEXT,
    -- Trailer / TrailerImages
    image_url TEXT,
    small_image_url TEXT,
    medium_image_url TEXT,
    large_image_url TEXT,
    maximum_image_url TEXT,

    -- linked_ids
    mal_id INTEGER,
    anilist_id INTEGER,
    anidb_id INTEGER,
    kitsu_id INTEGER,

    year INTEGER,

    -- Studios, mal ids
    studios INTEGER[] NOT NULL,

    -- producers, mal ids
    producers INTEGER[] NOT NULL,

    -- Genres, mal ids
    genres INTEGER[] NOT NULL,

    -- Themes, mal ids
    themes INTEGER[] NOT NULL,

    -- The mal score of the anime, ex 8.32
    score REAL NOT NULL,

    -- song info
    spotify_id CHAR(22) NOT NULL,
    ann_song_id INTEGER PRIMARY KEY,
    song_name TEXT,

    spotify_artist_ids char(22)[] NOT NULL,
    artist_names TEXT[] NOT NULL,
    -- anisong ann_ids
    artists_ann_id INTEGER[] NOT NULL,
    composers_ann_id INTEGER[] NOT NULL,
    arrangers_ann_id INTEGER[] NOT NULL,

    track_index_type SMALLINT NOT NULL,
    track_index_number INTEGER NOT NULL

);

CREATE INDEX idx_anime_spotify_ids ON animes(spotify_id);
CREATE INDEX idx_anime_ann_ids ON animes(ann_id);

CREATE TABLE IF NOT EXISTS genres (
    mal_id INTEGER PRIMARY KEY,
    genre_type TEXT NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS producers (
    mal_id INTEGER PRIMARY KEY,
    producer_type TEXT NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS artists (
    spotify_id CHAR(22) PRIMARY KEY,
    ann_id INTEGER NOT NULL, -- Probably ann_id, some id atleast that is used in anisong db
    names TEXT[] NOT NULL,
    groups_ids INTEGER[], -- Array of artist ann_id
    members INTEGER[] -- Array of artist ann_id
);

CREATE INDEX idx_artists_ann_id ON artists(ann_id);

CREATE TABLE anime_artists (
    anime_id INTEGER REFERENCES animes(ann_song_id),
    artist_id CHAR(22) REFERENCES artists(spotify_id),
    PRIMARY KEY (anime_id, artist_id)
);
