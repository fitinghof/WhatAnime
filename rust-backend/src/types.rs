use axum::response::IntoResponse;
use serde::{Serialize, Deserialize};
use serde_json::to_string;
use crate::{anisong, Error, Result};

#[derive(Serialize)]
pub struct SongInfo {
    pub title: String,
    pub artists: Vec<String>,
    pub album_picture_url: String,
}
#[derive(Serialize)]
pub enum AnimeType {
    TV,
    Movie,
    OVA,
    ONA,
    Special,
}

impl AnimeType{
    pub fn new(type_string: &str) -> Result<Self> {
        match type_string {
            "TV" => Ok(AnimeType::TV),
            "Movie" => Ok(AnimeType::Movie),
            "OVA" => Ok(AnimeType::OVA),
            "ONA" => Ok(AnimeType::ONA),
            "Special" => Ok(AnimeType::Special),
            _ => Err(Error::ParseError(type_string.to_string())),
        }
    }
}
#[derive(Serialize)]
pub enum AnimeTrackIndex {
    Opening(u32),
    Insert(u32),
    Ending(u32),
}

impl AnimeTrackIndex {
    pub fn from_str(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.split_whitespace().collect();


        let track_number = if parts.len() == 1 {
            1 as u32
        } else if parts.len() == 2 {
            if parts[0] != "Insert" {
                match parts[1].parse::<u32>() {
                    Ok(value) => value,
                    Err(_) => return Err(Error::ParseError(input.to_string())),
                }
            }
            else {
                0
            }
        }
        else {
            return Err(Error::ParseError(input.to_string()));
        };

        let anime_index_type = parts[0];

        match anime_index_type {
            "Opening" => Ok(AnimeTrackIndex::Opening(track_number)),
            "Insert" => Ok(AnimeTrackIndex::Insert(0)),
            "Ending" => Ok(AnimeTrackIndex::Ending(track_number)),
            _ => {
                println!("Found weird anime index type: {} number: {}", anime_index_type, track_number);
                Err(Error::ParseError(input.to_string()))
            },
        }
    }
}


#[derive(Serialize)]
pub enum AnimeIndex {
    Season(u32),
    Movie(u32),
    ONA(u32),
    OVA(u32),
    TVSpecial(u32),
    Special(u32),
    MusicVideo(u32),
}

fn split_string(input: &str) -> (String, Option<u32>) {
    let mut words: Vec<&str> = input.split_whitespace().collect();
    if let Some(last) = words.last() {
        if let Ok(num) = last.parse::<u32>() {
            words.pop();
            let text = words.join(" ");
            return (text, Some(num));
        }
    }
    (input.to_owned(), None)
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
            },
        }
    }
    }


#[derive(Serialize)]
pub struct Anime {
    pub title: String,
    pub title_japanese: String,
    pub anime_index: AnimeIndex,
    pub track_index: AnimeTrackIndex,
    pub anime_type: Option<AnimeType>,
    pub image_url: Option<String>,
    pub linked_ids: anisong::AnimeListLinks
}
impl Anime {
    pub fn new(anisong_anime: &anisong::Anime, image_url: Option<String>) -> Result<Self> {
        let anime_type = if anisong_anime.animeType.is_some() {
            AnimeType::new(&anisong_anime.animeType.as_ref().unwrap()).ok()
        }
        else {
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
                })
    }
}

#[derive(Serialize)]
pub struct SongHit {
    pub song_info: SongInfo,
    pub certainty: i32,
    pub anime_info: Vec<Anime>,
    pub more_with_artist: Vec<Anime>,
}
#[derive(Serialize)]
pub struct SongMiss {
    pub song_info: SongInfo,
    pub possible_anime: Vec<Anime>,
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
    pub data: JikanAnime
}
// Jikan API response types
#[derive(Debug, Serialize, Deserialize)]
pub struct JikanAnime {
    pub mal_id: u32,
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
    pub episodes: Option<u32>,
    pub status: String,
    pub airing: bool,
    pub aired: Aired,
    pub duration: String,
    pub rating: String,
    pub score: Option<f32>,
    pub scored_by: Option<u32>,
    pub rank: Option<u32>,
    pub popularity: u32,
    pub members: u32,
    pub favorites: u32,
    pub synopsis: Option<String>,
    pub background: Option<String>,
    pub season: Option<String>,
    pub year: Option<u32>,
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
    pub day: Option<u32>,
    pub month: Option<u32>,
    pub year: Option<u32>,
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
    pub mal_id: u32,
    #[serde(rename = "type")]
    pub producer_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Genre {
    pub mal_id: u32,
    #[serde(rename = "type")]
    pub genre_type: String,
    pub name: String,
    pub url: String,
}