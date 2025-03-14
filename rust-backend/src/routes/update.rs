use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::{
    AppState,
    error::{Error, Result},
    spotify::{
        api::{currently_playing, refresh_access_token},
        responses::{CurrentlyPlayingResponses, Item},
    },
    types::ContentUpdate,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateParams {
    refresh: Option<bool>,
}

pub async fn update(
    State(app_state): State<Arc<AppState>>,
    session: Session,
    Query(params): Query<UpdateParams>,
) -> Result<impl IntoResponse> {
    session.load().await.unwrap();

    let token_option = session.get::<String>("access_token").await.unwrap();

    match token_option {
        Some(_) => {
            let expire_time_option = session.get::<u64>("expire_time").await.unwrap().unwrap();
            if expire_time_option
                < SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            {
                refresh_access_token(session.clone(), app_state.clone())
                    .await
                    .unwrap();
            }
            let current_song_response = currently_playing(session.clone()).await.unwrap();

            let current_song = match current_song_response {
                CurrentlyPlayingResponses::Playing(value) => value,
                CurrentlyPlayingResponses::NotPlaying => {
                    session.insert("previously_played", "").await.unwrap();
                    return Ok(Json(ContentUpdate::NotPlaying));
                }
                CurrentlyPlayingResponses::BadToken => {
                    return Ok(Json(ContentUpdate::LoginRequired));
                }
                CurrentlyPlayingResponses::Ratelimited => {
                    return Ok(Json(ContentUpdate::NotPlaying));
                }
            };

            match current_song.item {
                Item::TrackObject(song) => {
                    if params.refresh.is_none_or(|value| !value)
                        && session
                            .get::<String>("previously_played")
                            .await
                            .unwrap()
                            .is_some_and(|value| value == song.id)
                    {
                        return Ok(Json(ContentUpdate::NoUpdates));
                    }
                    session.insert("previously_played", &song.id).await.unwrap();
                    let start = Instant::now();
                    let value = Ok(Json(ContentUpdate::NewSong(
                        app_state
                            .database
                            .get_anime_2(&song, &app_state.anisong_db, 40.0)
                            .await
                            .unwrap(),
                    )));
                    let duration = start.elapsed();
                    if duration > Duration::from_secs(1) {
                        warn!("Time to find animes: {:?}", duration);
                    }
                    value
                }
                _ => Err(Error::NotASong),
            }
        }
        None => {
            return Ok(Json(ContentUpdate::LoginRequired));
        }
    }
}
