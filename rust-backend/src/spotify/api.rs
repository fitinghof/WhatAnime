use crate::{AppState, Error, Result};

use super::responses::{CurrentlyPlayingResponse, SpotifyTokenResponse};
use axum::extract::{Query, State};
use axum::http::response;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, redirect};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_sessions::Session;

pub async fn refresh_access_token(session: Session, app_state: Arc<AppState>) -> Result<()> {
    let token_data = HashMap::from([
        ("grant_type", "refresh_token".to_string()),
        (
            "refresh_token",
            session.get("refresh_token").await?.unwrap(),
        ),
        ("client_id", app_state.client_id.clone()),
        ("client_secret", app_state.client_secret.clone()),
    ]);

    let refresh_url = "https://accounts.spotify.com/api/token";
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    let token_response = Client::new()
        .post(refresh_url)
        .headers(headers)
        .json(&token_data)
        .send()
        .await
        .unwrap();

    let token_info: SpotifyTokenResponse = token_response.json().await?;

    session
        .insert("access_token", token_info.access_token)
        .await?;
    session
        .insert(
            "expire_time",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + token_info.expires_in,
        )
        .await?;
    session
        .insert("refresh_token", token_info.refresh_token)
        .await?;

    return Ok(());
}

pub async fn currently_playing(session: Session) -> Result<CurrentlyPlayingResponse> {
    let access_token = session
        .get::<String>("access_token")
        .await?
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
    );
    let currently_playing_url = "https://api.spotify.com/v1/me/player/currently-playing";

    let response = Client::new()
        .get(currently_playing_url)
        .headers(headers)
        .send()
        .await?;

    match response.status() {
        status if status.is_success() => Ok(response.json().await.unwrap()),
        status => Err(Error::BadRequest {
            url: "https://api.spotify.com/v1/me/player/currently-playing".to_string(),
            status_code: (status),
        }),
    }
}
