use crate::anisong::{Anime, AnisongClient};
use crate::japanese_processing::{process_possible_japanese, process_similarity};
use crate::spotify::responses::TrackObject;
use crate::types::{
    self, FrontendAnimeEntry as ReturnAnime, JikanAnime, JikanResponses, JikanSuccessResponse, NewSong, SongHit,
    SongInfo, SongMiss,
};
use crate::{Error, Result};

use fuzzywuzzy::fuzz;
use itertools::Itertools;
use reqwest::Client;
use std::collections::HashSet;
use std::sync::Arc;

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

pub async fn find_songs_by_artists(
    song: &TrackObject,
    anisong_db: &AnisongClient,
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

    let mut set = HashSet::with_capacity(anime_song_entries.len());

    let romanji_song_title = process_possible_japanese(&song.name);
    Ok(anime_song_entries
        .into_iter()
        .filter(|anime| {
            // if anime.annId.is_none() || anime.annSongId.is_none() {
            //     println!("Something that should not be None was None : (")
            // }
            set.insert((anime.annId, anime.annSongId))
        })
        .map(|anime| {
            let score = process_similarity(&romanji_song_title, &anime.songName);
            (anime, score)
        })
        .collect())
}

pub async fn find_most_likely_anime(
    song: &TrackObject,
    accuracy_cutoff: f32,
    anisong_db: &AnisongClient,
) -> Result<NewSong> {
    let romanji_title = process_possible_japanese(&song.name);

    println!("{} : {}", &song.name, &romanji_title);

    let mut weighed_anime = find_songs_by_artists(&song, &anisong_db)
        .await
        .unwrap();

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
                let (animehit, more_by_artists): (Vec<(Anime, f32)>, Vec<(Anime, f32)>) =
                    weighed_anime
                        .into_iter()
                        .partition(|anime| anime.1 == max_score);

                let mut anime_hit_info_vec = Vec::new();
                for anime in animehit {
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
                // Add to database
                (anime_hit_info_vec, anime_more_by_artist_info_vec)
            } else {
                let anime_hit_info: Vec<(Anime, f32)> = weighed_anime
                    .into_iter()
                    .filter(|value| value.1 == max_score)
                    .collect();
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
                anime_info: animehit,
                more_with_artist: more_by_artists,
            };

            return Ok(types::NewSong::Hit(hit));
        } else {
            let mut anime_hit_info_vec = Vec::new();
            for anime in weighed_anime {
                let extra_info = fetch_jikan(anime.0.linked_ids.myanimelist.unwrap())
                    .await
                    .map(|info| info.images.webp.image_url);
                let anime_info = ReturnAnime::new(&anime.0, extra_info.ok()).unwrap();
                anime_hit_info_vec.push(anime_info);
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
                possible_anime: anime_hit_info_vec,
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
