use crate::{AppState, Error, Result};

use super::responses::{CurrentlyPlayingResponses, SpotifyToken, SpotifyUser, TrackObject};
use base64::{Engine, engine};
use log::{error, warn};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
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
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Basic {}", client_creds_b64)).unwrap(),
    );

    let token_response = Client::new()
        .post(refresh_url)
        .headers(headers)
        .form(&token_data)
        .send()
        .await
        .unwrap();

    let token_info: SpotifyToken = match token_response.status() {
        status if status.is_success() => token_response.json().await.unwrap(),
        status => {
            error!(
                "Spotify returned error code: {} response text:\n{}",
                status,
                token_response.text().await.unwrap()
            );
            return Err(Error::BadRequest {
                url: refresh_url.to_string(),
                status_code: status,
            });
        }
    };

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
        // Note to self, make sure not parsable success cases are before the if status.is_success()
        axum::http::StatusCode::NO_CONTENT => Ok(CurrentlyPlayingResponses::NotPlaying),
        status if status.is_success() => Ok(CurrentlyPlayingResponses::Playing(
            response.json().await.unwrap(),
        )),
        axum::http::StatusCode::UNAUTHORIZED => Ok(CurrentlyPlayingResponses::BadToken),
        axum::http::StatusCode::FORBIDDEN => Err(Error::BadOAuth),
        axum::http::StatusCode::TOO_MANY_REQUESTS => Ok(CurrentlyPlayingResponses::Ratelimited),
        status if status.is_server_error() => {
            warn!(
                "Spotify returned code {}, response text:\n{}",
                status,
                response.text().await.unwrap(),
            );
            Ok(CurrentlyPlayingResponses::SpotifyError(status)) // Might want to implement better logic here
        }
        status => {
            error!(
                "Spotify returned unhandled code {}, response text:\n{}",
                status,
                response.text().await.unwrap(),
            );
            Err(Error::BadRequest {
                url: currently_playing_url.to_string(),
                status_code: status,
            })
        }
    }
}

pub async fn get_song(spotify_id: String, token: String) -> Result<TrackObject> {
    //--header 'Authorization: Bearer 1POdFZRZbvb...qqillRxMr2z'
    let url = format!("https://api.spotify.com/v1/tracks/{}", spotify_id);

    let response = Client::new()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    match response.status() {
        status if status.is_success() => Ok(response.json().await.unwrap()),
        status => {
            error!(
                "Spotify returned error code: {} response text:\n{}",
                status,
                response.text().await.unwrap()
            );
            Err(Error::BadRequest {
                url: url,
                status_code: status,
            })
        }
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

    match response.status() {
        status if status.is_success() => Ok(response.json().await.unwrap()),
        status => {
            error!(
                "Spotify returned error code: {} response text:\n{}",
                status,
                response.text().await.unwrap()
            );
            Err(Error::BadRequest {
                url: url.to_string(),
                status_code: status,
            })
        }
    }
}
