use crate::AppState;

use axum::{extract::State, response::IntoResponse};
use rand::Rng;
use std::{collections::HashMap, sync::Arc};
use tower_sessions::Session;
use url::form_urlencoded;

pub async fn login(State(app_state): State<Arc<AppState>>, session: Session) -> impl IntoResponse {
    let random_bytes: [u8; 16] = rand::rng().random();
    let scope =
        "user-read-private user-read-email user-read-playback-state user-read-currently-playing";
    let state = hex::encode(random_bytes);

    session.insert("state", state.clone()).await.unwrap();

    let auth_params = HashMap::from([
        ("client_id", app_state.client_id.clone()),
        ("response_type", "code".to_string()),
        ("redirect_uri", app_state.redirect_uri.clone()),
        ("state", state),
        ("scope", scope.to_string()),
    ]);

    let auth_url = format!(
        "https://accounts.spotify.com/authorize?{}",
        form_urlencoded::Serializer::new(String::new())
            .extend_pairs(auth_params)
            .finish()
    );

    return axum::response::Redirect::to(&auth_url);
}
