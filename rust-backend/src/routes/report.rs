use std::sync::Arc;

use crate::{AppState, Result, spotify::api::get_user};
use axum::{Json, extract::State, response::IntoResponse};
use log::info;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

#[derive(Serialize, Deserialize)]
pub struct ReportParams {
    spotify_id: String,
    ann_song_id: i32,
    reason: String,
}

pub async fn report(
    State(app_state): State<Arc<AppState>>,
    session: Session,
    Json(params): Json<ReportParams>,
) -> Result<impl IntoResponse> {
    let token = session.get::<String>("access_token").await.unwrap_or(None);
    let user = match token {
        Some(token) => Some(get_user(token).await.unwrap()),
        None => None,
    };

    info!("{:?} made a report", user.as_ref().map(|a| &a.display_name));

    sqlx::query("INSERT INTO reports (spotify_id, ann_song_id, reason, user_name, user_mail) VALUES ($1, $2, $3, $4, $5)")
        .bind(&params.spotify_id)
        .bind(&params.ann_song_id)
        .bind(&params.reason)
        .bind(&user.as_ref().map(|u| u.display_name.clone()))
        .bind(&user.map(|u| u.email))
        .execute(&app_state.database.pool)
        .await
        .unwrap();

    return Ok(());
}
