use super::Database;
use crate::anisong::{Anime, AnisongClient};
use crate::japanese_processing::process_possible_japanese;
use crate::spotify::responses::TrackObject;
use crate::types::{
    self, FrontendAnimeEntry, JikanAnime, JikanResponses, NewSong, SongHit,
    SongInfo, SongMiss,
};
use futures::future::join_all;
use crate::{Error, Result};

use fuzzywuzzy::fuzz;
use itertools::Itertools;
use reqwest::Client;

pub async fn fetch_jikan(mal_id: i32) -> Result<JikanAnime> {
    let response = Client::new()
        .get(format!("https://api.jikan.moe/v4/anime/{}", mal_id))
        .send()
        .await?;

    if response.status().is_success() {
        let jikan_response = response.json().await.unwrap();
        match jikan_response {
            JikanResponses::Success(value) => Ok(value.data),
            JikanResponses::Fail(_) => Err(Error::BadRequest {
                url: "https://api.jikan.moe/v4/anime/".to_string(),
                status_code: axum::http::StatusCode::TOO_MANY_REQUESTS,
            }),
        }
    } else {
        Err(Error::BadRequest {
            url: "https://api.jikan.moe/v4/anime/".to_string(),
            status_code: axum::http::StatusCode::TOO_MANY_REQUESTS,
        })
    }
}

impl Database {
    pub async fn find_most_likely_anime(
        &self,
        song: &TrackObject,
        accuracy_cutoff: f32,
        anisong_db: &AnisongClient,
    ) -> Result<NewSong> {
        let romanji_title = process_possible_japanese(&song.name);

        println!("{} : {}", &song.name, &romanji_title);

        let mut weighed_anime = anisong_db.find_songs_by_artists(&song).await.unwrap();

        println!("{}", weighed_anime.len());

        let mut found_by_artist = true;

        if weighed_anime.is_empty() {
            found_by_artist = false;

            let animes = anisong_db
                .get_animes_by_song_title(romanji_title.clone(), false)
                .await
                .unwrap();

            for anime in animes {
                let artists_string: String = anime
                    .artists
                    .iter()
                    .map(|artist| artist.names[0].clone()) // Extract first name
                    .intersperse(" ".to_string()) // Insert space between names
                    .collect();

                let spotify_artists: String = song
                    .artists
                    .iter()
                    .map(|artist| process_possible_japanese(&artist.name)) // Extract first name
                    .intersperse(" ".to_string()) // Insert space between names
                    .collect();

                let score =
                    fuzz::token_set_ratio(&spotify_artists, &artists_string, false, true) as f32;

                weighed_anime.push((anime, score));
            }
            println!("Search by song: {}", weighed_anime.len())
        }

        if weighed_anime.len() > 0 {
            let max_score = weighed_anime
                .iter()
                .map(|a| a.1)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

            if max_score > accuracy_cutoff {
                let (animehit, more_by_artists) = if found_by_artist {
                    let (animehit_evaluated, more_by_artists): (
                        Vec<(Anime, f32)>,
                        Vec<(Anime, f32)>,
                    ) = weighed_anime
                        .into_iter()
                        .partition(|anime| anime.1 == max_score);

                    if max_score > Self::ACCURACY_AUTOADD_LIMIT {
                        for anime in &animehit_evaluated {
                            let _ = self.try_add_anime_db(song, anime.0.clone()).await;
                        }
                        if animehit_evaluated[0].0.artists.len() == 1 && song.artists.len() == 1 {
                            let _ = self
                                .add_artist_db(
                                    &animehit_evaluated[0].0.artists[0],
                                    &song.artists[0].id,
                                )
                                .await;
                        }
                    }

                    let mut anime_hit_info_vec = FrontendAnimeEntry::from_anisongs(&animehit_evaluated.iter().map(|a| &a.0).collect()).await.unwrap();
                    let mut anime_more_by_artist_info_vec = FrontendAnimeEntry::from_anisongs(&more_by_artists.iter().map(|a| &a.0).collect()).await.unwrap();

                    anime_hit_info_vec.sort_by(|a, b| a.title.cmp(&b.title));
                    anime_more_by_artist_info_vec.sort_by(|a, b| a.title.cmp(&b.title));

                    (anime_hit_info_vec, anime_more_by_artist_info_vec)
                } else {
                    // found by song title
                    let anime_hit_info: Vec<(Anime, f32)> = weighed_anime
                        .into_iter()
                        .filter(|value| value.1 == max_score)
                        .collect();

                    if max_score > Self::ACCURACY_AUTOADD_LIMIT {
                        for anime in &anime_hit_info {
                            let _ = self.try_add_anime_db(song, anime.0.clone()).await;
                        }
                        if anime_hit_info[0].0.artists.len() == 1 && song.artists.len() == 1 {
                            let _ = self
                                .add_artist_db(&anime_hit_info[0].0.artists[0], &song.artists[0].id)
                                .await;
                        }
                    }

                    let mut anime_hit_info_vec = FrontendAnimeEntry::from_anisongs(&anime_hit_info.iter().map(|a| &a.0).collect()).await.unwrap();

                    let anime_more_by_artist_info = anisong_db
                        .get_animes_by_artists_ids(
                            anime_hit_info[0]
                                .0
                                .artists
                                .iter()
                                .map(|artist| artist.id.clone())
                                .collect(),
                        )
                        .await?;

                    let mut anime_more_by_artists_info_vec = FrontendAnimeEntry::from_anisongs(&anime_more_by_artist_info.iter().map(|a| a).collect()).await.unwrap();

                    anime_hit_info_vec.sort_by(|a, b| a.title.cmp(&b.title));
                    anime_more_by_artists_info_vec.sort_by(|a, b| a.title.cmp(&b.title));

                    (anime_hit_info_vec, anime_more_by_artists_info_vec)
                };

                let hit: SongHit = SongHit {
                    song_info: SongInfo::from_track_obj(song),
                    certainty: max_score as i32,
                    anime_info: animehit,
                    more_with_artist: more_by_artists,
                };

                return Ok(types::NewSong::Hit(hit));
            } else {

                let possible_anime = FrontendAnimeEntry::from_anisongs(&weighed_anime.iter().map(|a| &a.0).collect()).await.unwrap();

                let miss = SongMiss {
                    song_info: SongInfo::from_track_obj(song),
                    possible_anime,
                };

                return Ok(types::NewSong::Miss(miss));
            }
        } else {
            let possible_anime = anisong_db
                .get_animes_by_song_title(romanji_title.clone(), true)
                .await
                .unwrap();

            let found_anime = FrontendAnimeEntry::from_anisongs(&possible_anime.iter().map(|a| a).collect()).await.unwrap();
            
            let miss = SongMiss {
                song_info: SongInfo::from_track_obj(song),
                possible_anime: found_anime,
            };

            return Ok(types::NewSong::Miss(miss));
        }
    }
}
