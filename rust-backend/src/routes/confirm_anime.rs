use std::sync::Arc;

use axum::{extract::{Query, State}, response::{IntoResponse, Response}, Json};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use crate::{spotify::api::get_user, Error, Result};

use crate::{spotify::api::get_song, AppState};

#[derive(Deserialize, Serialize)]
pub struct ConfirmationParams {
    pub song_name: String,
    pub artist_ids: Vec<i32>,
    pub spotify_id: String,
}

pub async fn confirm_anime(
    State(app_state): State<Arc<AppState>>,
    session: Session,
    Json(params): Json<ConfirmationParams>,
) -> Result<impl IntoResponse> {
    let anisongs = app_state.anisong_db.get_exact_song(params.artist_ids, params.song_name).await.unwrap();

    let track = get_song(params.spotify_id, session.get::<String>("access_token").await.unwrap().unwrap()).await.unwrap();

    let user = get_user(session.get::<String>("access_token").await.unwrap().unwrap()).await.unwrap();

    let mut successes = Vec::new();
    let mut fails = Vec::new();

    let display_name = user.display_name.clone();
    let email = user.email.clone();
    for anime in &anisongs {
        let anime_name = anime.animeENName.clone();
        match app_state.database.try_add_anime_user(&track, anime.clone(), display_name.clone(), email.clone()).await {
            Ok(_) => {successes.push(anime_name);},
            Err(_) => {fails.push(anime_name)},
        };
    }
    if anisongs.len() > 0 && anisongs[0].artists.len() == 1 && track.artists.len() == 1 {
        app_state.database.add_artist_db(&anisongs[0].artists[0], &track.artists[0].id).await;
    }


    let mut string_response = String::new();
    if successes.len() > 0 {
        string_response.push_str(&format!("Succeded in adding: {}\n", successes.join(", ")));
    }
    if successes.len() > 0 {
        string_response.push_str(&format!("Succeded in adding: {}", successes.join(", ")));
    }

    Ok(Json(string_response))
}
