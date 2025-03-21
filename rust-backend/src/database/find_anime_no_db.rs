use super::Database;
use crate::Result;
use crate::anisong::AnisongClient;
use crate::japanese_processing::process_possible_japanese;
use crate::spotify::responses::TrackObject;
use crate::types::{self, FrontendAnimeEntry, NewSong, SongHit, SongInfo, SongMiss};

impl Database {
    pub async fn find_most_likely_anime(
        &self,
        song: &TrackObject,
        accuracy_cutoff: f32,
        anisong_db: &AnisongClient,
    ) -> Result<NewSong> {
        let romanji_title = process_possible_japanese(&song.name);

        let mut anime = anisong_db.find_songs_by_artists(&song).await.unwrap();

        let mut found_by_artist = true;

        if anime.is_empty() {
            found_by_artist = false;

            anime = anisong_db
                .get_animes_by_song_title(romanji_title.clone(), false)
                .await
                .unwrap();
        }

        if !anime.is_empty() {
            let (best_anime, max_score) = if found_by_artist {
                AnisongClient::pick_best_by_song_name(&mut anime, &song.name).unwrap()
            } else {
                AnisongClient::pick_best_by_artist_names(
                    &mut anime,
                    song.artists.iter().map(|a| &a.name).collect(),
                )
                .unwrap()
            };

            let mut song_group_id = None;
            if max_score > Self::ACCURACY_AUTOADD_LIMIT {
                song_group_id = Some(
                    self.add_song_group_link(
                        &song.id,
                        &best_anime[0].songName,
                        &best_anime[0].artists.iter().map(|a| a.id).collect(),
                    )
                    .await,
                );

                self.try_add_artists(&best_anime[0].artists, &song.artists)
                    .await;
            }

            let (mut hit, mut more) = self
                .merge(vec![], vec![], best_anime, anime, song_group_id)
                .await
                .unwrap();

            if max_score > accuracy_cutoff {
                return Ok(types::NewSong::Hit(SongHit {
                    song_info: SongInfo::from_track_obj(song),
                    certainty: max_score as i32,
                    anime_info: FrontendAnimeEntry::from_db_animes(&hit),
                    more_with_artist: FrontendAnimeEntry::from_db_animes(&more),
                }));
            } else {
                more.append(&mut hit);

                return Ok(types::NewSong::Miss(SongMiss {
                    song_info: SongInfo::from_track_obj(song),
                    possible_anime: FrontendAnimeEntry::from_db_animes(&more),
                }));
            }
        } else {
            let possible_anime = anisong_db
                .get_animes_by_song_title(romanji_title.clone(), true)
                .await
                .unwrap();

            let found_anime =
                FrontendAnimeEntry::from_anisongs(&possible_anime.iter().map(|a| a).collect())
                    .await
                    .unwrap();

            let miss = SongMiss {
                song_info: SongInfo::from_track_obj(song),
                possible_anime: found_anime,
            };

            return Ok(types::NewSong::Miss(miss));
        }
    }
}
