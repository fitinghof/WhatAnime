use crate::{Result, Error};
use crate::anisong::{Anime, AnisongClient, MalIdsSearchRequest};
use crate::japanese_processing::{process_possible_japanese, process_similarity};
use crate::spotify::responses::TrackObject;
use crate::types::{self, Anime as ReturnAnime, JikanAnime, JikanResponses, JikanSuccessResponse, NewSong, SongHit, SongInfo, SongMiss};

use axum::http::response;
use fuzzywuzzy::fuzz;
use itertools::Itertools;
use reqwest::Client;
use std::sync::Arc;

pub async fn fetch_jikan(mal_id: i32) -> Result<JikanAnime> {
    let response: JikanResponses = Client::new()
                    .get(format!(
                        "https://api.jikan.moe/v4/anime/{}",
                        mal_id
                    ))
                    .send()
                    .await?
                    .json()
                    .await.unwrap();

    match response {
        JikanResponses::Success(value) => Ok(value.data),
        JikanResponses::Fail(_) => Err(Error::BadRequest { url: "https://api.jikan.moe/v4/anime/".to_string(), status_code: axum::http::StatusCode::TOO_MANY_REQUESTS }),
    }
}

pub async fn find_songs_by_artists(
    song: &TrackObject,
    anisong_db: Arc<AnisongClient>,
) -> Result<Vec<(Anime, f32)>> {
    let artists = &song.artists;
    let mut anime_song_entries = Vec::new();

    for artist in artists {
        let romanji_artist = process_possible_japanese(&artist.name);
        let songs = anisong_db
            .get_animes_by_artist_name(romanji_artist)
            .await
            .unwrap();
        anime_song_entries.extend(songs);
    }

    let romanji_song_title = process_possible_japanese(&song.name);
    Ok(anime_song_entries
        .into_iter()
        .map(|anime| {
            let score = process_similarity(&romanji_song_title, &anime.songName);
            (anime, score)
        })
        .collect())
}

pub async fn find_most_likely_anime(
    song: &TrackObject,
    accuracy_cutoff: f32,
    anisong_db: Arc<AnisongClient>,
) -> Result<NewSong> {
    let romanji_title = process_possible_japanese(&song.name);

    let mut weighed_anime =
        find_songs_by_artists(&song, anisong_db.clone()).await.unwrap();


    println!("{}", weighed_anime.len());

    if !weighed_anime.is_empty() {
        weighed_anime.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let max_score = weighed_anime[0].1;


        if max_score > accuracy_cutoff {
            let (animehit, more_by_artists): (Vec<(Anime, f32)>, Vec<(Anime, f32)>) = weighed_anime
                .into_iter()
                .partition(|anime| anime.1 == max_score);

            let mut anime_hit_info_vec = Vec::new();
            for anime in animehit {
                if anime.0.linked_ids.myanimelist.is_some() {
                    let extra_info = fetch_jikan(anime.0.linked_ids.myanimelist.unwrap()).await;
                    if extra_info.is_ok() {
                        let anime_info = ReturnAnime::new(&anime.0, &extra_info.unwrap().images.webp.image_url)?;
                        anime_hit_info_vec.push(anime_info);
                    }
                }
            }


            let mut anime_more_by_artist_info_vec = Vec::new();
            for anime in more_by_artists {
                if anime.0.linked_ids.myanimelist.is_some() {
                    let extra_info = fetch_jikan(anime.0.linked_ids.myanimelist.unwrap()).await.map(|info| info.images.webp.image_url)
                    .unwrap_or_else(|_| String::new());
                    let anime_info = ReturnAnime::new(&anime.0, &extra_info)?;
                    anime_more_by_artist_info_vec.push(anime_info);
                }

                // Add to database
            }

            let hit: SongHit = SongHit {
                song_info: SongInfo {
                    title: song.name.clone(),
                    artists: song
                        .artists
                        .iter()
                        .map(|artist| artist.name.clone())
                        .collect(),
                    album_picture_url: song.album.images[0].url.clone(),
                },
                certainty: max_score as i32,
                anime_info: anime_hit_info_vec,
                more_with_artist: anime_more_by_artist_info_vec,
            };

            return Ok(types::NewSong::Hit(hit));
        }

        let mut found_anime = Vec::new();
        for anime in weighed_anime {
            if anime.0.linked_ids.myanimelist.is_some() {
                let extra_info = fetch_jikan(anime.0.linked_ids.myanimelist.unwrap()).await.map(|info| info.images.webp.image_url)
                .unwrap_or_else(|_| String::new());
                let anime_info = ReturnAnime::new(&anime.0, &extra_info)?;
                found_anime.push(anime_info);
            }

            // Add to database
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
    } else {
        let animes = anisong_db
            .get_animes_by_song_title(romanji_title.clone())
            .await.unwrap();
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
                .map(|artist| artist.name.clone()) // Extract first name
                .intersperse(" ".to_string()) // Insert space between names
                .collect();

            spotify_artists = process_possible_japanese(&spotify_artists);

            let score =
                fuzz::token_set_ratio(&spotify_artists, &artists_string, false, true) as f32;

            weighed_anime.push((anime, score));
        }

        if weighed_anime.len() == 0 {
            return Ok(NewSong::Miss(SongMiss {
                song_info: SongInfo {
                    title: song.name.clone(),
                    artists: song
                        .artists
                        .iter()
                        .map(|artist| artist.name.clone())
                        .collect(),
                    album_picture_url: song.album.images[0].url.clone(),
                },
                possible_anime: vec![],
            }));
        }

        weighed_anime.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let max_score = weighed_anime[0].1;

        if max_score > accuracy_cutoff {
            weighed_anime.retain(|anime| anime.1 == max_score);

            let mut anime_hit_info_vec = Vec::new();
            for anime in weighed_anime {
                if anime.0.linked_ids.myanimelist.is_some() {
                    let extra_info = fetch_jikan(anime.0.linked_ids.myanimelist.unwrap()).await.map(|info| info.images.webp.image_url)
                    .unwrap_or_else(|_| String::new());
                    let anime_info = ReturnAnime::new(&anime.0, &extra_info)?;
                    anime_hit_info_vec.push(anime_info);
                }
                // Add to database
            }

            // TODO load artists, filter by song already in the hit vec, and pass to song_info

            // let more_by_artists = anisong_db.get_animes_by_artist_name(artist)
            // let mut anime_more_by_artist_info_vec = Vec::new();
            // for anime in more_by_artists {
            //     let extra_info: JikanAnime = Client::new()
            //     .get(format!("https://api.jikan.moe/v4/anime/{}", anime.0.linked_ids.myanimelist))
            //     .send()
            //     .await?.json().await?;

            //     let anime_info = ReturnAnime::new(&anime.0, &extra_info)?;
            //     anime_more_by_artist_info_vec.push(anime_info);

            //     // Add to database
            // }

            let hit: SongHit = SongHit {
                song_info: SongInfo {
                    title: song.name.clone(),
                    artists: song
                        .artists
                        .iter()
                        .map(|artist| artist.name.clone())
                        .collect(),
                    album_picture_url: song.album.images[0].url.clone(),
                },
                certainty: max_score as i32,
                anime_info: anime_hit_info_vec,
                more_with_artist: vec![],
            };

            return Ok(types::NewSong::Hit(hit));
        }

        let mut found_anime = Vec::new();
        for anime in weighed_anime {
            if anime.0.linked_ids.myanimelist.is_some() {
                let extra_info = fetch_jikan(anime.0.linked_ids.myanimelist.unwrap()).await.map(|info| info.images.webp.image_url)
                .unwrap_or_else(|_| String::new());
                let anime_info = ReturnAnime::new(&anime.0, &extra_info)?;
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
