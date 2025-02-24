mod anisong;
mod error;
mod japanese_processing;
mod link;
mod routes;
mod spotify;
mod types;

use anisong::AnisongClient;
use axum::{Router, http::HeaderValue, routing::get};
use dotenv::dotenv;
pub use error::{Error, Result};
use axum::http::Method;
use axum::http::header::{AUTHORIZATION, ACCEPT};
use std::{env, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tower_sessions::{
    Expiry, MemoryStore, SessionManagerLayer,
    cookie::{SameSite, time::Duration},
};

use routes::{callback, login, update};

struct AppState {
    client_id: String,
    client_secret: String,
    ip: String,
    redirect_uri: String,
    anisong_db: Arc<AnisongClient>,
}

impl AppState {
    fn load() -> Self {
        let ip = env::var("ip").unwrap();
        return Self {
            client_id: env::var("ClientID").unwrap(),
            client_secret: env::var("ClientSecret").unwrap(),
            redirect_uri: format!("http://{ip}/callback"),
            ip,
            anisong_db: Arc::new(AnisongClient::new()),
        };
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_always_save(true);
        //.with_expiry(Expiry::OnInactivity(Duration::seconds(10)));

    let shared_state = Arc::new(AppState::load());

    let allowed_origins = [
        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:5173".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:8000".parse::<HeaderValue>().unwrap(),
        "http://localhost:8000".parse::<HeaderValue>().unwrap(),
    ];

    let app = Router::new()
        .route("/api/update", get(update))
        .route("/login", get(login))
        .route("/callback", get(callback))
        .layer(session_layer)
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([AUTHORIZATION, ACCEPT])
        )
        .with_state(shared_state); // Enable CORS

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap()
}
