use crate::anilist::Media;
use crate::anisong::Anime;
// use axum_sessions::async_session::chrono::{DateTime, Utc};
use crate::japanese_processing::{process_possible_japanese, process_similarity};
use crate::spotify::responses::TrackObject;
use crate::types::{AnimeIndex, AnimeTrackIndex, AnimeType};
use crate::{Error, Result, spotify};
use axum_sessions::async_session::chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::anilist::types::{AnilistID, HexColor, ImageURL, TagID, URL};

#[derive(FromRow, Serialize, Deserialize, Clone)]
pub struct DBAnime {
    pub ann_id: i32,
    pub title_eng: String,
    pub title_jpn: String,
    pub index_type: i16,
    pub index_number: i32,
    pub anime_type: i16,
    pub episodes: Option<i32>,

    pub mean_score: Option<i32>,

    pub banner_image: Option<ImageURL>,

    pub cover_image_color: Option<HexColor>,
    pub cover_image_medium: Option<ImageURL>,
    pub cover_image_large: Option<ImageURL>,
    pub cover_image_extra_large: Option<ImageURL>,

    pub media_format: Option<i16>,
    pub genres: Option<Vec<String>>,
    pub source: Option<String>,
    pub studio_ids: Option<Vec<i32>>,
    pub studio_names: Option<Vec<String>>,
    pub studio_urls: Option<Vec<Option<URL>>>,
    pub tag_ids: Option<Vec<TagID>>,
    pub tag_names: Option<Vec<String>>,
    pub trailer_id: Option<String>,
    pub trailer_site: Option<String>,
    pub thumbnail: Option<ImageURL>,

    pub release_year: Option<i32>,
    pub release_season: Option<i16>,

    // Song info
    pub ann_song_id: i32,
    pub song_name: String,
    pub spotify_artist_ids: Option<Vec<String>>,
    // pub spotify_title: String, // ?
    pub artist_names: Vec<String>,
    pub artists_ann_id: Vec<i32>,
    pub composers_ann_id: Vec<i32>,
    pub arrangers_ann_id: Vec<i32>,

    pub track_index_type: i16,
    pub track_index_number: i32,
    // linked_ids
    pub mal_id: Option<i32>,
    pub anilist_id: Option<AnilistID>,
    pub anidb_id: Option<i32>,
    pub kitsu_id: Option<i32>,

    pub song_group_id: Option<i32>,
    // pub date_added: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

impl DBAnime {
    pub fn pick_best_by_song_name<'a>(
        animes: &'a Vec<DBAnime>,
        song_name: &String,
    ) -> Result<(Vec<&'a DBAnime>, f32)> {
        if animes.len() == 0 {
            return Ok((vec![], 0.0));
        }
        let mut evaluated_animes: Vec<(&DBAnime, f32)> = animes
            .iter()
            .map(|a| (a, process_similarity(&song_name, &a.song_name)))
            .sorted_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .collect();
        let max_score = evaluated_animes[0].1;
        evaluated_animes.retain(|a| a.1 == max_score);
        Ok((evaluated_animes.iter().map(|a| a.0).collect(), max_score))
    }

    pub fn from_anisong_and_anilist(
        anisong: &Anime,
        anilist: Option<&Media>,
        track: Option<&TrackObject>,
        group_id: Option<i32>,
    ) -> Self {
        let anime_index = AnimeIndex::from_str(&anisong.animeCategory).unwrap();
        let anime_type = AnimeType::from_str(&anisong.animeType.as_ref().unwrap()).unwrap();
        let cover_image = anilist.map(|a| a.cover_image.as_ref()).flatten();
        let studios = anilist
            .map(|a| a.studios.as_ref().map(|s| &s.nodes))
            .flatten();
        let tags = anilist.map(|a| a.tags.as_ref()).flatten();
        let trailer = anilist.map(|a| a.trailer.as_ref()).flatten();
        let spotify_artist_ids = track.map(|t| {
            t.artists
                .iter()
                .map(|a| a.id.clone())
                .collect::<Vec<String>>()
        });
        let track_index = AnimeTrackIndex::from_str(&anisong.songType).unwrap();
        Self {
            ann_id: anisong.annId,
            title_eng: anisong.animeENName.clone(),
            title_jpn: anisong.animeJPName.clone(),
            index_type: anime_index.discriminant() as i16,
            index_number: anime_index.value(),
            anime_type: anime_type as i16,
            mean_score: anilist.map(|a| a.mean_score),
            banner_image: anilist.map(|a| a.banner_image.clone()).flatten(),
            cover_image_color: cover_image.map(|b| b.color.clone()).flatten(),
            cover_image_medium: cover_image.map(|b| b.medium.clone()).flatten(),
            cover_image_large: cover_image.map(|b| b.large.clone()).flatten(),
            cover_image_extra_large: cover_image.map(|b| b.extra_large.clone()).flatten(),
            media_format: anilist
                .map(|a| a.format.as_ref().map(|f| f.clone() as i16))
                .flatten(),
            genres: anilist.map(|a| a.genres.clone()).flatten(),
            source: anilist.map(|a| a.source.clone()).flatten(),
            studio_ids: studios.map(|s| s.iter().map(|s| s.id).collect()),
            studio_names: studios
                .as_ref()
                .map(|s| s.iter().map(|s| s.name.clone()).collect()),
            studio_urls: studios.map(|s| s.iter().map(|s| s.site_url.clone()).collect()),
            episodes: anilist.map(|a| a.episodes).flatten(),
            tag_ids: tags.map(|a| a.iter().map(|t| t.id.clone()).collect()),
            tag_names: tags.map(|a| a.iter().map(|t| t.name.clone()).collect()),
            trailer_id: trailer.map(|a| a.id.clone()),
            trailer_site: trailer.map(|t| t.site.clone()),
            thumbnail: trailer.map(|t| t.thumbnail.clone()),
            release_year: anilist.map(|a| a.season_year).flatten(),
            release_season: anilist
                .map(|a| a.season.as_ref().map(|s| s.to_owned() as i16))
                .flatten(),
            ann_song_id: anisong.annSongId,
            song_name: anisong.songName.clone(),
            spotify_artist_ids: spotify_artist_ids,
            artist_names: anisong.artists.iter().map(|a| a.names[0].clone()).collect(),
            artists_ann_id: anisong.artists.iter().map(|a| a.id).collect(),
            composers_ann_id: anisong.composers.iter().map(|a| a.id).collect(),
            arrangers_ann_id: anisong.arrangers.iter().map(|a| a.id).collect(),
            track_index_type: track_index.discriminant() as i16,
            track_index_number: track_index.value(),
            mal_id: anisong.linked_ids.myanimelist,
            anilist_id: anisong.linked_ids.anilist,
            anidb_id: anisong.linked_ids.anidb,
            kitsu_id: anisong.linked_ids.kitsu,
            song_group_id: group_id,
            last_updated: Utc::now(),
        }
    }

    // expects that the Media vec is sorted by id and the Anime vec is sorted by anilist_id
    pub fn from_anisongs_and_anilists(
        anisongs: &Vec<Anime>,
        anilists: &Vec<Media>,
        track: Option<&TrackObject>,
        group_id: Option<i32>,
    ) -> Result<Vec<DBAnime>> {
        if anisongs.is_empty() || anilists.is_empty() {
            return Ok(vec![]);
        }
        let mut anilist_index = 0;
        let mut anisong_index = 0;
        let mut db_animes = Vec::with_capacity(anisongs.len());
        while anilist_index < anilists.len() && anisong_index < anisongs.len() {
            let media = &anilists[anilist_index];
            let anisong = &anisongs[anisong_index];

            let db_anime = if anisong.linked_ids.anilist.is_none() {
                anisong_index += 1;
                Self::from_anisong_and_anilist(anisong, None, track, group_id)
            } else if anisong.linked_ids.anilist.unwrap() == media.id {
                anisong_index += 1;
                Self::from_anisong_and_anilist(anisong, Some(&media), track, group_id)
            } else {
                match media.id > anisong.linked_ids.anilist.unwrap() {
                    true => {
                        anilist_index += 1;
                        continue;
                    }
                    false => {
                        anisong_index += 1;
                        Self::from_anisong_and_anilist(anisong, None, track, group_id)
                    }
                }
            };
            db_animes.push(db_anime);
        }
        Ok(db_animes)
    }
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct DBArtist {
    pub spotify_id: String,
    pub ann_id: i32,
    pub names: Vec<String>,
    pub groups_ids: Option<Vec<i32>>,
    pub members: Option<Vec<i32>>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct SongGroup {
    pub group_id: i32,
    pub song_title: String,
    pub artist_ids: Vec<i32>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct SongGroupLink {
    pub spotify_id: String,
    pub group_id: i32,
}
