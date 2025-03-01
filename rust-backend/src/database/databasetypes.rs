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
    pub source: String,
    pub episodes: Option<i32>,
    pub image_url_jpg_small: String,
    pub image_url_jpg_normal: String,
    pub image_url_jpg_big: String,
    pub image_url_webp_small: String,
    pub image_url_webp_normal: String,
    pub image_url_webp_big: String,

    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
    pub image_url: Option<String>,
    pub small_image_url: Option<String>,
    pub medium_image_url: Option<String>,
    pub large_image_url: Option<String>,
    pub maximum_image_url: Option<String>,
    pub mal_id: Option<i32>,
    pub anilist_id: Option<AnilistID>,
    pub anidb_id: Option<i32>,
    pub kitsu_id: Option<i32>,
    pub year: Option<i32>,

    pub studios: Vec<i32>,
    pub producers: Vec<i32>,
    pub genres: Vec<i32>,
    pub themes: Vec<i32>,
    pub score: f32,

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