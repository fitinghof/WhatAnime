use std::{collections::HashMap, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use axum::{extract::{Query, State}, http::HeaderValue, response::{IntoResponse, Redirect}};
use base64::{engine, Engine};
use axum::http::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use tower_sessions::Session;

use crate::{AppState, spotify::responses::SpotifyTokenResponse};

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

pub async fn callback(
    Query(params): Query<CallbackParams>,
    State(app_state): State<Arc<AppState>>,
    session: Session,
) -> axum::response::Response {
    let session_id = session.id();
    println!("Former Session state: {}", {&params.state});
    println!("Session ID: {:?}", session_id);
    let session_state = session.get::<String>("state").await.unwrap();

    if session_state.is_none_or(|value| value != params.state){
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }

    let client_creds = format!("{}:{}", app_state.client_id, app_state.client_secret);
    let client_creds_b64 = engine::general_purpose::STANDARD.encode(client_creds);

    let token_data = HashMap::from([
        ("code", params.code),
        ("redirect_uri", app_state.redirect_uri.clone()),
        ("grant_type", "authorization_code".to_string()),
    ]);

    let mut token_headers = HeaderMap::new();
    token_headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Basic {client_creds_b64}")).unwrap(),
    );
    token_headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    let token_response = Client::new()
        .post("https://accounts.spotify.com/authorize?")
        .headers(token_headers)
        .json(&token_data)
        .send()
        .await
        .unwrap();

    if token_response.status().is_success() {
        let token_info: SpotifyTokenResponse = token_response.json().await.unwrap();

        session
            .insert("access_token", token_info.access_token)
            .await
            .unwrap();
        session
            .insert("refresh_token", token_info.refresh_token)
            .await
            .unwrap();
        session
            .insert(
                "expire_time",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + token_info.expires_in,
            )
            .await
            .unwrap();

        let redirect_result = session
            .remove::<String>("redirect_url")
            .await
            .ok()
            .flatten();

        match redirect_result {
            Some(value) => return Redirect::to(&value).into_response(),
            _ => return Redirect::to(&format!("http://{}/", app_state.ip)).into_response(),
        }
    }

    return axum::http::StatusCode::BAD_REQUEST.into_response();
}