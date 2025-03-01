-- Add migration script here
ALTER TABLE animes
DROP COLUMN studios,
DROP COLUMN producers,
DROP COLUMN genres,
DROP COLUMN themes,
DROP COLUMN score,

DROP COLUMN image_url_jpg_small,
DROP COLUMN image_url_jpg_normal,
DROP COLUMN image_url_jpg_big,
DROP COLUMN image_url_webp_small,
DROP COLUMN image_url_webp_normal,
DROP COLUMN image_url_webp_big,
DROP COLUMN source,

    -- Trailer
DROP COLUMN youtube_id,
DROP COLUMN url,
DROP COLUMN embed_url,
DROP COLUMN image_url,
DROP COLUMN small_image_url,
DROP COLUMN medium_image_url,
DROP COLUMN large_image_url,
DROP COLUMN maximum_image_url;

ALTER TABLE animes
RENAME year TO release_year;

ALTER TABLE animes
ADD COLUMN mean_score INTEGER,

ADD COLUMN banner_image TEXT,

ADD COLUMN cover_image_color VARCHAR(8),
ADD COLUMN cover_image_medium TEXT,
ADD COLUMN cover_image_large TEXT,
ADD COLUMN cover_image_extra_large TEXT,

ADD COLUMN media_format SMALLINT,
ADD COLUMN genres VARCHAR(32)[],
ADD COLUMN source VARCHAR(32),

ADD COLUMN studio_ids INTEGER[],
ADD COLUMN studio_names TEXT[],
ADD COLUMN studio_urls TEXT[],

ADD COLUMN tag_ids INTEGER[],
ADD COLUMN tag_names VARCHAR(32)[],

ADD COLUMN trailer_id TEXT,
ADD COLUMN trailer_site TEXT,
ADD COLUMN thumbnail TEXT,

ADD COLUMN release_season SMALLINT;
