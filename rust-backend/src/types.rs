use axum::response::IntoResponse;
use serde::{Serialize, Deserialize};
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

}

impl AnimeType{
    pub fn new(type_string: &str) -> Result<Self> {
        match type_string {
            "TV" => Ok(AnimeType::TV),
            "Moive" => Ok(AnimeType::Movie),
            "OVA" => Ok(AnimeType::OVA),
            "ONA" => Ok(AnimeType::ONA),
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

        let track_type = parts[0];

        match track_type {
            "Opening" => Ok(AnimeTrackIndex::Opening(track_number)),
            "Insert" => Ok(AnimeTrackIndex::Insert(0)),
            "Ending" => Ok(AnimeTrackIndex::Ending(track_number)),
            _ => Err(Error::ParseError(input.to_string())),
        }
    }
}


#[derive(Serialize)]
enum AnimeIndex {
    Season(u32),
    Movie(u32),
    ONA{year: u32}
}

impl AnimeIndex {
    pub fn from_str(anime_category: &str) -> Result<Self> {
        let parts: Vec<&str> = anime_category.split_whitespace().collect();

        let track_number = match parts.len() {
            1 => 1 as u32,
            2 => {
                match parts[1].parse::<u32>() {
                    Ok(value) => value,
                    Err(_) => return Err(Error::ParseError(anime_category.to_string())),
                }
            }
            _ => return Err(Error::ParseError(anime_category.to_string())),
        };

        let track_type = parts[0];

        match track_type {
            "Season" => Ok(AnimeIndex::Season(track_number)),
            "Movie" => Ok(AnimeIndex::Movie(track_number)),
            "ONA" => Ok(AnimeIndex::ONA{year: track_number}),
            _ => Err(Error::ParseError(anime_category.to_string())),
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
    pub image_url: String,
}
impl Anime {
    pub fn new(anisong_anime: &anisong::Anime, jikan_anime: &JikanAnime) -> Result<Self> {
        Ok(Self {
                    title: anisong_anime.anime_en_name.clone(),
                    title_japanese: anisong_anime.anime_jp_name.clone(),
                    anime_index: AnimeIndex::from_str(&anisong_anime.anime_category)?,
                    track_index: AnimeTrackIndex::from_str(&anisong_anime.song_type)?,

                    anime_type: AnimeType::new(&anisong_anime.anime_type).ok(),
                    image_url: jikan_anime.images.webp.image_url.clone(),
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