use std::clone;

use crate::{Error, Result};

use reqwest::Client;
use serde::{Deserialize, Serialize};


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
            anime_search_filter: None,
            song_name_search_filter: None,
            artist_search_filter: Some(SearchFilter {
                search: artist,
                partial_match: false,
                group_granularity: Some(0),
                max_other_artist: Some(99),
                arrangement: Some(true),
            }),
            composer_search_filter: None,
            and_logic: Some(true),
            ignore_duplicate: Some(false),
            opening_filter: Some(true),
            ending_filter: Some(true),
            insert_filter: Some(true),
            normal_broadcast: Some(true),
            dub: Some(true),
            rebroadcast: Some(true),
            standard: Some(true),
            instrumental: Some(true),
            chanting: Some(true),
            character: Some(true),
        };

        let response = self
            .client
            .post(Self::ANISONG_DB_SEARCH_REQUEST)
            .json(&search)
            .send()
            .await
            .unwrap();

        if response.status().is_success() {
            // println!("{}", response.text().await.unwrap());
            // return Err(Error::NotASong);
            Ok(response.json().await.unwrap())
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
    pub line_up_id: Option<i32>,
    pub groups: Option<Vec<Artist>>,
    pub members: Option<Vec<Artist>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimeListLinks {
    pub myanimelist: Option<i32>,
    pub anidb: Option<i32>,
    pub anilist: Option<i32>,
    pub kitsu: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Anime {
    pub annId: Option<i32>,
    pub annSongId: Option<i32>,
    pub animeENName: String,
    pub animeJPName: String,
    pub animeAltName: Option<Vec<String>>,
    pub animeVintage: Option<String>,
    pub linked_ids: AnimeListLinks,
    pub animeType: Option<String>,
    pub animeCategory: String,
    pub songType: String,
    pub songName: String,
    pub songArtist: String,
    pub songComposer: String,
    pub songArranger: String,
    pub songDifficulty: Option<f64>,
    pub songCategory: String,
    pub songLength: Option<f64>,
    pub isDub: bool,
    pub isRebroadcast: bool,
    pub HQ: Option<String>,
    pub MQ: Option<String>,
    pub audio: Option<String>,
    pub artists: Vec<Artist>,
    pub composers: Vec<Artist>,
    pub arrangers: Vec<Artist>,
}
