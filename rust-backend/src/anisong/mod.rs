use crate::{
    Error, Result,
    anilist::types::AnilistID,
    japanese_processing::{normalize_text, process_possible_japanese, process_similarity},
    spotify::responses::TrackObject,
};
use core::f32;

use fuzzywuzzy::fuzz;
use itertools::Itertools;
use log::{error, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AnisongClient {
    client: Client,
}

impl AnisongClient {
    const SEARCH_REQUEST_URL: &str = "https://anisongdb.com/api/search_request";
    const ARTIST_ID_SEARCH_REQUEST_URL: &str = "https://anisongdb.com/api/artist_ids_request";

    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_animes_by_artists_ids(&self, ids: Vec<i32>) -> Result<Vec<Anime>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let search = ArtistIDSearchRequest {
            artist_ids: ids,
            group_granularity: 0,
            max_other_artist: 0,
            ignore_duplicate: false,
            opening_filter: true,
            ending_filter: true,
            insert_filter: true,
            normal_broadcast: true,
            dub: true,
            rebroadcast: true,
            standard: true,
            instrumental: true,
            chanting: true,
            character: true,
        };

        let response = self
            .client
            .post(Self::ARTIST_ID_SEARCH_REQUEST_URL)
            .json(&search)
            .send()
            .await?;

        match response.status() {
            value if value.is_success() => Ok(response.json().await?),
            value if value == 503 => Ok(vec![]),
            _ => {
                let status = response.status();
                warn!("{}", response.text().await.unwrap());
                Err(Error::BadRequest {
                    url: Self::SEARCH_REQUEST_URL.to_string(),
                    status_code: status,
                })
            }
        }
    }

    pub async fn get_exact_song(
        &self,
        artist_ids: Vec<i32>,
        song_title: String,
    ) -> Result<Vec<Anime>> {
        Ok(self
            .get_animes_by_artists_ids(artist_ids)
            .await
            .unwrap()
            .into_iter()
            .filter(|a| a.songName == song_title)
            .collect::<Vec<Anime>>())
    }

    pub async fn get_animes_by_song_title(
        &self,
        title: String,
        partial: bool,
    ) -> Result<Vec<Anime>> {
        let search = SearchRequest {
            artist_search_filter: None,
            anime_search_filter: None,
            song_name_search_filter: Some(SearchFilter {
                search: title,
                partial_match: partial,
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
            .post(Self::SEARCH_REQUEST_URL)
            .json(&search)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            println!("{}", response.text().await?);
            Ok(vec![])
        }
    }

    pub async fn get_animes_by_artist_name(
        &self,
        artist: Option<&String>,
        composer: Option<&String>,
    ) -> Result<Vec<Anime>> {
        let search = SearchRequest {
            anime_search_filter: None,
            song_name_search_filter: None,
            artist_search_filter: match artist {
                Some(name) => Some(SearchFilter {
                    search: name.clone(),
                    partial_match: false,
                    group_granularity: Some(0),
                    max_other_artist: Some(99),
                    arrangement: Some(true),
                }),
                None => None,
            },
            composer_search_filter: match composer {
                Some(name) => Some(SearchFilter {
                    search: name.clone(),
                    partial_match: false,
                    group_granularity: Some(0),
                    max_other_artist: Some(99),
                    arrangement: Some(true),
                }),
                None => None,
            },
            and_logic: Some(false),
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
            .post(Self::SEARCH_REQUEST_URL)
            .json(&search)
            .send()
            .await
            .unwrap();

        match response.status().is_success() {
            true => Ok(response.json().await.unwrap()),
            false => {
                error!("Anisong fetch {}", response.text().await.unwrap());
                Ok(vec![])
            }
        }
    }
    pub async fn find_songs_by_artists(&self, song: &TrackObject) -> Result<Vec<Anime>> {
        let artists = &song.artists;
        let mut anime_song_entries = Vec::new();

        for artist in artists {
            let romanji_artist = process_possible_japanese(&artist.name);
            let songs = self
                .get_animes_by_artist_name(Some(&romanji_artist), Some(&romanji_artist))
                .await
                .unwrap();
            anime_song_entries.extend(songs);
        }
        Ok(anime_song_entries)
    }

    pub fn pick_best_by_song_name<'a>(
        animes: &mut Vec<Anime>,
        song_name: &String,
    ) -> Result<(Vec<Anime>, f32)> {
        if animes.len() == 0 {
            return Ok((vec![], 0.0));
        }

        let evaluations: Vec<f32> = animes
            .iter()
            .map(|a| process_similarity(&song_name, &a.songName))
            .collect();

        let max_score = evaluations.iter().map(|s| *s).fold(f32::MIN, f32::max);

        let mut best_animes = Vec::new();
        let mut i = evaluations.len();

        while i > 0 {
            i -= 1;
            if evaluations[i] == max_score {
                best_animes.push(animes.swap_remove(i));
            }
        }

        Ok((best_animes, max_score))
    }

    pub fn pick_best_by_artist_names<'a>(
        animes: &mut Vec<Anime>,
        artist_names: Vec<&String>,
    ) -> Result<(Vec<Anime>, f32)> {
        if animes.len() == 0 {
            return Ok((vec![], 0.0));
        }

        let artist_names = artist_names.into_iter().join(" ");
        let evaluations: Vec<f32> = animes
            .iter()
            .map(|a| {
                let anisong_artists_names = a.artists.iter().map(|b| &b.names[0]).join(" ");
                fuzz::token_set_ratio(
                    &normalize_text(&process_possible_japanese(&artist_names)),
                    &normalize_text(&anisong_artists_names),
                    true,
                    true,
                ) as f32
            })
            .collect();

        let max_score = evaluations.iter().map(|s| *s).fold(f32::MIN, f32::max);

        let mut best_animes = Vec::new();
        let mut i = evaluations.len();

        while i > 0 {
            i -= 1;
            if evaluations[i] == max_score {
                best_animes.push(animes.swap_remove(i));
            }
        }

        Ok((best_animes, max_score))
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub anilist: Option<AnilistID>,
    pub kitsu: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Anime {
    pub annId: i32,
    pub annSongId: i32,
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
