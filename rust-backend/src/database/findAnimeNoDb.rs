use super::Database;
use crate::anisong::{Anime, AnisongClient};
use crate::japanese_processing::{process_possible_japanese, process_similarity};
use crate::spotify::responses::TrackObject;
use crate::types::{
    self, FrontendAnimeEntry as ReturnAnime, JikanAnime, JikanResponses, NewSong, SongHit,
    SongInfo, SongMiss,
};
use crate::{Error, Result};

use fuzzywuzzy::fuzz;
use itertools::Itertools;
use reqwest::Client;
use std::collections::HashSet;

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

                let mut spotify_artists: String = song
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

        weighed_anime.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if weighed_anime.len() > 0 {
            let max_score = weighed_anime[0].1;

            if max_score > accuracy_cutoff {
                let (animehit, more_by_artists) = if found_by_artist {
                    let (animehit_evaluated, more_by_artists): (
                        Vec<(Anime, f32)>,
                        Vec<(Anime, f32)>,
                    ) = weighed_anime
                        .into_iter()
                        .partition(|anime| anime.1 == max_score);
                    if max_score > 80.0 {
                        for anime in &animehit_evaluated {
                            let _ = self.try_add_anime_db(song, anime.0.clone()).await.unwrap();
                        }
                        if animehit_evaluated[0].0.artists.len() == 1 && song.artists.len() == 1 {
                            let _ = self.add_artist_db(&animehit_evaluated[0].0.artists[0], &song.artists[0].id).await;
                        }
                    }

                    let mut anime_hit_info_vec = Vec::new();
                    for anime in animehit_evaluated {
                        if anime.0.linked_ids.myanimelist.is_some() {
                            let extra_info = fetch_jikan(anime.0.linked_ids.myanimelist.unwrap())
                                .await
                                .map(|info| info.images.webp.image_url);
                            let anime_info = ReturnAnime::new(&anime.0, extra_info.ok()).unwrap();
                            anime_hit_info_vec.push(anime_info);
                        }
                    }

                    let mut anime_more_by_artist_info_vec = Vec::new();
                    for anime in more_by_artists {
                        if anime.0.linked_ids.myanimelist.is_some() {
                            let extra_info = fetch_jikan(anime.0.linked_ids.myanimelist.unwrap())
                                .await
                                .map(|info| info.images.webp.image_url);
                            let anime_info = ReturnAnime::new(&anime.0, extra_info.ok()).unwrap();
                            anime_more_by_artist_info_vec.push(anime_info);
                        }
                    }
                    (anime_hit_info_vec, anime_more_by_artist_info_vec)
                } else {
                    // found by song title
                    let anime_hit_info: Vec<(Anime, f32)> = weighed_anime
                        .into_iter()
                        .filter(|value| value.1 == max_score)
                        .collect();

                    if max_score > 80.0 {
                        for anime in &anime_hit_info {
                            let _ = self.try_add_anime_db(song, anime.0.clone()).await.unwrap();
                        }
                        if anime_hit_info[0].0.artists.len() == 1 && song.artists.len() == 1 {
                            let _ = self.add_artist_db(&anime_hit_info[0].0.artists[0], &song.artists[0].id).await;
                        }
                    }


                    let mut anime_hit_info_vec = Vec::new();
                    for anime in &anime_hit_info {
                        if anime.0.linked_ids.myanimelist.is_some() {
                            let extra_info = fetch_jikan(anime.0.linked_ids.myanimelist.unwrap())
                                .await
                                .map(|info| info.images.webp.image_url);
                            let anime_info = ReturnAnime::new(&anime.0, extra_info.ok()).unwrap();
                            anime_hit_info_vec.push(anime_info);
                        }
                    }

                    let mut anime_more_by_artist_info = anisong_db
                        .get_animes_by_artists_ids(
                            anime_hit_info[0]
                                .0
                                .artists
                                .iter()
                                .map(|artist| artist.id.clone())
                                .collect(),
                        )
                        .await?;
                    let mut anime_more_by_artists_info_vec = Vec::new();
                    for anime in anime_more_by_artist_info {
                        if anime.linked_ids.myanimelist.is_some() {
                            let extra_info = fetch_jikan(anime.linked_ids.myanimelist.unwrap())
                                .await
                                .map(|info| info.images.webp.image_url);
                            let anime_info = ReturnAnime::new(&anime, extra_info.ok()).unwrap();
                            anime_more_by_artists_info_vec.push(anime_info);
                        }
                    }

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
                let mut possible_anime = Vec::new();
                for anime in weighed_anime {
                    let extra_info = fetch_jikan(anime.0.linked_ids.myanimelist.unwrap())
                        .await
                        .map(|info| info.images.webp.image_url);
                    let anime_info = ReturnAnime::new(&anime.0, extra_info.ok()).unwrap();
                    possible_anime.push(anime_info);
                }

                let miss = SongMiss {
                    song_info: SongInfo {
                        title: song.name.clone(),
                        artists: song
                            .artists
                            .iter()
                            .map(|artist| artist.name.clone())
                            .collect(),
                        album_picture_url: song.album.images[0].url.clone(),
                    },
                    possible_anime,
                };

                return Ok(types::NewSong::Miss(miss));
            }
        } else {
            let possible_anime = anisong_db
                .get_animes_by_song_title(romanji_title.clone(), true)
                .await
                .unwrap();
            let mut found_anime = Vec::new();
            for anime in possible_anime {
                if anime.linked_ids.myanimelist.is_some() {
                    let extra_info = fetch_jikan(anime.linked_ids.myanimelist.unwrap())
                        .await
                        .map(|info| info.images.webp.image_url);
                    let anime_info = ReturnAnime::new(&anime, extra_info.ok()).unwrap();
                    found_anime.push(anime_info);
                }
            }

            let miss = SongMiss {
                song_info: SongInfo {
                    title: song.name.clone(),
                    artists: song
                        .artists
                        .iter()
                        .map(|artist| artist.name.clone())
                        .collect(),
                    album_picture_url: song.album.images[0].url.clone(),
                },
                possible_anime: found_anime,
            };

            return Ok(types::NewSong::Miss(miss));
        }
    }
}
