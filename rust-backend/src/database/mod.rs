pub mod databasetypes;
pub mod find_anime_no_db;

use crate::anilist::types::{Genre, TagID, URL};
use crate::anilist::{self, Media};
use crate::anisong::{Anime, AnisongClient, Artist};
use crate::japanese_processing::process_similarity;
use crate::spotify::responses::TrackObject;
use crate::types::{
    AnimeIndex, AnimeTrackIndex, AnimeType, FrontendAnimeEntry, NewSong, SongHit, SongInfo,
    SongMiss,
};
use crate::{Error, Result};
use databasetypes::{DBAnime, DBArtist, SongGroup, SongGroupLink};
use itertools::Itertools;
use reqwest::StatusCode;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::collections::HashSet;
use std::env;

pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    const ACCURACY_AUTOADD_LIMIT: f32 = 80.0;
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

    async fn get_animes_by_artists_ids_(&self, spotify_ids: &Vec<String>) -> Result<Vec<DBAnime>> {
        Ok(sqlx::query_as::<Postgres, DBAnime>(
            r#"
                SELECT DISTINCT a.*
                FROM animes a
                INNER JOIN anime_artists aa ON a.ann_id = aa.anime_id
                INNER JOIN artists ar ON aa.artist_id = ar.spotify_id
                WHERE ar.spotify_id = ANY($1)
                "#,
        )
        .bind(&spotify_ids)
        .fetch_all(&self.pool)
        .await?)
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
        let anime_index = AnimeIndex::from_str(&anisong_anime.animeCategory).unwrap();
        let anime_type = AnimeType::from_str(&anisong_anime.animeType.unwrap()).unwrap();
        let track_index = AnimeTrackIndex::from_str(&anisong_anime.songType).unwrap();
        let _ = sqlx::query::<Postgres>(
            r#"
            INSERT INTO animes (
                ann_id,
                title_eng,
                title_jpn,
                index_type,
                index_number,
                anime_type,
                episodes,
                mean_score,
                banner_image,
                cover_image_color,
                cover_image_medium,
                cover_image_large,
                cover_image_extra_large,
                media_format,
                genres,
                source,
                studio_ids,
                studio_names,
                studio_urls,
                tag_ids,
                tag_names,
                trailer_id,
                trailer_site,
                thumbnail,
                release_season,
                release_year,
                from_user_name,
                from_user_mail,
                spotify_id,
                ann_song_id,
                song_name,
                spotify_artist_ids,
                artist_names,
                artists_ann_id,
                composers_ann_id,
                arrangers_ann_id,
                track_index_type,
                track_index_number,
                mal_id,
                anilist_id,
                anidb_id,
                kitsu_id,
                song_group_id
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31,
                $32, $33, $34, $35, $36, $37, $38, $39, $40, $41, $42, $43
            )
            ON CONFLICT DO NOTHING
        "#,
        )
        .bind(anisong_anime.annId) // i32
        .bind(anisong_anime.animeENName) // String
        .bind(anisong_anime.animeJPName) // String
        .bind(anime_index.discriminant() as i16) // i16
        .bind(anime_index.value()) // i32
        .bind(anime_type as i16) // i16
        .bind(info.episodes) // Option<i32>
        .bind(info.mean_score) // i32
        .bind(info.banner_image.as_ref()) // Option<String>
        .bind(info.cover_image.as_ref().and_then(|c| c.color.clone())) // Option<String>
        .bind(info.cover_image.as_ref().and_then(|c| c.medium.clone())) // Option<String>
        .bind(info.cover_image.as_ref().and_then(|c| c.large.clone())) // Option<String>
        .bind(
            info.cover_image
                .as_ref()
                .and_then(|c| c.extra_large.clone()),
        ) // Option<String>
        .bind(info.format.map(|f| f as i16)) // Option<MediaFormat>
        .bind(info.genres.as_ref()) // Option<Vec<Genre>>
        .bind(info.source.as_ref()) // Option<String>
        .bind(
            info.studios
                .as_ref()
                .map(|s| s.nodes.iter().map(|s| s.id).collect::<Vec<i32>>()),
        ) // Option<Vec<i32>>
        .bind(info.studios.as_ref().map(|s| {
            s.nodes
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<Option<String>>>()
        }))
        .bind(info.studios.as_ref().map(|s| {
            s.nodes
                .iter()
                .map(|s| s.site_url.clone())
                .collect::<Vec<Option<URL>>>()
        }))
        .bind(
            info.tags
                .as_ref()
                .map(|t| t.iter().map(|a| a.id.clone()).collect::<Vec<TagID>>()),
        ) // Option<Vec<i32>>
        .bind(info.tags.as_ref().map(|t| {
            t.iter()
                .map(|t| t.name.clone())
                .collect::<Vec<Option<String>>>()
        }))
        .bind(info.trailer.as_ref().map(|t| t.id.clone())) // Option<String>
        .bind(info.trailer.as_ref().map(|t| t.site.clone())) // Option<String>
        .bind(info.trailer.as_ref().map(|t| t.thumbnail.clone())) // Option<String>
        .bind(info.season.map(|s| s as i16)) // Option<ReleaseSeason>
        .bind(info.season_year) // Option<i32>
        .bind(from_user_name) // Option<String>
        .bind(from_user_mail) // Option<String>
        .bind("") // Should be removed completly by next database migration, replaced by song group id.
        .bind(anisong_anime.annSongId)
        .bind(anisong_anime.songName.clone())
        .bind(
            spotify_track_object
                .artists
                .iter()
                .map(|a| a.id.clone())
                .collect::<Vec<String>>(),
        )
        .bind(
            anisong_anime
                .artists
                .iter()
                .map(|a| a.names[0].clone())
                .collect::<Vec<String>>(),
        )
        .bind(
            anisong_anime
                .artists
                .iter()
                .map(|a| a.id.clone())
                .collect::<Vec<i32>>(),
        )
        .bind(
            anisong_anime
                .composers
                .iter()
                .map(|a| a.id)
                .collect::<Vec<i32>>(),
        )
        .bind(
            anisong_anime
                .arrangers
                .iter()
                .map(|a| a.id)
                .collect::<Vec<i32>>(),
        )
        .bind(track_index.discriminant() as i16)
        .bind(track_index.value())
        .bind(anisong_anime.linked_ids.myanimelist)
        .bind(anisong_anime.linked_ids.anilist)
        .bind(anisong_anime.linked_ids.anidb)
        .bind(anisong_anime.linked_ids.kitsu)
        .bind(group_id)
        .execute(&self.pool)
        .await
        .unwrap();
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
            "User {:?}, mail: {:?}, added bind for {}  ---  {}",
            &from_user_name,
            &from_user_mail,
            &anisong_anime.animeENName,
            &spotify_track_object.name
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

    pub async fn get_anime(
        &self,
        spotify_track_object: &TrackObject,
        anisong_db: &AnisongClient,
        accuracy_cutoff: f32,
    ) -> Result<NewSong> {
        let anime_result = self
            .get_anime_by_spotify_id(&spotify_track_object.id)
            .await
            .unwrap();

        if anime_result.len() > 0 {
            let mut anime_set = HashSet::new();

            anime_result.iter().for_each(|a| {
                anime_set.insert((a.ann_id, a.ann_song_id));
            });

            let more_by_artists_db: Vec<DBAnime> = self
                .get_animes_by_artists_ids_(&anime_result[0].spotify_artist_ids)
                .await
                .unwrap()
                .into_iter()
                .filter(|a| anime_set.insert((a.ann_id, a.ann_song_id)))
                .collect();

            let anisong_hit: Vec<Anime> = anisong_db
                .get_exact_song(
                    anime_result[0].artists_ann_id.to_vec(),
                    anime_result[0].song_name.clone(),
                )
                .await
                .unwrap()
                .into_iter()
                .filter(|a| anime_set.insert((a.annId, a.annSongId)))
                .collect();

            let mut return_anime_hit = Vec::new();

            if anisong_hit.len() > 0 {
                for anime in anisong_hit {
                    let _ = self
                        .try_add_anime_db(spotify_track_object, anime.clone())
                        .await;
                    return_anime_hit.push(FrontendAnimeEntry::from_anisong(&anime).await.unwrap());
                }
            }
            anime_result
                .iter()
                .for_each(|a| return_anime_hit.push(FrontendAnimeEntry::from_db(a)));

            let anisong_more_by_artists: Vec<Anime> = anisong_db
                .get_animes_by_artists_ids(anime_result[0].artists_ann_id.clone())
                .await
                .unwrap()
                .into_iter()
                .filter(|a| anime_set.insert((a.annId, a.annSongId)))
                .collect();

            let mut return_more_by_artists =
                FrontendAnimeEntry::from_anisongs(&anisong_more_by_artists.iter().collect())
                    .await
                    .unwrap();

            for anime in more_by_artists_db {
                return_more_by_artists.push(FrontendAnimeEntry::from_db(&anime));
            }

            return_anime_hit.sort_by(|a, b| a.title.cmp(&b.title));
            return_more_by_artists.sort_by(|a, b| a.title.cmp(&b.title));

            Ok(NewSong::Hit(SongHit {
                song_info: SongInfo::from_track_obj(spotify_track_object),
                anime_info: return_anime_hit,
                more_with_artist: return_more_by_artists,
                certainty: 100,
            }))
        } else {
            // --------------- GET BY ARTISTS ---------------
            let artists_db = self
                .get_artists_spotify_id(
                    &spotify_track_object
                        .artists
                        .iter()
                        .map(|a| a.id.clone())
                        .collect::<Vec<String>>(),
                )
                .await
                .unwrap();

            if artists_db.len() > 0 {
                let anisong_anime = anisong_db
                    .get_animes_by_artists_ids(artists_db.iter().map(|a| a.ann_id).collect())
                    .await
                    .unwrap();
                if anisong_anime.len() > 0 {
                    let mut evaluated_anime: Vec<(&Anime, f32)> = anisong_anime
                        .iter()
                        .map(|a| {
                            (
                                a,
                                process_similarity(&spotify_track_object.name, &a.songName),
                            )
                        })
                        .collect();
                    evaluated_anime.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                    let best_score = evaluated_anime[0].1;

                    if best_score > accuracy_cutoff {
                        let (animehit, more_by_artists): (Vec<(&Anime, f32)>, Vec<(&Anime, f32)>) =
                            evaluated_anime
                                .into_iter()
                                .partition(|anime| anime.1 == best_score);

                        if best_score > Self::ACCURACY_AUTOADD_LIMIT {
                            for anime in &animehit {
                                let _ = self
                                    .try_add_anime_db(spotify_track_object, anime.0.clone())
                                    .await;
                            }

                            // try adding artist to database if correctness can be semi garanteed
                            if animehit[0].0.artists.len() == (artists_db.len() - 1)
                                && spotify_track_object.artists.len() == animehit[0].0.artists.len()
                            {
                                let spotify_artist =
                                    &spotify_track_object.artists.iter().find(|a| {
                                        !artists_db.iter().map(|b| &b.spotify_id).contains(&a.id)
                                    });
                                let anisong_artist =
                                    &animehit[0].0.artists.iter().find(|a| {
                                        !artists_db.iter().map(|b| b.ann_id).contains(&a.id)
                                    });

                                if spotify_artist.is_some() && anisong_artist.is_some() {
                                    let _ = self.add_artist_db(
                                        anisong_artist.unwrap(),
                                        &spotify_artist.unwrap().id,
                                    );
                                }
                            }
                        }

                        let mut anime_info = FrontendAnimeEntry::from_anisongs(
                            &animehit.iter().map(|a| a.0).collect(),
                        )
                        .await
                        .unwrap();

                        let mut more_with_artist = FrontendAnimeEntry::from_anisongs(
                            &more_by_artists.iter().map(|a| a.0).collect(),
                        )
                        .await
                        .unwrap();

                        anime_info.sort_by(|a, b| a.title.cmp(&b.title));
                        more_with_artist.sort_by(|a, b| a.title.cmp(&b.title));

                        return Ok(NewSong::Hit(SongHit {
                            song_info: SongInfo::from_track_obj(spotify_track_object),
                            anime_info: anime_info,
                            more_with_artist: more_with_artist,
                            certainty: best_score as i32,
                        }));
                    } else {
                        let mut possible_anime =
                            FrontendAnimeEntry::from_anisongs(&anisong_anime.iter().collect())
                                .await
                                .unwrap();

                        possible_anime.sort_by(|a, b| a.title.cmp(&b.title));

                        return Ok(NewSong::Miss(SongMiss {
                            song_info: SongInfo::from_track_obj(spotify_track_object),
                            possible_anime: possible_anime,
                        }));
                    }
                } else {
                    return Ok(NewSong::Miss(SongMiss {
                        song_info: SongInfo::from_track_obj(&spotify_track_object),
                        possible_anime: vec![],
                    }));
                }
            } else {
                return Ok(self
                    .find_most_likely_anime(spotify_track_object, accuracy_cutoff, anisong_db)
                    .await
                    .unwrap());
            }
        }
    }
    // You can add more bound functions here for your queries and operations.
}
