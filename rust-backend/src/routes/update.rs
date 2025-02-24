use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{extract::State, response::IntoResponse, Json};
use tower_sessions::Session;

use crate::{
    error::{Error, Result}, link::find_most_likely_anime, spotify::{api::{currently_playing, refresh_access_token}, responses::Item}, types::ContentUpdate, AppState
};

pub async fn update(
    State(app_state): State<Arc<AppState>>,
    session: Session,
) -> Result<impl IntoResponse> {
    session.load().await.unwrap();

    let token_option = session.get::<String>("access_token").await?;

    match token_option {
        Some(_) => {
            let expire_time_option = session.get::<u64>("expire_time").await.unwrap().unwrap();
            if expire_time_option
                < SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            {
                refresh_access_token(session.clone(), app_state.clone()).await?;
            }
            let current_song_response = currently_playing(session.clone()).await?;
            match current_song_response.item {
                Item::TrackObject(song) => {
                    if session.get::<String>("previously_played").await?.is_some_and(|value| value == song.id){
                        return Ok(Json(ContentUpdate::NoUpdates));
                    }
                    session.insert("previously_played", &song.id).await?;
                    return Ok(Json(ContentUpdate::NewSong(find_most_likely_anime(&song, 40.0, app_state.anisong_db.clone()).await.unwrap())));
                }
                _ => {
                    Err(Error::NotASong)
                }
            }
        }
        None => {
            return Ok(Json(ContentUpdate::LoginRequired));
        }
    }
}
