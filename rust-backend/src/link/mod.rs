use crate::anisong::{Anime, AnisongClient};
use crate::japanese_processing::{process_possible_japanese, process_similarity};
use crate::spotify::responses::{CurrentlyPlayingResponse, TrackObject};
use crate::types::{
    self, Anime as ReturnAnime, AnimeTrackIndex, AnimeType, JikanAnime, NewSong, SongHit, SongInfo,
    SongMiss,
};
use crate::{Error, Result, spotify};

use fuzzywuzzy::fuzz;
use itertools::Itertools;
use reqwest::Client;
use std::cmp::max;
use std::collections::HashSet;
use std::sync::Arc;

pub async fn find_songs_by_artists(
    song: &TrackObject,
    accuracy_cutoff: f32,
    anisong_db: Arc<AnisongClient>,
) -> Result<Vec<(Anime, f32)>> {
    let artists = &song.artists;
    let mut anime_song_entries = Vec::new();

    for artist in artists {
        let romanji_artist = process_possible_japanese(&artist.name);
        anime_song_entries.extend(anisong_db.get_animes_by_artist_name(romanji_artist).await?);
    }

    let romanji_song_title = process_possible_japanese(&song.name);
    Ok(anime_song_entries
        .into_iter()
        .map(|anime| {
            let score = process_similarity(&romanji_song_title, &anime.song_name);
            (anime, score)
        })
        .filter(|(_, score)| *score > accuracy_cutoff)
        .collect())
}

pub async fn find_most_likely_anime(
    song: &TrackObject,
    accuracy_cutoff: f32,
    anisong_db: Arc<AnisongClient>,
) -> Result<NewSong> {
    let romanji_title = process_possible_japanese(&song.name);

    let mut weighed_anime =
        find_songs_by_artists(&song, accuracy_cutoff, anisong_db.clone()).await?;

    if !weighed_anime.is_empty() {
        weighed_anime.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let max_score = weighed_anime[0].1;

        if max_score > accuracy_cutoff {
            let (animehit, more_by_artists): (Vec<(Anime, f32)>, Vec<(Anime, f32)>) = weighed_anime
                .into_iter()
                .partition(|anime| anime.1 == max_score);

            let mut anime_hit_info_vec = Vec::new();
            for anime in animehit {
                let extra_info: JikanAnime = Client::new()
                    .get(format!(
                        "https://api.jikan.moe/v4/anime/{}",
                        anime.0.linked_ids.myanimelist
                    ))
                    .send()
                    .await?
                    .json()
                    .await?;
                let anime_info = ReturnAnime::new(&anime.0, &extra_info)?;
                anime_hit_info_vec.push(anime_info);
                // Add to database
            }

            let mut anime_more_by_artist_info_vec = Vec::new();
            for anime in more_by_artists {
                let extra_info: JikanAnime = Client::new()
                    .get(format!(
                        "https://api.jikan.moe/v4/anime/{}",
                        anime.0.linked_ids.myanimelist
                    ))
                    .send()
                    .await?
                    .json()
                    .await?;

                let anime_info = ReturnAnime::new(&anime.0, &extra_info)?;
                anime_more_by_artist_info_vec.push(anime_info);

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
            let extra_info: JikanAnime = Client::new()
                .get(format!(
                    "https://api.jikan.moe/v4/anime/{}",
                    anime.0.linked_ids.myanimelist
                ))
                .send()
                .await?
                .json()
                .await?;

            let anime_info = types::Anime::new(&anime.0, &extra_info)?;
            found_anime.push(anime_info);

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
            .await?;
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
                let extra_info: JikanAnime = Client::new()
                    .get(format!(
                        "https://api.jikan.moe/v4/anime/{}",
                        anime.0.linked_ids.myanimelist
                    ))
                    .send()
                    .await?
                    .json()
                    .await?;
                let anime_info = ReturnAnime::new(&anime.0, &extra_info)?;
                anime_hit_info_vec.push(anime_info);
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
            let extra_info: JikanAnime = Client::new()
                .get(format!(
                    "https://api.jikan.moe/v4/anime/{}",
                    anime.0.linked_ids.myanimelist
                ))
                .send()
                .await?
                .json()
                .await?;

            let anime_info = types::Anime::new(&anime.0, &extra_info)?;
            found_anime.push(anime_info);
        }

        let miss = SongMiss {
            song_info: SongInfo {
                title: song.name.clone(),
                artists: song.artists.iter().map(|artist| artist.name.clone()).collect(),
                album_picture_url: song.album.images[0].url.clone(),
            },
            possible_anime: found_anime,
        };

        return Ok(types::NewSong::Miss(miss));
    }
}
