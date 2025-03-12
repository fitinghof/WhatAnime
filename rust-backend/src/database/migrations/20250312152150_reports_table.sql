-- Add migration script here
CREATE TABLE IF NOT EXISTS reports (
    report_id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    spotify_id VARCHAR(22) NOT NULL,
    ann_song_id INTEGER NOT NULL,
    reason TEXT NOT NULL,
    user_name TEXT,
    user_mail TEXT
);