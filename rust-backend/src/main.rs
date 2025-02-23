mod anisong;
mod error;
mod japanese_processing;
mod link;
mod routes;
mod spotify;
mod types;

use anisong::AnisongClient;
use axum::{
    Router,
    routing::{get, post},
};
use dotenv::dotenv;
pub use error::{Error, Result};
use std::{env, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tower_sessions::{cookie::{time::Duration, SameSite}, Expiry, MemoryStore, Session, SessionManagerLayer};

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
        .with_expiry(Expiry::OnInactivity(Duration::seconds(10)))
        .with_always_save(true)
        .with_same_site(SameSite::None);

    let shared_state = Arc::new(AppState::load());
    let app = Router::new()
        .route("/update", get(update))
        .route("/login", get(login))
        .route("/callback", get(callback))
        .layer(session_layer)
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(shared_state); // Enable CORS
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap()
}
