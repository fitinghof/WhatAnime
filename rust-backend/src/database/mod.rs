pub mod databasetypes;
pub mod find_anime_no_db;
pub mod regex_search;

use crate::Result;
use crate::anilist::Media;
use crate::anisong::{Anime, AnisongClient, Artist};
use crate::japanese_processing::process_similarity;
use crate::spotify::responses::{SimplifiedArtist, TrackObject};
use crate::types::{FrontendAnimeEntry, NewSong, SongHit, SongInfo, SongMiss};
// use axum_sessions::async_session::chrono::Duration;
use axum_sessions::async_session::log::info;
use databasetypes::{DBAnime, DBArtist, SongGroup, SongGroupLink};
use regex::{self, Regex};
use regex_search::{create_artist_regex, process_artist_name};
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, Pool, Postgres, QueryBuilder};
use std::collections::HashSet;
use std::{env, vec};

pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    const ACCURACY_AUTOADD_LIMIT: f32 = 80.0;
    // const UPDATE_TIME: Duration = Duration::days(7);
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

    async fn get_artists_spotify_id(&self, spotify_ids: &Vec<String>) -> Result<Vec<DBArtist>> {
        Ok(sqlx::query_as::<Postgres, DBArtist>(
            r#"
                SELECT * 
                FROM new_artists
                JOIN artist_links ON new_artists.ann_id = artist_links.ann_id
                WHERE artist_links.spotify_id = ANY($1)
                "#,
        )
        .bind(&spotify_ids)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn get_anime_by_spotify_id(&self, spotify_id: &String) -> Result<Vec<DBAnime>> {
        Ok(sqlx::query_as::<Postgres, DBAnime>(
            r#"
                SELECT *
                FROM animes
                JOIN song_group_links ON animes.song_group_id = song_group_links.group_id
                WHERE song_group_links.spotify_id = $1
                "#,
        )
        .bind(&spotify_id)
        .fetch_all(&self.pool)
        .await
        .unwrap())
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

    pub async fn update_or_add_animes(
        &self,
        animes: Vec<&DBAnime>,
        from_user: Option<String>,
        from_user_mail: Option<String>,
    ) {
        if animes.is_empty() {
            return;
        }

        info!("Trying to add or update {} animes", animes.len());

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"INSERT INTO animes (
                ann_id, title_eng, title_jpn, index_type, index_number, anime_type, episodes, mean_score,
                banner_image, cover_image_color, cover_image_medium, cover_image_large, cover_image_extra_large,
                media_format, genres, source, studio_ids, studio_names, studio_urls, tag_ids, tag_names, trailer_id,
                trailer_site, thumbnail, release_season, release_year,
                ann_song_id, song_name, spotify_artist_ids, artist_names, artists_ann_id, composers_ann_id,
                arrangers_ann_id, track_index_type, track_index_number, mal_id, anilist_id, anidb_id, kitsu_id, 
                song_group_id, from_user_name, from_user_mail
            ) "#,
        );

        query_builder.push_values(animes, |mut builder, anime| {
            builder
                .push_bind(&anime.ann_id)
                .push_bind(&anime.title_eng)
                .push_bind(&anime.title_jpn)
                .push_bind(&anime.index_type)
                .push_bind(&anime.index_number)
                .push_bind(&anime.anime_type)
                .push_bind(&anime.episodes)
                .push_bind(&anime.mean_score)
                .push_bind(&anime.banner_image)
                .push_bind(&anime.cover_image_color)
                .push_bind(&anime.cover_image_medium)
                .push_bind(&anime.cover_image_large)
                .push_bind(&anime.cover_image_extra_large)
                .push_bind(&anime.media_format)
                .push_bind(&anime.genres)
                .push_bind(&anime.source)
                .push_bind(&anime.studio_ids)
                .push_bind(&anime.studio_names)
                .push_bind(&anime.studio_urls)
                .push_bind(&anime.tag_ids)
                .push_bind(&anime.tag_names)
                .push_bind(&anime.trailer_id)
                .push_bind(&anime.trailer_site)
                .push_bind(&anime.thumbnail)
                .push_bind(&anime.release_season)
                .push_bind(&anime.release_year)
                .push_bind(&anime.ann_song_id)
                .push_bind(&anime.song_name)
                .push_bind(&anime.spotify_artist_ids)
                .push_bind(&anime.artist_names)
                .push_bind(&anime.artists_ann_id)
                .push_bind(&anime.composers_ann_id)
                .push_bind(&anime.arrangers_ann_id)
                .push_bind(&anime.track_index_type)
                .push_bind(&anime.track_index_number)
                .push_bind(&anime.mal_id)
                .push_bind(&anime.anilist_id)
                .push_bind(&anime.anidb_id)
                .push_bind(&anime.kitsu_id)
                .push_bind(&anime.song_group_id)
                .push_bind(&from_user)
                .push_bind(&from_user_mail);
        });

        query_builder.push(
        r#"ON CONFLICT (ann_song_id) DO UPDATE SET 
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
            song_group_id = COALESCE(EXCLUDED.song_group_id, animes.song_group_id),
            last_updated = EXCLUDED.last_updated"#
            );

        let query = query_builder.build();

        query.execute(&self.pool).await.unwrap();
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

    pub async fn try_add_anime_user(
        &self,
        spotify_track_object: &TrackObject,
        anisong_anime: Anime,
        from_user_name: Option<String>,
        from_user_mail: Option<String>,
    ) -> Result<()> {
        info!(
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

        let media = match anisong_anime.linked_ids.anilist {
            Some(id) => Media::fetch_one(id).await,
            None => None,
        };

        let group_id = self
            .add_song_group_link(
                &spotify_track_object.id,
                &anisong_anime.songName,
                &anisong_anime
                    .artists
                    .iter()
                    .map(|a| a.id)
                    .collect::<Vec<i32>>(),
            )
            .await;

        let db_anime =
            DBAnime::from_anisong_and_anilist(&anisong_anime, media.as_ref(), Some(group_id));

        self.update_or_add_animes(vec![&db_anime], from_user_name, from_user_mail)
            .await;
        Ok(())
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
                "SELECT * FROM new_artists WHERE EXISTS (
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

        let mut temp = artist_ann_ids.clone();
        artists
            .iter()
            .filter_map(|a| a.groups_ids.clone())
            .for_each(|g| temp.extend(g));

        let mut more_by_artists = self.get_animes_by_artists_ann_ids(&temp).await.unwrap();

        if anime.len() > 0 {
            Ok((anime, more_by_artists, artist_ann_ids, artists, 100.0))
        } else if more_by_artists.len() > 0 {
            let (best_match, certainty) =
                DBAnime::pick_best_by_song_name(&mut more_by_artists, &track.name).unwrap();

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

    // pub async fn try_add_artists_variation(
    //     &self,
    //     anisong_artists: &Vec<DBArtist>,
    //     spotify_artists: &Vec<SimplifiedArtist>,
    // ) {
    //     let mut artists_to_add = Vec::new();

    //     for artist in anisong_artists {
    //         let artist_pattern = create_artist_regex(artist.names.iter().collect());
    //         let re = Regex::new(&artist_pattern).unwrap();
    //         for sartist in spotify_artists {
    //             if re.is_match(&sartist.name) {
    //                 let mut new_artist = artist.clone();
    //                 info!(
    //                     "Binding ani artist {:?} to spotify artist {}",
    //                     &new_artist.names, &sartist.name
    //                 );
    //                 new_artist.spotify_id = sartist.id.clone();
    //                 artists_to_add.push(new_artist);
    //             }
    //         }
    //     }
    //     let mut tx = self.pool.begin().await.unwrap();

    //     for new_artist in artists_to_add {
    //         let _ = sqlx::query!(
    //             r#"INSERT INTO artists (spotify_id, ann_id, names, groups_ids, members)
    //             VALUES ($1, $2, $3, $4, $5)
    //             ON CONFLICT (spotify_id) DO NOTHING"#,
    //             &new_artist.spotify_id,
    //             &new_artist.ann_id,
    //             &new_artist.names,
    //             &new_artist.groups_ids.unwrap_or(vec![]),
    //             &new_artist.members.unwrap_or(vec![]),
    //         )
    //         .execute(&mut *tx)
    //         .await
    //         .unwrap();
    //     }

    //     tx.commit().await.unwrap();
    // }

    pub async fn try_add_artists(
        &self,
        anisong_artists: &Vec<Artist>,
        spotify_artists: &Vec<SimplifiedArtist>,
    ) {
        // Fetch already existing links to make better choices
        let existing_artist_links = sqlx::query_as::<Postgres, (i32, String)>(
            "SELECT * FROM artist_links WHERE spotify_id = ANY($1)",
        )
        .bind(
            spotify_artists
                .iter()
                .map(|a| a.id.clone())
                .collect::<Vec<String>>(),
        )
        .fetch_all(&self.pool)
        .await
        .unwrap();

        // make arrays of references to the artists so we can shuffle them around as we wish
        let mut anisong_artists: Vec<&Artist> = anisong_artists.iter().collect();
        let mut spotify_artists: Vec<&SimplifiedArtist> = spotify_artists.iter().collect();

        // filter out stuff already added
        for link in existing_artist_links {
            anisong_artists.retain(|a| a.id != link.0);
            spotify_artists.retain(|a| a.id != link.1);
        }

        let mut links = Vec::new();

        // Find best match
        for artist in anisong_artists.clone() {
            let mut eval_spotify: Vec<(&SimplifiedArtist, f32)> = spotify_artists
                .iter()
                .map(|&a| {
                    let max_score = artist
                        .names
                        .iter()
                        .map(|an| process_similarity(&process_artist_name(&a.name), &an))
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();
                    (a, max_score)
                })
                .collect();

            eval_spotify.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            if !eval_spotify.is_empty() && eval_spotify[0].1 > Self::ACCURACY_AUTOADD_LIMIT {
                links.push((artist.id, eval_spotify[0].0.id.clone()));
            }
        }

        let mut tx = self.pool.begin().await.unwrap();

        if !links.is_empty() {
            info!("Binding {} artists", links.len());

            // Insert links
            let mut query_builder: QueryBuilder<Postgres> =
                QueryBuilder::new(r#"Insert into artist_links (ann_id, spotify_id) "#);

            query_builder.push_values(links, |mut builder, link| {
                builder.push_bind(link.0).push_bind(link.1);
            });

            query_builder.push(" ON CONFLICT DO NOTHING");

            query_builder.build().execute(&mut *tx).await.unwrap();
        }

        // Insert all artists as these could still be usefull without links

        if !anisong_artists.is_empty() {
            info!("Adding {} artists to the database", anisong_artists.len());
            let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                r#"INSERT INTO new_artists (ann_id, names, groups_ids, members) "#,
            );

            query_builder.push_values(anisong_artists, |mut builder, artist| {
                builder
                    .push_bind(artist.id)
                    .push_bind(artist.names.clone())
                    .push_bind(
                        artist
                            .groups
                            .as_ref()
                            .map(|o| o.iter().map(|a| a.id).collect::<Vec<i32>>()),
                    )
                    .push_bind(
                        artist
                            .members
                            .as_ref()
                            .map(|o| o.iter().map(|a| a.id).collect::<Vec<i32>>()),
                    );
            });

            query_builder.push(" ON CONFLICT DO NOTHING");

            let query = query_builder.build();
            query.execute(&mut *tx).await.unwrap();
        }

        tx.commit().await.unwrap();
    }

    pub async fn merge(
        &self,
        mut anime_hits_db: Vec<DBAnime>,
        mut more_by_artist_db: Vec<DBAnime>,
        mut anime_hits_anisong: Vec<Anime>,
        mut more_by_artist_anisong: Vec<Anime>,
        song_group_id: Option<i32>,
    ) -> Result<(Vec<DBAnime>, Vec<DBAnime>)> {
        // Filter out initial duplicates, prefering stuff from our own database and hits over more by artist.
        let mut anime_set = HashSet::with_capacity(anime_hits_db.len() + more_by_artist_db.len());
        anime_hits_db.retain(|a| anime_set.insert(a.ann_song_id));
        anime_hits_anisong.retain(|a| anime_set.insert(a.annSongId));

        more_by_artist_db.retain(|a| anime_set.insert(a.ann_song_id));
        more_by_artist_anisong.retain(|a| anime_set.insert(a.annSongId));

        // fetch stuff from anisong that might already be in our database
        #[derive(Debug, FromRow)]
        pub struct LabledAnime {
            pub label: i16,
            #[sqlx(flatten)]
            pub anime: DBAnime,
        }

        let found_anime = sqlx::query_as::<Postgres, LabledAnime>(
            "SELECT 
                (CASE 
                    WHEN ann_song_id = ANY($1) THEN 0  -- Group A
                    ELSE 1  -- Group B
                END)::int2 AS label,
                *
            FROM animes
            WHERE ann_song_id = ANY($1) OR ann_song_id = ANY($2)",
        )
        .bind(
            anime_hits_anisong
                .iter()
                .map(|a| a.annSongId)
                .collect::<Vec<i32>>(),
        )
        .bind(
            more_by_artist_anisong
                .iter()
                .map(|a| a.annSongId)
                .collect::<Vec<i32>>(),
        )
        .fetch_all(&self.pool)
        .await
        .unwrap();

        // push it to the correct vec and filter it out form anisong vecs.
        let mut anisong_filter = HashSet::with_capacity(found_anime.len());
        for db_anime in found_anime {
            anisong_filter.insert(db_anime.anime.ann_song_id);
            match db_anime.label {
                0 => anime_hits_db.push(db_anime.anime),
                _ => more_by_artist_db.push(db_anime.anime),
            }
        }
        anime_hits_anisong.retain(|a| anisong_filter.insert(a.annSongId));
        more_by_artist_anisong.retain(|a| anisong_filter.insert(a.annSongId));

        // Gather ids for anilist fetch
        // We use a set here to prevent sending unneccessary data since many anisongs may contain the same annSongId
        let mut anilist_ids_set =
            HashSet::with_capacity(more_by_artist_anisong.len() + anime_hits_anisong.len());
        anilist_ids_set.extend(
            anime_hits_db
                .iter()
                .filter(|a| a.is_outdated())
                .filter_map(|a| a.anilist_id),
        );
        anilist_ids_set.extend(
            more_by_artist_db
                .iter()
                .filter(|a| a.is_outdated())
                .filter_map(|a| a.anilist_id),
        );
        anilist_ids_set.extend(
            anime_hits_anisong
                .iter()
                .filter_map(|a| a.linked_ids.anilist),
        );
        anilist_ids_set.extend(
            more_by_artist_anisong
                .iter()
                .filter_map(|a| a.linked_ids.anilist),
        );
        let anilist_ids = Vec::from_iter(anilist_ids_set.into_iter());
        // fetch all media
        let media = Media::fetch_many(anilist_ids).await.unwrap();

        // Promote the anisongs Anime to DBAnime
        let mut promoted_anisong_hit =
            DBAnime::from_anisongs_and_anilists(&anime_hits_anisong, &media, song_group_id)
                .unwrap();
        let mut promoted_anisong_more_by_artist =
            DBAnime::from_anisongs_and_anilists(&more_by_artist_anisong, &media, None).unwrap();

        // Update existing DBAnime and collect the copies of the Updated DBAnime
        let mut update_copies = DBAnime::update_all(&mut anime_hits_db, &media, song_group_id);

        update_copies.extend(DBAnime::update_all(&mut more_by_artist_db, &media, None));

        // Collect refs to everything that needs to be sent to the database
        let mut updates_or_adds = update_copies.iter().collect::<Vec<&DBAnime>>();
        updates_or_adds.extend(promoted_anisong_hit.iter());
        updates_or_adds.extend(promoted_anisong_more_by_artist.iter());

        // Send to database
        self.update_or_add_animes(updates_or_adds, Some("Database".to_string()), None)
            .await;

        // Assemble all hits and more_by_artist entries
        anime_hits_db.append(&mut promoted_anisong_hit);
        more_by_artist_db.append(&mut promoted_anisong_more_by_artist);

        Ok((anime_hits_db, more_by_artist_db))
    }

    pub async fn get_anime_2(
        &self,
        track: &TrackObject,
        anisong_db: &AnisongClient,
        accuracy_cutoff: f32,
    ) -> Result<NewSong> {
        let (mut hit_anime, more_by_artists, artists_ann_id, artists_searched, certainty) =
            self.db_full_search(track).await.unwrap();

        if certainty == 100.0 {
            let anisong_animes = anisong_db
                .get_animes_by_artists_ids(artists_ann_id)
                .await
                .unwrap();

            // split anisongs into hits and misses and add more ids
            let (anisong_anime_hits, anisong_anime_more): (Vec<Anime>, Vec<Anime>) =
                anisong_animes.into_iter().partition(|a| {
                    a.artists.iter().map(|a| a.id).collect::<Vec<i32>>()
                        == hit_anime[0].artists_ann_id
                        && a.songName == hit_anime[0].song_name
                });

            // Add artists and try and add artist links
            if let Some(artists) = anisong_anime_hits.first().map(|a| &a.artists) {
                self.try_add_artists(&artists, &track.artists).await;
            }

            // get group id.
            let group_id = self
                .add_song_group_link(
                    &track.id,
                    &hit_anime[0].song_name,
                    &hit_anime[0].artists_ann_id,
                )
                .await;

            let (mut hit_anime, mut more_by_artists) = self
                .merge(
                    hit_anime,
                    more_by_artists,
                    anisong_anime_hits,
                    anisong_anime_more,
                    Some(group_id),
                )
                .await
                .unwrap();

            more_by_artists.sort_by(|a, b| a.title_eng.cmp(&b.title_eng));
            hit_anime.sort_by(|a, b| a.title_eng.cmp(&b.title_eng));

            // Return result
            Ok(NewSong::Hit(SongHit {
                song_info: SongInfo::from_track_obj(track),
                certainty: certainty as i32,
                anime_info: hit_anime
                    .iter()
                    .map(|a| FrontendAnimeEntry::from_db_anime(a))
                    .collect(),
                more_with_artist: more_by_artists
                    .iter()
                    .map(|a| FrontendAnimeEntry::from_db_anime(a))
                    .collect(),
            }))
        } else {
            if artists_ann_id.len() > 0 {
                let mut anisongs = anisong_db
                    .get_animes_by_artists_ids(artists_ann_id.clone())
                    .await
                    .unwrap();
                let (mut anime_hits, score) =
                    AnisongClient::pick_best_by_song_name(&mut anisongs, &track.name).unwrap();

                // Add constant for acceptable match
                if score > accuracy_cutoff {
                    // get data for the best song
                    let best_song_name = anime_hits[0].songName.clone();
                    let best_artist_ids: Vec<i32> =
                        anime_hits[0].artists.iter().map(|a| a.id).collect();

                    let mut artist_set = HashSet::with_capacity(anime_hits[0].artists.len());

                    artists_ann_id.iter().for_each(|&id| {
                        artist_set.insert(id);
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
                        let (mut more_hits, mut more_by_artist): (Vec<Anime>, Vec<Anime>) =
                            additional_anisongs.into_iter().partition(|a| {
                                a.artists.iter().map(|a| a.id).collect::<Vec<i32>>()
                                    == best_artist_ids
                                    && a.songName == best_song_name
                            });
                        anime_hits.append(&mut more_hits);
                        anisongs.append(&mut more_by_artist);
                    }

                    // Try and add more artists to the database
                    if score > Self::ACCURACY_AUTOADD_LIMIT {
                        self.try_add_artists(&anime_hits[0].artists, &track.artists)
                            .await;
                    }

                    let group_id = if score > Self::ACCURACY_AUTOADD_LIMIT {
                        Some(
                            self.add_song_group_link(
                                &track.id,
                                &anime_hits[0].songName,
                                &anime_hits[0].artists.iter().map(|a| a.id).collect(),
                            )
                            .await,
                        )
                    } else {
                        None
                    };

                    let (mut anime_hit, mut more_by_artists) = self
                        .merge(vec![], more_by_artists, anime_hits, anisongs, group_id)
                        .await
                        .unwrap();

                    more_by_artists.sort_by(|a, b| a.title_eng.cmp(&b.title_eng));
                    anime_hit.sort_by(|a, b| a.title_eng.cmp(&b.title_eng));

                    // return Hit
                    Ok(NewSong::Hit(SongHit {
                        song_info: SongInfo::from_track_obj(track),
                        certainty: score as i32,
                        anime_info: anime_hit
                            .iter()
                            .map(|a| FrontendAnimeEntry::from_db_anime(a))
                            .collect(),
                        more_with_artist: more_by_artists
                            .iter()
                            .map(|a| FrontendAnimeEntry::from_db_anime(a))
                            .collect(),
                    }))
                } else {
                    let (_, mut possible) = self
                        .merge(vec![], more_by_artists, vec![], anisongs, None)
                        .await
                        .unwrap();

                    possible.append(&mut hit_anime);

                    Ok(NewSong::Miss(SongMiss {
                        song_info: SongInfo::from_track_obj(track),
                        possible_anime: possible
                            .iter()
                            .map(|a| FrontendAnimeEntry::from_db_anime(&a))
                            .collect(),
                    }))
                }
            } else {
                info!("It is a sad moment for the database");
                return Ok(self
                    .find_most_likely_anime(track, accuracy_cutoff, anisong_db)
                    .await
                    .unwrap());
            }
        }
    }
}
