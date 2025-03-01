mod anisong;
mod error;
mod japanese_processing;
mod routes;
mod spotify;
mod types;
mod database;
mod Anilist;

use anisong::AnisongClient;
use axum::routing::post;
use axum::{Router, http::HeaderValue, routing::get};
use dotenv::dotenv;
pub use error::{Error, Result};
use axum::http::Method;
use axum::http::header::{AUTHORIZATION, ACCEPT};
use std::{env, sync::Arc};
use tower_http::cors::CorsLayer;
use tower_sessions::{
    MemoryStore, SessionManagerLayer,
    cookie::SameSite,
};
use database::Database;

use routes::{callback, login, update, confirm_anime};

struct AppState {
    client_id: String,
    client_secret: String,
    ip: String,
    redirect_uri: String,
    anisong_db: AnisongClient,
    database: Database,
}

impl AppState {
    async fn load() -> Self {
        let ip = env::var("ip").unwrap();
        let database = Database::new().await;
        database.run_migrations().await.unwrap();
        return Self {
            client_id: env::var("ClientID").unwrap(),
            client_secret: env::var("ClientSecret").unwrap(),
            redirect_uri: format!("http://{ip}:8000/callback"),
            ip,
            anisong_db: AnisongClient::new(),
            database,
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

    let shared_state = Arc::new(AppState::load().await);

    let allowed_origins = [
        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:5173".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:8000".parse::<HeaderValue>().unwrap(),
        "http://localhost:8000".parse::<HeaderValue>().unwrap(),
        format!("http://{}:5173", &shared_state.ip).parse::<HeaderValue>().unwrap(),
    ];

    let app = Router::new()
        .route("/api/update", get(update))
        .route("/api/login", get(login))
        .route("/callback", get(callback))
        .route("/api/confirm_anime", post(confirm_anime))
        .layer(session_layer)
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([AUTHORIZATION, ACCEPT])
        )
        .with_state(shared_state.clone()); // Enable CORS

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap()
}
