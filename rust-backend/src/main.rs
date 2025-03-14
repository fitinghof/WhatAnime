mod anilist;
mod anisong;
mod database;
mod error;
mod japanese_processing;
mod routes;
mod spotify;
mod types;

use anisong::AnisongClient;
use axum::http::Method;
use axum::http::header::{ACCEPT, AUTHORIZATION};
use axum::routing::post;
use axum::{Router, http::HeaderValue, routing::get};
use database::Database;
use dotenv::dotenv;
use env_logger::Target;
pub use error::{Error, Result};
use log::info;
use std::time::Duration;
use std::{env, sync::Arc};
use tokio::task;
use tokio::time::interval;
use tower_http::cors::CorsLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer, cookie::SameSite};

use routes::{callback, confirm_anime, login, report, update};

struct AppState {
    client_id: String,
    client_secret: String,
    // ip: String,
    redirect_uri: String,
    anisong_db: AnisongClient,
    database: Database,
}

impl AppState {
    async fn load() -> Self {
        // let ip = env::var("ip").unwrap();
        let database = Database::new().await;
        database.run_migrations().await.unwrap();
        return Self {
            client_id: env::var("ClientID").unwrap(),
            client_secret: env::var("ClientSecret").unwrap(),
            redirect_uri: format!("http://whatanime.ddns.net:8000/callback"),
            // ip,
            anisong_db: AnisongClient::new(),
            database,
        };
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .filter_module("tracing", log::LevelFilter::Warn)
        .target(Target::Stdout)
        .init();

    task::spawn(async {
        let interval_duration = Duration::from_secs(60 * 60); // 1 hour
        let mut interval = interval(interval_duration);
        loop {
            interval.tick().await;
            info!("Sent Heartbeat");
        }
    });

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_always_save(true);
    //.with_expiry(Expiry::OnInactivity(Duration::seconds(10)));

    let shared_state = Arc::new(AppState::load().await);

    // migrate_database(&shared_state.database).await;

    let allowed_origins = [
        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
        "http://whatanime.ddns.net:5173"
            .parse::<HeaderValue>()
            .unwrap(),
    ];

    let app = Router::new()
        .route("/api/update", get(update))
        .route("/api/login", get(login))
        .route("/callback", get(callback))
        .route("/api/confirm_anime", post(confirm_anime))
        .route("/api/report", post(report))
        .layer(session_layer)
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([AUTHORIZATION, ACCEPT]),
        )
        .with_state(shared_state.clone()); // Enable CORS

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}
