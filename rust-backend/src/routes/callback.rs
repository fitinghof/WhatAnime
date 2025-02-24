use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::http::HeaderMap;
use axum::{
    extract::{Query, State},
    http::HeaderValue,
    response::{IntoResponse, Redirect, Response},
};
use base64::{Engine, engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::{AppState, spotify::responses::SpotifyTokenResponse};

//#[derive(Deserialize)]
// enum CodeOrState {
//     error(String),
//     state(String),
// }

#[derive(Deserialize, Serialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

pub async fn callback(
    Query(params): Query<CallbackParams>,
    State(app_state): State<Arc<AppState>>,
    session: Session,
) -> Result<axum::response::Redirect, axum::http::StatusCode> {
    session.load().await.unwrap();

    let session_state = session.get::<String>("state").await.unwrap_or(None);
    if session_state.as_deref() != Some(&params.state) {
        println!("Sate missmatch occured, probably");
        println!("{}, {:?}", params.state, session_state);
        return Err(axum::http::StatusCode::BAD_REQUEST);

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
        .post("https://accounts.spotify.com/api/token")
        .headers(token_headers)
        .form(&token_data)
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

        session.save().await.unwrap();

        return Ok(Redirect::to(&format!("http://127.0.0.1:5173/")));
    }

    return Err(axum::http::StatusCode::BAD_REQUEST);
}
