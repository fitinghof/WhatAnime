use std::str::FromStr;

use crate::{Error, Result};
use axum::http::response;
use num_enum::TryFromPrimitive;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, Type};
// use serde_json::to_string;

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromRow, Deserialize, Serialize, Clone, Copy, Type,
)]
#[sqlx(transparent)]
pub struct AnilistID(pub i32);

impl From<i32> for AnilistID {
    fn from(id: i32) -> Self {
        Self { 0: id }
    }
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromRow, Deserialize, Serialize, Type, Clone,
)]
#[sqlx(transparent)]
pub struct ImageURL(URL);

impl ImageURL {
    pub fn from_str(s: &str) -> Self {
        Self {
            0: URL::from_str(s),
        }
    }
}

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromRow, Deserialize, Serialize, Type, Clone,
)]
#[sqlx(transparent)]
pub struct HexColor(String);

#[derive(Deserialize, Serialize, FromRow)]
pub struct CoverImage {
    pub color: Option<HexColor>,
    pub medium: Option<ImageURL>,
    pub large: Option<ImageURL>,
    #[serde(rename = "extraLarge")]
    pub extra_large: Option<ImageURL>,
}

#[derive(Debug, Deserialize, Serialize, TryFromPrimitive, Clone)]
#[repr(i16)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaFormat {
    Tv,
    TvShort,
    Movie,
    Special,
    Ova,
    Ona,
    Music,
    Manga,
    Novel,
    OneShot,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromRow, Deserialize, Serialize, Type)]
#[sqlx(transparent)]
pub struct Genre(String);

// #[derive(Debug, Deserialize, Serialize, TryFromPrimitive, Clone)]
// #[repr(i16)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// pub enum MediaSource {
//     Original,
//     Manga,
//     LightNovel,
//     VisualNovel,
//     VideoGame,
//     Other,
//     Novel,
//     Doujinshi,
//     Anime,
//     WebNovel,
//     LiveAction,
//     Game,
//     Comic,
//     MultimediaProject,
//     PictureBook,
// }

#[derive(Debug, Deserialize, Serialize, TryFromPrimitive, Clone)]
#[repr(i16)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReleaseSeason {
    Winter,
    Spring,
    Summer,
    Fall,
}

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromRow, Deserialize, Serialize, Type, Clone,
)]
#[sqlx(transparent)]
pub struct URL(String);

impl URL {
    pub fn from_str(s: &str) -> Self {
        Self { 0: s.to_string() }
    }
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct Studio {
    pub id: i32,
    pub name: Option<String>,
    #[serde(rename = "siteUrl")]
    pub site_url: Option<URL>,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct StudioConnection {
    // edges: StudioEdge
    pub nodes: Vec<Studio>, // pageInfo: PageInfo
}
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromRow, Deserialize, Serialize, Type, Clone,
)]
#[sqlx(transparent)]
pub struct TagID(i32);
#[derive(Deserialize, Serialize, FromRow)]
pub struct MediaTag {
    pub id: TagID,
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct MediaTrailer {
    pub id: String,
    pub site: String,
    pub thumbnail: ImageURL,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct Media {
    pub id: AnilistID,
    pub title: MediaTitle,
    #[serde(rename = "meanScore")]
    pub mean_score: i32,
    #[serde(rename = "bannerImage")]
    pub banner_image: Option<ImageURL>,
    #[serde(rename = "coverImage")]
    pub cover_image: Option<CoverImage>,
    pub format: Option<MediaFormat>,
    pub genres: Option<Vec<Genre>>,
    pub source: Option<String>,
    pub studios: Option<StudioConnection>,
    pub tags: Option<Vec<MediaTag>>,
    pub trailer: Option<MediaTrailer>,
    pub episodes: Option<i32>,
    pub season: Option<ReleaseSeason>,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<i32>,
}

impl Media {
    pub async fn fetch_one(id: AnilistID) -> Option<Media> {
        let anime = Self::fetch_many(vec![id]).await.unwrap();
        if anime.len() == 1 {
            Some(anime.into_iter().next().unwrap())
        } else {
            None
        }
    }
    pub async fn fetch_many(ids: Vec<AnilistID>) -> Result<Vec<Media>> {
        let mut all_media: Vec<Media> = Vec::new();
        let mut page = 1;
        let per_page = 50;

        loop {
            let json_body = json!({
                "query": QUERY_STRING,
                "variables": {
                    "ids": &ids,
                    "isMain": false,
                    "page": page,
                    "perPage": per_page,
                }
            });

            let response = Client::new()
                .post("https://graphql.anilist.co")
                .json(&json_body)
                .send()
                .await
                .unwrap();

            if response.status().is_success() {
                let data: AnilistResponse = response.json().await.unwrap();
                all_media.extend(data.data.page.media);

                if data.data.page.page_info.is_none_or(|a| !a.has_next_page) {
                    break;
                }
                page += 1;
            } else {
                println!("{}", response.text().await.unwrap());
                break;
            }
        }
        Ok(all_media)
    }
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct MediaList {
    media: Vec<Media>,
    #[serde(rename = "pageInfo")]
    page_info: Option<PageInfo>,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct PageData {
    #[serde(rename = "Page")]
    page: MediaList,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct AnilistResponse {
    pub data: PageData,
}

const QUERY_STRING: &str = r#"
query ($ids: [Int] = [170695], $isMain: Boolean = true, $version: Int = 3, $page: Int, $perPage: Int) {
	Page(page: $page, perPage: $perPage) {
		media(id_in: $ids) {
			id
			title {
				romaji
				english
				native
			}
			averageScore
			bannerImage
			coverImage {
				medium
				large
				extraLarge
				color
			}
			format
			genres
			meanScore
			source(version: $version)
			studios(isMain: $isMain) {
				nodes {
					name
					id
					siteUrl
				}
			}
			tags {
				id
				name
			}
			trailer {
				site
				thumbnail
				id
			}
			episodes
    season
    seasonYear
  }
  pageInfo {
    hasNextPage
  }
}
}
"#;
