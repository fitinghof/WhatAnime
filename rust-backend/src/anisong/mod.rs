use crate::{Error, Result};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::default;

use crate::spotify::responses::SimplifiedArtist;

pub struct AnisongClient {
    client: Client,
}

impl AnisongClient {
    const ANISONG_DB_URL: &str = "https://anisongdb.com/api";
    const ANISONG_DB_SEARCH_REQUEST: &str = "https://anisongdb.com/api/search_request";

    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_animes_by_song_title(&self, title: String) -> Result<Vec<Anime>> {
        let search = SearchRequest {
            song_name_search_filter: Some(SearchFilter {
                search: title,
                ..Default::default()
            }),
            ..Default::default()
        };
        let response = self
            .client
            .post(Self::ANISONG_DB_SEARCH_REQUEST)
            .json(&search)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Error::BadRequest {
                url: Self::ANISONG_DB_SEARCH_REQUEST.to_string(),
                status_code: response.status(),
            })
        }
    }

    pub async fn get_animes_by_artist_name(&self, artist: String) -> Result<Vec<Anime>> {
        let search = SearchRequest {
            artist_search_filter: Some(SearchFilter {
                search: artist,
                ..Default::default()
            }),
            ..Default::default()
        };
        let response = self
            .client
            .post(Self::ANISONG_DB_SEARCH_REQUEST)
            .json(&search)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Error::BadRequest {
                url: Self::ANISONG_DB_SEARCH_REQUEST.to_string(),
                status_code: response.status(),
            })
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SearchFilter {
    search: String,
    partial_match: bool,
    group_granularity: Option<i32>,
    max_other_artist: Option<i32>,
    arrangement: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SearchRequest {
    anime_search_filter: Option<SearchFilter>,
    song_name_search_filter: Option<SearchFilter>,
    artist_search_filter: Option<SearchFilter>,
    composer_search_filter: Option<SearchFilter>,
    and_logic: Option<bool>,
    ignore_duplicate: Option<bool>,
    opening_filter: Option<bool>,
    ending_filter: Option<bool>,
    insert_filter: Option<bool>,
    normal_broadcast: Option<bool>,
    dub: Option<bool>,
    rebroadcast: Option<bool>,
    standard: Option<bool>,
    instrumental: Option<bool>,
    chanting: Option<bool>,
    character: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArtistIDSearchRequest {
    pub artist_ids: Vec<i32>,
    pub group_granularity: i32,
    pub max_other_artist: i32,
    pub ignore_duplicate: bool,
    pub opening_filter: bool,
    pub ending_filter: bool,
    pub insert_filter: bool,
    pub normal_broadcast: bool,
    pub dub: bool,
    pub rebroadcast: bool,
    pub standard: bool,
    pub instrumental: bool,
    pub chanting: bool,
    pub character: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComposerIDSearchRequest {
    composer_ids: Vec<i32>,
    arrangement: bool,
    ignore_duplicate: bool,
    opening_filter: bool,
    ending_filter: bool,
    insert_filter: bool,
    normal_broadcast: bool,
    dub: bool,
    rebroadcast: bool,
    standard: bool,
    instrumental: bool,
    chanting: bool,
    character: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnnIdSearchRequest {
    ann_id: i32,
    ignore_duplicate: bool,
    opening_filter: bool,
    ending_filter: bool,
    insert_filter: bool,
    normal_broadcast: bool,
    dub: bool,
    rebroadcast: bool,
    standard: bool,
    instrumental: bool,
    chanting: bool,
    character: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MalIdsSearchRequest {
    mal_ids: Vec<i32>,
    ignore_duplicate: bool,
    opening_filter: bool,
    ending_filter: bool,
    insert_filter: bool,
    normal_broadcast: bool,
    dub: bool,
    rebroadcast: bool,
    standard: bool,
    instrumental: bool,
    chanting: bool,
    character: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Artist {
    pub id: i32,
    pub names: Vec<String>,
    pub line_up_id: i32,
    pub groups: Vec<Artist>,
    pub members: Vec<Artist>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeListLinks {
    pub myanimelist: i32,
    pub anidb: i32,
    pub anilist: i32,
    pub kitsu: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Anime {
    pub ann_id: i32,
    pub ann_song_id: i32,
    pub anime_en_name: String,
    pub anime_jp_name: String,
    pub anime_alt_name: Vec<String>,
    pub anime_vintage: String,
    pub linked_ids: AnimeListLinks,
    pub anime_type: String,
    pub anime_category: String,
    pub song_type: String,
    pub song_name: String,
    pub song_artist: String,
    pub song_composer: String,
    pub song_arranger: String,
    pub song_difficulty: f64,
    pub song_category: String,
    pub song_length: f64,
    pub is_dub: bool,
    pub is_rebroadcast: bool,
    pub hq: String,
    pub mq: String,
    pub audio: String,
    pub artists: Vec<Artist>,
    pub composers: Vec<Artist>,
    pub arrangers: Vec<Artist>,
}
