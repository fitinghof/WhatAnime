pub mod databasetypes;
pub mod find_anime_no_db;
pub mod regex_search;

use crate::anilist::Media;
use crate::anilist::types::{AnilistID, TagID, URL};
use crate::anisong::{Anime, AnisongClient, Artist, ArtistIDSearchRequest};
use crate::japanese_processing::process_similarity;
use crate::spotify::responses::{SimplifiedArtist, TrackObject};
use crate::types::{
    AnimeIndex, AnimeTrackIndex, AnimeType, FrontendAnimeEntry, NewSong, SongHit, SongInfo,
    SongMiss,
};
use crate::{Error, Result};
use axum_sessions::async_session::chrono::{Duration, Utc};
use databasetypes::{DBAnime, DBArtist, SongGroup, SongGroupLink};
use itertools::Itertools;
use regex_search::create_artist_regex;
use reqwest::StatusCode;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Transaction};
use std::cmp::max;
use std::collections::HashSet;
use std::env;

pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    const ACCURACY_AUTOADD_LIMIT: f32 = 80.0;
    const UPDATE_TIME: Duration = Duration::days(7);
    // A bound function to initialize the Database. You can call this once on startup.
    pub async fn new() -> Self {
        // Ensure the DATABASE_URL environment variable is set.
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set.");

        //println!("{}", &database_url);

        // Create the connection pool.
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create the pool");

        Database { pool }
    }

    // Example of a method that could run migrations, query for data, etc.
    pub async fn run_migrations(&self) -> Result<()> {
        // Using sqlx::migrate! macro to run migrations located in "migrations" folder.
        sqlx::migrate!("src\\database\\migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_artist(&self, spotify_id: &String) -> Result<Option<DBArtist>> {
        Ok(
            sqlx::query_as::<Postgres, DBArtist>("SELECT * FROM artists WHERE spotify_id = $1")
                .bind(&spotify_id)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    async fn get_artists_spotify_id(&self, spotify_ids: &Vec<String>) -> Result<Vec<DBArtist>> {
        Ok(
            sqlx::query_as::<Postgres, DBArtist>(
                "SELECT * FROM artists WHERE spotify_id = ANY($1)",
            )
            .bind(&spotify_ids)
            .fetch_all(&self.pool)
            .await?,
        )
    }

    async fn get_artists_ann_id(&self, ann_id: &Vec<i32>) -> Result<Vec<DBArtist>> {
        Ok(
            sqlx::query_as::<Postgres, DBArtist>("SELECT * FROM artists WHERE ann_id = ANY($1)")
                .bind(&ann_id)
                .fetch_all(&self.pool)
                .await?,
        )
    }

    async fn get_animes_by_annids(&self, ann_ids: &Vec<i32>) -> Result<Vec<DBAnime>> {
        Ok(
            sqlx::query_as::<Postgres, DBAnime>("SELECT * FROM animes WHERE ann_id = ANY($1)")
                .bind(&ann_ids)
                .fetch_all(&self.pool)
                .await?,
        )
    }

    async fn get_anime_by_spotify_id(&self, spotify_id: &String) -> Result<Vec<DBAnime>> {
        let group_id = sqlx::query_as!(
            SongGroupLink,
            "SELECT * FROM song_group_links WHERE spotify_id = $1",
            spotify_id
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap();
        if group_id.is_some() {
            Ok(
                sqlx::query_as::<Postgres, DBAnime>(
                    "SELECT * FROM animes WHERE song_group_id = $1",
                )
                .bind(&group_id.unwrap().group_id)
                .fetch_all(&self.pool)
                .await?,
            )
        } else {
            Ok(vec![])
        }
    }

    async fn get_animes_by_artists_ann_ids(&self, ann_ids: &Vec<i32>) -> Result<Vec<DBAnime>> {
        Ok(
            sqlx::query_as::<Postgres, DBAnime>("SELECT * FROM animes WHERE artists_ann_id && $1")
                .bind(&ann_ids)
                .fetch_all(&self.pool)
                .await
                .unwrap(),
        )
    }

    async fn get_animes_by_artists_ids(&self, spotify_ids: &Vec<String>) -> Result<Vec<DBAnime>> {
        // Might want to use the jointable, but that would require starting to fill the join table aswell.
        let artists = sqlx::query_as!(
            DBArtist,
            "SELECT * FROM artists WHERE spotify_id = ANY($1)",
            spotify_ids
        )
        .fetch_all(&self.pool)
        .await
        .unwrap();

        let ann_ids: Vec<i32> = artists.into_iter().map(|a| a.ann_id).collect();

        self.get_animes_by_artists_ann_ids(&ann_ids).await
    }

    pub async fn update_or_add_animes(
        &self,
        animes: Vec<&DBAnime>,
        from_user: Option<String>,
        from_user_mail: Option<String>,
    ) {
        println!("Trying to add or update {} animes", animes.len());
        let mut tx = self.pool.begin().await.unwrap();
        for anime in animes {
            let _ = sqlx::query::<Postgres>(
                r#"
                INSERT INTO animes (
                    ann_id, title_eng, title_jpn, index_type, index_number, anime_type, episodes, mean_score,
                    banner_image, cover_image_color, cover_image_medium, cover_image_large, cover_image_extra_large,
                    media_format, genres, source, studio_ids, studio_names, studio_urls, tag_ids, tag_names, trailer_id,
                    trailer_site, thumbnail, release_season, release_year,
                    ann_song_id, song_name, spotify_artist_ids, artist_names, artists_ann_id, composers_ann_id,
                    arrangers_ann_id, track_index_type, track_index_number, mal_id, anilist_id, anidb_id, kitsu_id, 
                    song_group_id, from_user_name, from_user_mail
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31,
                    $32, $33, $34, $35, $36, $37, $38, $39, $40, $41, $42
                )
                ON CONFLICT (ann_song_id) DO UPDATE SET 
                episodes = COALESCE(EXCLUDED.episodes, animes.episodes),
                mean_score = COALESCE(EXCLUDED.mean_score, animes.mean_score),
                banner_image = COALESCE(EXCLUDED.banner_image, animes.banner_image),
                cover_image_color = COALESCE(EXCLUDED.cover_image_color, animes.cover_image_color),
                cover_image_medium = COALESCE(EXCLUDED.cover_image_medium, animes.cover_image_medium),
                cover_image_large = COALESCE(EXCLUDED.cover_image_large, animes.cover_image_large),
                cover_image_extra_large = COALESCE(EXCLUDED.cover_image_extra_large, animes.cover_image_extra_large),
                media_format = COALESCE(EXCLUDED.media_format, animes.media_format),
                genres = COALESCE(EXCLUDED.genres, animes.genres),
                source = COALESCE(EXCLUDED.source, animes.source),
                studio_ids = COALESCE(EXCLUDED.studio_ids, animes.studio_ids),
                studio_names = COALESCE(EXCLUDED.studio_names, animes.studio_names),
                studio_urls = COALESCE(EXCLUDED.studio_urls, animes.studio_urls),
                tag_ids = COALESCE(EXCLUDED.tag_ids, animes.tag_ids),
                tag_names = COALESCE(EXCLUDED.tag_names, animes.tag_names),
                trailer_id = COALESCE(EXCLUDED.trailer_id, animes.trailer_id),
                trailer_site = COALESCE(EXCLUDED.trailer_site, animes.trailer_site),
                thumbnail = COALESCE(EXCLUDED.thumbnail, animes.thumbnail),
                release_season = COALESCE(EXCLUDED.release_season, animes.release_season),
                release_year = COALESCE(EXCLUDED.release_year, animes.release_year),
                song_group_id = COALESCE(EXCLUDED.song_group_id, animes.song_group_id)
            "#)
            .bind(&anime.ann_id)
            .bind(&anime.title_eng)
            .bind(&anime.title_jpn)
            .bind(&anime.index_type)
            .bind(&anime.index_number)
            .bind(&anime.anime_type)
            .bind(&anime.episodes)
            .bind(&anime.mean_score)
            .bind(&anime.banner_image)
            .bind(&anime.cover_image_color)
            .bind(&anime.cover_image_medium)
            .bind(&anime.cover_image_large)
            .bind(&anime.cover_image_extra_large)
            .bind(&anime.media_format)
            .bind(&anime.genres)
            .bind(&anime.source)
            .bind(&anime.studio_ids)
            .bind(&anime.studio_names)
            .bind(&anime.studio_urls)
            .bind(&anime.tag_ids)
            .bind(&anime.tag_names)
            .bind(&anime.trailer_id)
            .bind(&anime.trailer_site)
            .bind(&anime.thumbnail)
            .bind(&anime.release_season)
            .bind(&anime.release_year)
            .bind(&anime.ann_song_id)
            .bind(&anime.song_name)
            .bind(&anime.spotify_artist_ids)
            .bind(&anime.artist_names)
            .bind(&anime.artists_ann_id)
            .bind(&anime.composers_ann_id)
            .bind(&anime.arrangers_ann_id)
            .bind(&anime.track_index_type)
            .bind(&anime.track_index_number)
            .bind(&anime.mal_id)
            .bind(&anime.anilist_id)
            .bind(&anime.anidb_id)
            .bind(&anime.kitsu_id)
            .bind(&anime.song_group_id)
            .bind(&from_user)
            .bind(&from_user_mail)
            .execute(&mut *tx).await.unwrap();
        }
        tx.commit().await.unwrap();
    }

    pub async fn add_anime(
        &self,
        spotify_track_object: &TrackObject,
        anisong_anime: Anime,
        info: Media,
        from_user_name: Option<String>,
        from_user_mail: Option<String>,
    ) -> Result<()> {
        let group_id = self
            .add_song_group_link(
                &spotify_track_object.id,
                &anisong_anime.songName,
                &anisong_anime.artists.iter().map(|a| a.id.clone()).collect(),
            )
            .await;
        let db_anime =
            DBAnime::from_anisong_and_anilist(&anisong_anime, Some(&info), Some(group_id));

        self.update_or_add_animes(vec![&db_anime], from_user_name, from_user_mail)
            .await;
        Ok(())
    }

    pub async fn add_artist_db(&self, artist: &Artist, artist_spotify_id: &String) {
        let groups_ids = artist
            .groups
            .as_ref()
            .map(|groups| groups.iter().map(|b| b.id).collect::<Vec<i32>>());
        // Collect members' ids into a Vec<i32> and convert to an Option<&[i32]>
        let members_ids = artist
            .members
            .as_ref()
            .map(|members| members.iter().map(|b| b.id).collect::<Vec<i32>>());
        let _ = sqlx::query::<Postgres>(
            r#"INSERT INTO artists (
                spotify_id,
                ann_id,
                names,
                groups_ids,
                members
            )
            VALUES (
            $1, $2, $3, $4, $5
            )
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(artist_spotify_id.clone())
        .bind(artist.id)
        .bind(&artist.names)
        .bind(groups_ids.as_deref())
        .bind(members_ids.as_deref())
        .execute(&self.pool)
        .await
        .unwrap();
    }

    pub async fn add_song_group_link(
        &self,
        spotify_id: &String,
        song_title: &String,
        artist_ids: &Vec<i32>,
    ) -> i32 {
        let song_link = sqlx::query_as!(
            SongGroupLink,
            "SELECT * FROM song_group_links WHERE spotify_id = $1",
            spotify_id
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap();
        if song_link.is_none() {
            let song_group = sqlx::query_as!(
                SongGroup,
                "SELECT * FROM song_groups WHERE song_title = $1 AND artist_ids = $2",
                song_title,
                artist_ids,
            )
            .fetch_optional(&self.pool)
            .await
            .unwrap();
            let group_id = if song_group.is_some() {
                song_group.unwrap().group_id
            } else {
                let group_id = sqlx::query!(
                    "INSERT INTO song_groups (song_title, artist_ids) VALUES ($1, $2) ON CONFLICT DO NOTHING RETURNING group_id",
                    song_title,
                    artist_ids
                ).fetch_one(&self.pool).await.unwrap();
                group_id.group_id
            };
            let _ = sqlx::query!(
                "INSERT INTO song_group_links (spotify_id, group_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                spotify_id,
                group_id
            ).execute(&self.pool).await;
            group_id
        } else {
            song_link.unwrap().group_id
        }
    }

    pub async fn try_add_anime_db(
        &self,
        spotify_track_object: &TrackObject,
        anisong_anime: Anime,
    ) -> Result<()> {
        return self
            .try_add_anime(
                spotify_track_object,
                anisong_anime,
                Some("Database".to_string()),
                None,
            )
            .await;
    }

    pub async fn try_add_anime_user(
        &self,
        spotify_track_object: &TrackObject,
        anisong_anime: Anime,
        from_user_name: Option<String>,
        from_user_mail: Option<String>,
    ) -> Result<()> {
        println!(
            "User {:?}, mail: {:?}, added bind for {}\nSong: {}  ---  {}\nby: {:?}  ---  {:?}",
            &from_user_name,
            &from_user_mail,
            &anisong_anime.animeENName,
            &spotify_track_object.name,
            &anisong_anime.songName,
            &spotify_track_object
                .artists
                .iter()
                .map(|a| &a.name)
                .collect::<Vec<&String>>(),
            anisong_anime
                .artists
                .iter()
                .map(|a| &a.names[0])
                .collect::<Vec<&String>>(),
        );
        return self
            .try_add_anime(
                spotify_track_object,
                anisong_anime,
                from_user_name,
                from_user_mail,
            )
            .await;
    }

    pub async fn try_add_anime(
        &self,
        spotify_track_object: &TrackObject,
        anisong_anime: Anime,
        from_user_name: Option<String>,
        from_user_mail: Option<String>,
    ) -> Result<()> {
        match anisong_anime.linked_ids.anilist {
            Some(id) => match Media::fetch_one(id).await {
                Some(info) => {
                    self.add_anime(
                        spotify_track_object,
                        anisong_anime,
                        info,
                        from_user_name,
                        from_user_mail,
                    )
                    .await
                    .unwrap();
                    Ok(())
                }
                None => {
                    println!(
                        "Failed to fetch anilist info for {}",
                        anisong_anime.animeENName
                    );
                    Err(Error::BadRequest {
                        url: "Anilist and whatever their URL is".to_string(),
                        status_code: StatusCode::BAD_REQUEST,
                    })
                }
            },
            None => Ok(()),
        }
    }

    pub async fn db_full_search(
        &self,
        track: &TrackObject,
    ) -> Result<(Vec<DBAnime>, Vec<DBAnime>, Vec<i32>, Vec<DBArtist>, f32)> {
        let anime = self.get_anime_by_spotify_id(&track.id).await.unwrap();
        let artists = self
            .get_artists_spotify_id(&track.artists.iter().map(|a| a.id.clone()).collect())
            .await
            .unwrap();

        let artist_ann_ids = if anime.len() > 0 {
            anime[0].artists_ann_id.clone()
        } else if artists.len() > 0 {
            artists.iter().map(|a| a.ann_id).collect()
        } else {
            let artists = sqlx::query_as::<Postgres, DBArtist>(
                "SELECT * FROM public.artists WHERE EXISTS (
                    SELECT 1
                    FROM unnest(names) AS name WHERE name ~* $1);",
            )
            .bind(create_artist_regex(
                track.artists.iter().map(|a| &a.name).collect(),
            ))
            .fetch_all(&self.pool)
            .await
            .unwrap();

            artists.iter().map(|a| a.ann_id).collect()
        };

        let more_by_artists = self
            .get_animes_by_artists_ann_ids(&artist_ann_ids)
            .await
            .unwrap();

        if anime.len() > 0 {
            Ok((anime, more_by_artists, artist_ann_ids, artists, 100.0))
        } else if more_by_artists.len() > 0 {
            let (best_match, certainty) =
                DBAnime::pick_best_by_song_name(&more_by_artists, &track.name).unwrap();

            if certainty > Self::ACCURACY_AUTOADD_LIMIT {
                self.add_song_group_link(
                    &track.id,
                    &best_match[0].song_name,
                    &best_match[0].artists_ann_id,
                )
                .await;
            }
            Ok((
                best_match.into_iter().map(|a| a.clone()).collect(),
                more_by_artists,
                artist_ann_ids,
                artists,
                certainty,
            ))
        } else {
            Ok((vec![], vec![], artist_ann_ids, artists, 0.0))
        }
    }

    pub async fn try_add_artists(
        &self,
        already_existing_artists: &Vec<DBArtist>,
        mut anisong_artists: Vec<&Artist>,
        mut spotify_artists: Vec<&SimplifiedArtist>,
    ) {
        if anisong_artists.len() != spotify_artists.len() {
            return;
        }

        for dbartist in already_existing_artists {
            anisong_artists.retain(|a| a.id != dbartist.ann_id);
            spotify_artists.retain(|a| a.id != dbartist.spotify_id);
        }

        if anisong_artists.len() != spotify_artists.len() {
            return;
        }

        let mut artists_to_add = Vec::new();

        for artist in anisong_artists {
            let mut eval_spotify: Vec<(&SimplifiedArtist, f32)> = spotify_artists
                .iter()
                .map(|&a| {
                    let max_score = artist
                        .names
                        .iter()
                        .map(|an| process_similarity(&a.name, &an))
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();
                    (a, max_score)
                })
                .collect();

            eval_spotify.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            if !eval_spotify.is_empty() && eval_spotify[0].1 > Self::ACCURACY_AUTOADD_LIMIT {
                let new_artist = DBArtist {
                    spotify_id: eval_spotify[0].0.id.clone(),
                    ann_id: artist.id,
                    names: artist.names.clone(),
                    groups_ids: artist
                        .groups
                        .as_ref()
                        .map(|g| g.iter().map(|g2| g2.id).collect()),
                    members: artist
                        .members
                        .as_ref()
                        .map(|a| a.iter().map(|m| m.id).collect()),
                };
                artists_to_add.push(new_artist);
            }
        }
        let mut tx = self.pool.begin().await.unwrap();

        for new_artist in artists_to_add {
            let _ = sqlx::query!(
                r#"INSERT INTO artists (spotify_id, ann_id, names, groups_ids, members)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (spotify_id) DO NOTHING"#,
                &new_artist.spotify_id,
                &new_artist.ann_id,
                &new_artist.names,
                &new_artist.groups_ids.unwrap_or(vec![]),
                &new_artist.members.unwrap_or(vec![]),
            )
            .execute(&mut *tx)
            .await
            .unwrap();
        }

        tx.commit().await.unwrap();
    }

    pub async fn get_anime_2(
        &self,
        track: &TrackObject,
        anisong_db: &AnisongClient,
        accuracy_cutoff: f32,
    ) -> Result<NewSong> {
        let (mut hit_anime, mut more_by_artists, artists_ann_id, artists_searched, certainty) =
            self.db_full_search(track).await.unwrap();

        if certainty == 100.0 {
            let mut anisong_animes = anisong_db
                .get_animes_by_artists_ids(artists_ann_id)
                .await
                .unwrap();

            let max_unique = max(
                hit_anime.len() + more_by_artists.len(),
                anisong_animes.len(),
            );

            let mut anime_set = HashSet::with_capacity(max_unique);

            // filter out duplicates
            hit_anime.retain(|a| anime_set.insert(a.ann_song_id));
            more_by_artists.retain(|a| anime_set.insert(a.ann_song_id));
            anisong_animes.retain(|a| anime_set.insert(a.annSongId));

            // Get anilist ids for animes that need updates or extra info
            let mut anilist_ids: Vec<AnilistID> = more_by_artists
                .iter()
                .filter(|a| a.last_updated + Self::UPDATE_TIME < Utc::now())
                .filter_map(|a| a.anilist_id)
                .collect();

            // Same
            anilist_ids.extend(
                hit_anime
                    .iter()
                    .filter(|a| a.last_updated + Self::UPDATE_TIME < Utc::now())
                    .filter_map(|a| a.anilist_id),
            );

            // split anisongs into hits and misses and add more ids
            let (mut anisong_anime_hits, mut anisong_anime_more): (Vec<Anime>, Vec<Anime>) =
                anisong_animes.into_iter().partition(|a| {
                    if a.linked_ids.anilist.is_some() {
                        anilist_ids.push(a.linked_ids.anilist.unwrap());
                    }

                    a.artists.iter().map(|a| a.id).collect::<Vec<i32>>()
                        == hit_anime[0].artists_ann_id
                        && a.songName == hit_anime[0].song_name
                });

            // Fetch extra Anilist media
            let anilists = Media::fetch_many(anilist_ids).await.unwrap();

            // get group id.
            let group_id = self
                .add_song_group_link(
                    &track.id,
                    &hit_anime[0].song_name,
                    &hit_anime[0].artists_ann_id,
                )
                .await;

            // update arrays to contain new info
            DBAnime::update_all(&mut hit_anime, &anilists, Some(group_id));
            DBAnime::update_all(&mut more_by_artists, &anilists, None);

            // seperate out the updateted info to send to database
            let mut update_animes: Vec<&DBAnime> = more_by_artists
                .iter()
                .filter(|a| {
                    a.last_updated + Self::UPDATE_TIME < Utc::now() && a.anilist_id.is_some()
                })
                .collect();

            // Since we now garantee song_group id is Some we sorta cannot decide if it is neccessary to update it or not
            // Leading to having to update all hitanime, might not be the best idea, might also not be that bad.
            update_animes.extend(hit_anime.iter());

            let anisong_hit =
                DBAnime::from_anisongs_and_anilists(&anisong_anime_hits, &anilists, Some(group_id))
                    .unwrap();

            let anisong_more_by_artists =
                DBAnime::from_anisongs_and_anilists(&anisong_anime_more, &anilists, None).unwrap();

            // extend with stuff that is new and need to be added.
            update_animes.extend(anisong_hit.iter());
            update_animes.extend(anisong_more_by_artists.iter());

            // send to database for updates
            self.update_or_add_animes(update_animes, Some("Database".to_string()), None)
                .await;

            // combine stuff from database and stuff from anisongdb
            hit_anime.extend(anisong_hit);
            more_by_artists.extend(anisong_more_by_artists);

            more_by_artists.sort_by(|a, b| a.title_eng.cmp(&b.title_eng));
            hit_anime.sort_by(|a, b| a.title_eng.cmp(&b.title_eng));

            // Return result
            Ok(NewSong::Hit(SongHit {
                song_info: SongInfo::from_track_obj(track),
                certainty: certainty as i32,
                anime_info: hit_anime
                    .iter()
                    .map(|a| FrontendAnimeEntry::from_db(a))
                    .collect(),
                more_with_artist: more_by_artists
                    .iter()
                    .map(|a| FrontendAnimeEntry::from_db(a))
                    .collect(),
            }))
        } else {
            if artists_ann_id.len() > 0 {
                let mut anisongs = anisong_db
                    .get_animes_by_artists_ids(artists_ann_id.clone())
                    .await
                    .unwrap();
                let (best, score) =
                    AnisongClient::pick_best_by_song_name(&anisongs, &track.name).unwrap();

                // Add constant for acceptable match
                if score > accuracy_cutoff {
                    // get data for the best song
                    let best_song_name = best[0].songName.clone();

                    let best_artist_ids: Vec<i32> = best[0].artists.iter().map(|a| a.id).collect();

                    let mut artist_set = HashSet::with_capacity(best[0].artists.len());

                    artists_ann_id.iter().for_each(|id| {
                        artist_set.insert(*id);
                    });

                    // Since we were not certain that we got the correct anime from our database,
                    // check if there are any missing artists we need to fetch
                    let missing_artists: Vec<i32> = best_artist_ids
                        .iter()
                        .filter(|&&id| artist_set.insert(id))
                        .cloned()
                        .collect();

                    // fetch those artists animes
                    if !missing_artists.is_empty() {
                        let additional_anisongs = anisong_db
                            .get_animes_by_artists_ids(missing_artists)
                            .await
                            .unwrap();
                        anisongs.extend(additional_anisongs);
                    }

                    // Filter out duplicates
                    let mut anime_set = HashSet::with_capacity(anisongs.len());

                    let (mut anisong_anime_hits, mut anisong_anime_more): (Vec<Anime>, Vec<Anime>) =
                        anisongs.into_iter().partition(|a| {
                            a.artists.iter().map(|a| a.id).collect::<Vec<i32>>() == artists_ann_id
                                && a.songName == best_song_name
                        });

                    anisong_anime_hits.retain(|a| anime_set.insert(a.annSongId));
                    more_by_artists.retain(|a| anime_set.insert(a.ann_song_id));
                    anisong_anime_more.retain(|a| anime_set.insert(a.annSongId));

                    // Gather anilist ids for entries that need updates or adds
                    let mut anilist_ids: Vec<AnilistID> = anisong_anime_more
                        .iter()
                        .filter_map(|a| a.linked_ids.anilist)
                        .collect();
                    anilist_ids.extend(
                        more_by_artists
                            .iter()
                            .filter(|a| a.last_updated + Self::UPDATE_TIME < Utc::now())
                            .filter_map(|a| a.anilist_id),
                    );
                    anilist_ids.extend(
                        anisong_anime_hits
                            .iter()
                            .filter_map(|a| a.linked_ids.anilist),
                    );

                    // Fetch new data
                    let media = Media::fetch_many(anilist_ids).await.unwrap();

                    // Update data that we want to send to both frontend and database
                    DBAnime::update_all(&mut more_by_artists, &media, None);

                    // Try and add more artists to the database
                    if score > Self::ACCURACY_AUTOADD_LIMIT {
                        self.try_add_artists(
                            &artists_searched,
                            anisong_anime_hits[0].artists.iter().collect(),
                            track.artists.iter().collect(),
                        )
                        .await;
                    }

                    let group_id = if score > Self::ACCURACY_AUTOADD_LIMIT {
                        Some(
                            self.add_song_group_link(
                                &track.id,
                                &anisong_anime_hits[0].songName,
                                &anisong_anime_hits[0].artists.iter().map(|a| a.id).collect(),
                            )
                            .await,
                        )
                    } else {
                        None
                    };

                    // Promote anisong Anime to DBAnime using our gathered media
                    let mut anime_hit =
                        DBAnime::from_anisongs_and_anilists(&anisong_anime_hits, &media, group_id)
                            .unwrap();

                    let more_by_artists_anisong =
                        DBAnime::from_anisongs_and_anilists(&anisong_anime_more, &media, None)
                            .unwrap();

                    // send only stuff that should be updated to the database
                    let mut updates_and_adds: Vec<&DBAnime> = anime_hit.iter().collect();
                    updates_and_adds.extend(more_by_artists_anisong.iter());
                    updates_and_adds.extend(more_by_artists.iter().filter(|a| {
                        a.last_updated + Self::UPDATE_TIME < Utc::now() && a.anilist_id.is_some()
                    }));

                    self.update_or_add_animes(updates_and_adds, Some("Database".to_string()), None)
                        .await;

                    more_by_artists.sort_by(|a, b| a.title_eng.cmp(&b.title_eng));
                    anime_hit.sort_by(|a, b| a.title_eng.cmp(&b.title_eng));

                    // return Hit
                    Ok(NewSong::Hit(SongHit {
                        song_info: SongInfo::from_track_obj(track),
                        certainty: score as i32,
                        anime_info: anime_hit
                            .iter()
                            .map(|a| FrontendAnimeEntry::from_db(a))
                            .collect(),
                        more_with_artist: more_by_artists
                            .iter()
                            .map(|a| FrontendAnimeEntry::from_db(a))
                            .collect(),
                    }))
                } else {
                    // let best_score = max(certainty, score);

                    let mut anime_set = HashSet::with_capacity(anisongs.len());

                    more_by_artists.retain(|a| anime_set.insert(a.ann_song_id));
                    anisongs.retain(|a| anime_set.insert(a.annSongId));

                    let mut anilist_ids: Vec<AnilistID> = anisongs
                        .iter()
                        .filter_map(|a| a.linked_ids.anilist)
                        .collect();

                    anilist_ids.extend(
                        more_by_artists
                            .iter()
                            .filter(|a| a.last_updated + Self::UPDATE_TIME < Utc::now())
                            .filter_map(|a| a.anilist_id),
                    );

                    let media = Media::fetch_many(anilist_ids).await.unwrap();

                    DBAnime::update_all(&mut more_by_artists, &media, None);

                    let extra =
                        DBAnime::from_anisongs_and_anilists(&anisongs, &media, None).unwrap();

                    let mut updates_or_adds: Vec<&DBAnime> = more_by_artists
                        .iter()
                        .filter(|a| {
                            a.last_updated + Self::UPDATE_TIME < Utc::now()
                                && a.anilist_id.is_some()
                        })
                        .collect();

                    updates_or_adds.extend(extra.iter());

                    self.update_or_add_animes(updates_or_adds, Some("Database".to_string()), None)
                        .await;

                    more_by_artists.extend(extra);

                    Ok(NewSong::Miss(SongMiss {
                        song_info: SongInfo::from_track_obj(track),
                        possible_anime: more_by_artists
                            .iter()
                            .map(|a| FrontendAnimeEntry::from_db(&a))
                            .collect(),
                    }))
                }
            } else {
                println!("It is a sad moment for the database");
                // This is incorrect, at the very least we should add (CV:name) unwrapping to the artistnames in the func
                // We should probably also use the new update_or_add func somewhere inside there
                return Ok(self
                    .find_most_likely_anime(track, accuracy_cutoff, anisong_db)
                    .await
                    .unwrap());
            }
        }
    }
}
