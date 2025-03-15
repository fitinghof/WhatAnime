use crate::anilist::Media;
use crate::anisong::Anime;
// use axum_sessions::async_session::chrono::{DateTime, Utc};
use crate::Result;
use crate::japanese_processing::process_similarity;
use crate::types::{AnimeIndex, AnimeTrackIndex, AnimeType};
use axum_sessions::async_session::chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::anilist::types::{AnilistID, HexColor, ImageURL, TagID, URL};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
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
    pub fn pick_best_by_song_name(
        animes: &mut Vec<DBAnime>,
        song_name: &String,
    ) -> Result<(Vec<DBAnime>, f32)> {
        if animes.is_empty() {
            return Ok((vec![], 0.0));
        }

        // Compute similarity scores
        let evaluated_animes: Vec<f32> = animes
            .iter()
            .map(|a| process_similarity(song_name, &a.song_name))
            .collect();

        // Find the max score
        let max_score = evaluated_animes
            .iter()
            .map(|score| *score)
            .fold(f32::MIN, f32::max);

        // Collect indices of the best matches
        let mut best_animes = Vec::new();
        let mut i = evaluated_animes.len();

        while i > 0 {
            i -= 1;
            if evaluated_animes[i] == max_score {
                best_animes.push(animes.swap_remove(i));
            }
        }

        Ok((best_animes, max_score))
    }

    pub fn from_anisong_and_anilist(
        anisong: &Anime,
        anilist: Option<&Media>,
        //track: Option<&TrackObject>,
        group_id: Option<i32>,
    ) -> Self {
        let anime_index = AnimeIndex::from_str(&anisong.animeCategory).unwrap();
        let anime_type = AnimeType::from_str(anisong.animeType.as_ref().map(|a| a.as_str()));
        let cover_image = anilist.map(|a| a.cover_image.as_ref()).flatten();
        let studios = anilist
            .map(|a| a.studios.as_ref().map(|s| &s.nodes))
            .flatten();
        let tags = anilist.map(|a| a.tags.as_ref()).flatten();
        let trailer = anilist.map(|a| a.trailer.as_ref()).flatten();
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
            spotify_artist_ids: /*spotify_artist_ids*/ Some(vec![]),
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

    pub fn update(&mut self, anilist_data: &Media) {
        let cover_image = anilist_data.cover_image.as_ref();
        let studios = anilist_data.studios.as_ref().map(|s| &s.nodes);
        let tags = anilist_data.tags.as_ref();
        let trailer = anilist_data.trailer.as_ref();
        self.mean_score = Some(anilist_data.mean_score);
        self.banner_image = anilist_data.banner_image.clone();
        self.cover_image_color = cover_image.map(|b| b.color.clone()).flatten();
        self.cover_image_medium = cover_image.map(|b| b.medium.clone()).flatten();
        self.cover_image_large = cover_image.map(|b| b.large.clone()).flatten();
        self.cover_image_extra_large = cover_image.map(|b| b.extra_large.clone()).flatten();
        self.media_format = anilist_data.format.as_ref().map(|f| f.clone() as i16);
        self.genres = anilist_data.genres.clone();
        self.source = anilist_data.source.clone();
        self.studio_ids = studios.map(|s| s.iter().map(|s| s.id).collect());
        self.studio_names = studios
            .as_ref()
            .map(|s| s.iter().map(|s| s.name.clone()).collect());
        self.studio_urls = studios.map(|s| s.iter().map(|s| s.site_url.clone()).collect());
        self.episodes = anilist_data.episodes;
        self.tag_ids = tags.map(|a| a.iter().map(|t| t.id.clone()).collect());
        self.tag_names = tags.map(|a| a.iter().map(|t| t.name.clone()).collect());
        self.trailer_id = trailer.map(|a| a.id.clone());
        self.trailer_site = trailer.map(|t| t.site.clone());
        self.thumbnail = trailer.map(|t| t.thumbnail.clone());
        self.release_year = anilist_data.season_year;
        self.release_season = anilist_data.season.as_ref().map(|s| s.to_owned() as i16);
        self.last_updated = Utc::now();
    }

    pub fn update_all(
        db_animes: &mut Vec<DBAnime>,
        new_anilist: &Vec<Media>,
        group_id: Option<i32>,
    ) -> Vec<DBAnime> {
        db_animes.sort_by(|a, b| {
            a.anilist_id
                .unwrap_or(AnilistID(-1))
                .cmp(&b.anilist_id.unwrap_or(AnilistID(-1)))
        });
        let mut updated_anime = Vec::new();

        let mut anilist_index = 0;
        let mut db_anime_index = 0;
        while anilist_index < new_anilist.len() && db_anime_index < db_animes.len() {
            let media = &new_anilist[anilist_index];
            let db_anime = &mut db_animes[db_anime_index];

            if db_anime.anilist_id.is_some_and(|a| a == media.id) {
                db_anime_index += 1;
                db_anime.update(media);
                if group_id.is_none() {
                    updated_anime.push(db_anime.clone());
                }
            } else {
                match db_anime.anilist_id.is_none_or(|id| id < media.id) {
                    true => {
                        db_anime_index += 1;
                    }
                    false => {
                        anilist_index += 1;
                    }
                }
            };
        }
        if group_id.is_some() {
            for dbanime in db_animes {
                if dbanime.song_group_id.is_none() {
                    dbanime.song_group_id = group_id;
                    updated_anime.push(dbanime.clone());
                }
            }
        }
        updated_anime
    }

    // expects that the Media vec is sorted by id and the Anime vec is sorted by anilist_id
    pub fn from_anisongs_and_anilists(
        anisongs: &Vec<Anime>,
        anilists: &Vec<Media>,
        // track: Option<&TrackObject>,
        group_id: Option<i32>,
    ) -> Result<Vec<DBAnime>> {
        let mut anisongs_sorted: Vec<&Anime> = anisongs.iter().collect();

        anisongs_sorted.sort_by(|&a, &b| {
            a.linked_ids
                .anilist
                .unwrap_or(AnilistID(-1))
                .cmp(&b.linked_ids.anilist.unwrap_or(AnilistID(-1)))
        });

        let mut anilist_index = 0;
        let mut anisong_index = 0;
        let mut db_animes = Vec::with_capacity(anisongs.len());

        while anilist_index < anilists.len() && anisong_index < anisongs.len() {
            let media = &anilists[anilist_index];
            let anisong = &anisongs_sorted[anisong_index];

            let db_anime = if anisong.linked_ids.anilist.is_none() {
                anisong_index += 1;
                Self::from_anisong_and_anilist(anisong, None, group_id)
            } else if anisong.linked_ids.anilist.unwrap() == media.id {
                anisong_index += 1;
                Self::from_anisong_and_anilist(anisong, Some(&media), group_id)
            } else {
                match media.id < anisong.linked_ids.anilist.unwrap() {
                    true => {
                        anilist_index += 1;
                        continue;
                    }
                    false => {
                        anisong_index += 1;
                        Self::from_anisong_and_anilist(anisong, None, group_id)
                    }
                }
            };
            db_animes.push(db_anime);
        }
        while anisong_index < anisongs_sorted.len() {
            db_animes.push(Self::from_anisong_and_anilist(
                anisongs_sorted[anisong_index],
                None,
                group_id,
            ));
            anisong_index += 1;
        }
        Ok(db_animes)
    }

    pub fn is_outdated(&self) -> bool {
        return self.last_updated + Duration::days(7) < Utc::now();
    }
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
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
