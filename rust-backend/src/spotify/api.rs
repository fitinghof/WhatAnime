use crate::types::ContentUpdate;
use crate::{AppState, Error, Result};

use super::responses::{self, CurrentlyPlayingResponse, CurrentlyPlayingResponses, SpotifyToken, SpotifyUser, TrackObject};
use axum::response::IntoResponse;
use base64::{engine, Engine};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
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
        // ("client_id", app_state.client_id.clone()), only for PKCE extension or something
    ]);

    let refresh_url = "https://accounts.spotify.com/api/token";
    let mut headers = HeaderMap::new();

    let client_creds = format!("{}:{}", app_state.client_id, app_state.client_secret);
    let client_creds_b64 = engine::general_purpose::STANDARD.encode(client_creds);

    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    headers.insert("Authorization", HeaderValue::from_str(&format!("Basic {}", client_creds_b64)).unwrap());

    let token_response = Client::new()
        .post(refresh_url)
        .headers(headers)
        .form(&token_data)
        .send()
        .await
        .unwrap();

    if token_response.status() != 200 {
        println!("Spotify didn't want to refresh our toke : (, {}", token_response.status());
        let status = token_response.status();
        println!("{}", token_response.text().await.unwrap());
        return Err(Error::BadRequest { url: "https://accounts.spotify.com/api/token".to_string(), status_code: status});
    }

    let token_info: SpotifyToken = token_response.json().await.unwrap();
    if token_info.refresh_token.is_some() {
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
            .insert("refresh_token", token_info.refresh_token.unwrap())
            .await?;
    }
    return Ok(());
}

pub async fn currently_playing(session: Session) -> Result<CurrentlyPlayingResponses> {
    let access_token = session
        .get::<String>("access_token")
        .await
        .unwrap()
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
        .await
        .unwrap();

    match response.status() {
        status if status == axum::http::StatusCode::NO_CONTENT => {
            Ok(CurrentlyPlayingResponses::NotPlaying)
        },
        status if status == 401 => {
            Ok(CurrentlyPlayingResponses::BadToken)
        }
        status if status == 403 => {
            Err(Error::BadOAuth)
        }
        status if status == 429 => {
            Ok(CurrentlyPlayingResponses::Ratelimited)
        }
        status if status.is_success() => {
            Ok(CurrentlyPlayingResponses::Playing(response.json().await.unwrap()))
        },
        status => Err(Error::BadRequest {
            url: "https://api.spotify.com/v1/me/player/currently-playing".to_string(),
            status_code: status,
        }),
    }
}

pub async fn get_song(spotify_id: String, token: String) -> Result<TrackObject> {

    //--header 'Authorization: Bearer 1POdFZRZbvb...qqillRxMr2z'
    let url = format!("https://api.spotify.com/v1/tracks/{}", spotify_id);

    let response = Client::new()
    .get(&url)
    .header("Authorization", format!("Bearer {}", token))
    .send()
    .await.unwrap();

    if response.status().is_success() {
        Ok(response.json().await.unwrap())
    }
    else {
        let status = response.status();
        println!("{}", response.text().await.unwrap());
        Err(Error::BadRequest { url: url, status_code: status })
    }
}

pub async fn get_user(token: String) -> Result<SpotifyUser> {
    let url = "https://api.spotify.com/v1/me";

    let response = Client::new()
    .get(url)
    .header("Authorization", format!("Bearer {}", &token))
    .send()
    .await
    .unwrap();

    if response.status().is_success() {
        Ok(response.json().await.unwrap())
    }
    else {
        let status = response.status();
        println!("{}", response.text().await.unwrap());
        Err(Error::BadRequest { url: "https://api.spotify.com/v1/me".to_string(), status_code: status })
    }
}
