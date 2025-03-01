use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::Anilist::types::AnilistID;

#[derive(FromRow, Serialize, Deserialize)]
pub struct DBAnime {
    pub ann_id: i32,
    pub title_eng: String,
    pub title_jpn: String,
    pub index_type: i16,
    pub index_number: i32,
    pub anime_type: i16,
    pub episodes: Option<i32>,

    // linked_ids
    pub mal_id: Option<i32>,
    pub anilist_id: Option<AnilistID>,
    pub anidb_id: Option<i32>,
    pub kitsu_id: Option<i32>,

    pub release_year: Option<i32>,
    pub release_season: Option<i16>,

    pub mean_score: Option<i32>,

    pub banner_image: Option<String>,

    pub cover_image_color: Option<String>,
    pub cover_image_medium: Option<String>,
    pub cover_image_large: Option<String>,
    pub cover_image_extra_large: Option<String>,

    pub media_format: Option<i16>,
    pub genres: Option<Vec<String>>,
    pub source: Option<String>,
    pub studio_ids: Option<Vec<i32>>,
    pub studio_names: Option<Vec<String>>,
    pub studio_urls: Option<Vec<String>>,
    pub tag_ids: Option<Vec<i32>>,
    pub tag_names: Option<Vec<String>>,
    pub trailer_id: Option<String>,
    pub trailer_site: Option<String>,
    pub thumbnail: Option<String>,

    // Song info
    pub spotify_id: String,
    pub ann_song_id: i32,
    pub song_name: String,
    pub spotify_artist_ids: Vec<String>,
    // pub spotify_title: String, // ?
    pub artist_names: Vec<String>,
    pub artists_ann_id: Vec<i32>,
    pub composers_ann_id: Vec<i32>,
    pub arrangers_ann_id: Vec<i32>,

    pub track_index_type: i16,
    pub track_index_number: i32,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct DBArtist {
    pub spotify_id: String,
    pub ann_id: i32,
    pub names: Vec<String>,
    pub groups_ids: Option<Vec<i32>>,
    pub members: Option<Vec<i32>>,
}
