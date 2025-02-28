use std::fmt::format;

use crate::{anisong::{self, Anime, AnimeListLinks}, database::{databasetypes::DBAnime, find_anime_no_db::fetch_jikan }, spotify::responses::TrackObject, Error, Result};
use axum::response::IntoResponse;
use futures::future::join_all;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

#[derive(Serialize)]
pub struct SongInfo {
    pub title: String,
    pub artists: Vec<String>,
    pub album_picture_url: String,
    pub spotify_id: String,
}

impl SongInfo {
    pub fn from_track_obj(track_object: &TrackObject) -> Self {
        Self {
            title: track_object.name.clone(),
            artists: track_object.artists.iter().map(|a| a.name.clone()).collect(),
            album_picture_url: track_object.album.images[0].url.clone(),
            spotify_id: track_object.id.clone(),
        }
    }
}

fn split_string(input: &str) -> (String, Option<i32>) {
    let mut words: Vec<&str> = input.split_whitespace().collect();
    if let Some(last) = words.last() {
        if let Ok(num) = last.parse::<i32>() {
            words.pop();
            let text = words.join(" ");
            return (text, Some(num));
        }
    }
    (input.to_owned(), None)
}
#[derive(Serialize)]
pub enum AnimeType {
    TV,
    Movie,
    OVA,
    ONA,
    Special,
}

impl AnimeType {
    pub fn from_str(type_string: &str) -> Result<Self> {
        match type_string {
            "TV" => Ok(Self::TV),
            "Movie" => Ok(Self::Movie),
            "OVA" => Ok(Self::OVA),
            "ONA" => Ok(Self::ONA),
            "Special" => Ok(Self::Special),
            _ => Err(Error::ParseError(type_string.to_string())),
        }
    }
    pub fn from_db(discriminator: i16) -> Result<Self> {
        match discriminator {
            0 => Ok(Self::TV),
            1 => Ok(Self::Movie),
            2 => Ok(Self::OVA),
            3 => Ok(Self::ONA),
            4 => Ok(Self::Special),
            _ => Err(Error::ParseError(discriminator.to_string()))
        }
    }
}
#[derive(Serialize)]
#[repr(u8)]
pub enum AnimeTrackIndex {
    Opening(i32),
    Insert(i32),
    Ending(i32),
}

impl AnimeTrackIndex {
    pub fn from_str(input: &str) -> Result<Self> {
        let (anime_index_type, track_number) = split_string(input);

        let match_str: &str = &anime_index_type;
        match match_str {
            "Opening" => Ok(AnimeTrackIndex::Opening(track_number.unwrap_or(1))),
            "Insert Song" => Ok(AnimeTrackIndex::Insert(track_number.unwrap_or(0))),
            "Ending" => Ok(AnimeTrackIndex::Ending(track_number.unwrap_or(1))),
            _ => {
                println!("Found weird Anime track index type: {}", &input);
                Err(Error::ParseError(input.to_string()))
            }
        }
    }
    pub fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
    pub fn value(&self) -> i32 {
        match self {
            AnimeTrackIndex::Opening(val)
            | AnimeTrackIndex::Insert(val)
            | AnimeTrackIndex::Ending(val) => *val,
        }
    }
    pub fn from_db(discriminator: i16, value:i32) -> Result<Self> {
        match discriminator {
            0 => Ok(AnimeTrackIndex::Opening(value)),
            1 => Ok(AnimeTrackIndex::Insert(value)),
            2 => Ok(AnimeTrackIndex::Ending(value)),
            _ => Err(Error::ParseError(format!("{}:{}", discriminator, value)))
        }
    }
}


#[derive(Serialize)]
#[repr(u8)]
pub enum AnimeIndex {
    Season(i32),
    Movie(i32),
    ONA(i32),
    OVA(i32),
    TVSpecial(i32),
    Special(i32),
    MusicVideo(i32),
}

impl AnimeIndex {
    // funny little tihi
    pub fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }

    pub fn value(&self) -> i32 {
        match self {
            AnimeIndex::Season(val)
            | AnimeIndex::Movie(val)
            | AnimeIndex::ONA(val)
            | AnimeIndex::OVA(val)
            | AnimeIndex::TVSpecial(val)
            | AnimeIndex::Special(val)
            | AnimeIndex::MusicVideo(val) => *val,
        }
    }
}

impl AnimeIndex {
    pub fn from_str(anime_category: &str) -> Result<Self> {
        let (anime_index_type, track_number) = split_string(anime_category);

        let match_str: &str = &anime_index_type;
        match match_str {
            "TV" => Ok(AnimeIndex::Season(0)),
            "Season" => Ok(AnimeIndex::Season(track_number.unwrap_or(1))),
            "Movie" => Ok(AnimeIndex::Movie(track_number.unwrap_or(1))),
            "ONA" => Ok(AnimeIndex::ONA(track_number.unwrap_or(0))),
            "OVA" => Ok(AnimeIndex::OVA(track_number.unwrap_or(1))),
            "TV Special" => Ok(AnimeIndex::TVSpecial(track_number.unwrap_or(1))),
            "Special" => Ok(AnimeIndex::Special(track_number.unwrap_or(1))),
            "Music Video" => Ok(AnimeIndex::MusicVideo(track_number.unwrap_or(1))),
            _ => {
                println!("Found weird track type: {}", anime_index_type);
                Err(Error::ParseError(anime_category.to_string()))
            }
        }
    }
    pub fn from_db(discriminator: i16, value: i32) -> Result<Self> {
        match discriminator {
            0 => Ok(AnimeIndex::Season(value)),
            1 => Ok(AnimeIndex::Movie(value)),
            2 => Ok(AnimeIndex::ONA(value)),
            3 => Ok(AnimeIndex::OVA(value)),
            4 => Ok(AnimeIndex::TVSpecial(value)),
            5 => Ok(AnimeIndex::Special(value)),
            6 => Ok(AnimeIndex::MusicVideo(value)),
            _ => Err(Error::ParseError(format!("{}:{}", discriminator, value)))
        }
    }
}

#[derive(Serialize)]
pub struct FrontendAnimeEntry {
    pub title: String,
    pub title_japanese: String,
    pub anime_index: AnimeIndex,
    pub track_index: AnimeTrackIndex,
    pub anime_type: Option<AnimeType>,
    pub image_url: Option<String>,
    pub linked_ids: anisong::AnimeListLinks,

    pub song_name: String,
    pub artist_ids: Vec<i32>,
    pub artist_names: Vec<String>,
}
impl FrontendAnimeEntry {
    pub fn new(anisong_anime: &anisong::Anime, image_url: Option<String>) -> Result<Self> {
        let anime_type = if anisong_anime.animeType.is_some() {
            AnimeType::from_str(&anisong_anime.animeType.as_ref().unwrap()).ok()
        } else {
            None
        };
        Ok(Self {
            title: anisong_anime.animeENName.clone(),
            title_japanese: anisong_anime.animeJPName.clone(),
            anime_index: AnimeIndex::from_str(&anisong_anime.animeCategory).unwrap(),
            track_index: AnimeTrackIndex::from_str(&anisong_anime.songType).unwrap(),
            anime_type: anime_type,
            image_url,
            linked_ids: anisong_anime.linked_ids.clone(),

            song_name: anisong_anime.songName.clone(),
            artist_ids: anisong_anime.artists.iter().map(|a| a.id.clone()).collect(),
            artist_names: anisong_anime.artists.iter().map(|a| a.names[0].clone()).collect(),
        })
    }
    pub fn from_db(db_anime: &DBAnime) -> Self {
        Self {
            title: db_anime.title_eng.clone(),
            title_japanese: db_anime.title_jpn.clone(),
            anime_index: AnimeIndex::from_db(db_anime.index_type, db_anime.index_number).unwrap(),
            track_index: AnimeTrackIndex::from_db(db_anime.track_index_type, db_anime.track_index_number).unwrap(),
            anime_type: AnimeType::from_db(db_anime.anime_type).ok(),
            image_url: Some(db_anime.image_url_webp_normal.clone()),
            linked_ids: AnimeListLinks {
                myanimelist: db_anime.mal_id,
                anidb: db_anime.anidb_id,
                anilist: db_anime.anilist_id,
                kitsu: db_anime.kitsu_id,
            },
            song_name: db_anime.song_name.clone(),
            artist_ids: db_anime.artists_ann_id.clone(),
            artist_names: db_anime.artist_names.iter().map(|a| a.clone()).collect(),
        }
    }

    pub async fn from_anisong(anisong: &Anime) -> Result<Self>{
        let mut image_url = None;
        if anisong.linked_ids.myanimelist.is_some() {
            image_url = fetch_jikan(anisong.linked_ids.myanimelist.unwrap())
                .await
                .map(|info| info.images.webp.image_url)
                .ok();
        }
        Ok(Self::new(&anisong, image_url).unwrap())
    }

    pub async fn from_anisongs(anisongs: &Vec<&Anime>) -> Result<Vec<FrontendAnimeEntry>> {
        let mut future_anime = Vec::with_capacity(anisongs.len());
        for anime in anisongs {
            future_anime.push(FrontendAnimeEntry::from_anisong(&anime));
        }
        Ok(join_all(future_anime).await.into_iter().map(|a| a.unwrap()).collect())
    }

}

#[derive(Serialize)]
pub struct SongHit {
    pub song_info: SongInfo,
    pub certainty: i32,
    pub anime_info: Vec<FrontendAnimeEntry>,
    pub more_with_artist: Vec<FrontendAnimeEntry>,
}
#[derive(Serialize)]
pub struct SongMiss {
    pub song_info: SongInfo,
    pub possible_anime: Vec<FrontendAnimeEntry>,
}
#[derive(Serialize)]
pub enum NewSong {
    Hit(SongHit),
    Miss(SongMiss),
}
#[derive(Serialize)]
pub enum ContentUpdate {
    NewSong(NewSong),
    LoginRequired,
    NoUpdates,
    NotPlaying,
}

impl IntoResponse for ContentUpdate {
    fn into_response(self) -> axum::response::Response {
        let json = serde_json::to_string(&self).unwrap();
        axum::response::Json(json).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JikanResponses {
    Fail(JikanFailResponse),
    Success(JikanSuccessResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JikanFailResponse {
    status: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JikanSuccessResponse {
    pub data: JikanAnime,
}
// Jikan API response types
#[derive(Debug, Serialize, Deserialize)]
pub struct JikanAnime {
    pub mal_id: i32,
    pub url: String,
    pub images: Images,
    pub trailer: Trailer,
    pub approved: bool,
    pub titles: Vec<Title>,
    pub title: String,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub title_synonyms: Vec<String>,
    #[serde(rename = "type")]
    pub anime_type: String,
    pub source: String,
    pub episodes: Option<i32>,
    pub status: String,
    pub airing: bool,
    pub aired: Aired,
    pub duration: String,
    pub rating: String,
    pub score: Option<f32>,
    pub scored_by: Option<i32>,
    pub rank: Option<i32>,
    pub popularity: i32,
    pub members: i32,
    pub favorites: i32,
    pub synopsis: Option<String>,
    pub background: Option<String>,
    pub season: Option<String>,
    pub year: Option<i32>,
    pub broadcast: Broadcast,
    pub producers: Vec<Producer>,
    pub licensors: Vec<Producer>,
    pub studios: Vec<Producer>,
    pub genres: Vec<Genre>,
    pub explicit_genres: Vec<Genre>,
    pub themes: Vec<Genre>,
    pub demographics: Vec<Genre>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Images {
    pub jpg: ImageSet,
    pub webp: ImageSet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageSet {
    pub image_url: String,
    pub small_image_url: String,
    pub large_image_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trailer {
    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
    pub images: TrailerImages,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrailerImages {
    pub image_url: Option<String>,
    pub small_image_url: Option<String>,
    pub medium_image_url: Option<String>,
    pub large_image_url: Option<String>,
    pub maximum_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Title {
    #[serde(rename = "type")]
    pub title_type: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Aired {
    pub from: Option<String>,
    pub to: Option<String>,
    pub prop: AiredProp,
    pub string: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiredProp {
    pub from: DateInfo,
    pub to: DateInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateInfo {
    pub day: Option<i32>,
    pub month: Option<i32>,
    pub year: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Broadcast {
    pub day: Option<String>,
    pub time: Option<String>,
    pub timezone: Option<String>,
    pub string: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Producer {
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub producer_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Genre {
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub genre_type: String,
    pub name: String,
    pub url: String,
}
