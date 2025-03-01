mod Anilist;
mod anisong;
mod database;
mod error;
mod japanese_processing;
mod routes;
mod spotify;
mod types;

use Anilist::Media;
use Anilist::types::{AnilistID, TagID, URL};
use anisong::AnisongClient;
use axum::http::Method;
use axum::http::header::{ACCEPT, AUTHORIZATION};
use axum::routing::post;
use axum::{Router, http::HeaderValue, routing::get};
use database::Database;
use dotenv::dotenv;
pub use error::{Error, Result};
use std::{env, sync::Arc};
use tower_http::cors::CorsLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer, cookie::SameSite};

use routes::{callback, confirm_anime, login, update};

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
struct SimpleEntry {
    spotify_id: String,
    anilist_id: Option<i32>,
}
async fn migrate_database(database: &Database) {
    let mut entries = sqlx::query_as!(SimpleEntry, "SELECT spotify_id, anilist_id FROM animes")
        .fetch_all(&database.pool)
        .await
        .unwrap();

    entries.retain(|a| a.anilist_id.is_some());

    let mut anilist_entries = Media::fetch_many(
        entries
            .iter()
            .map(|a| AnilistID::from(a.anilist_id.unwrap()))
            .collect(),
    )
    .await
    .unwrap();

    anilist_entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries.sort_by(|a, b| a.anilist_id.cmp(&b.anilist_id));

    let mut simple_entry_index = 0;
    let mut anilist_index = 0;

    while simple_entry_index < entries.len() && anilist_index < anilist_entries.len() {
        if AnilistID::from(entries[simple_entry_index].anilist_id.unwrap())
            == anilist_entries[anilist_index].id
        {
            sqlx::query(
                r#"
            UPDATE animes
            SET
                mean_score = $1,
                banner_image = $2,
                cover_image_color = $3,
                cover_image_medium = $4,
                cover_image_large = $5,
                cover_image_extra_large = $6,
                media_format = $7,
                genres = $8,
                source = $9,
                studio_ids = $10,
                studio_names = $11,
                studio_urls = $12,
                tag_ids = $13,
                tag_names = $14,
                trailer_id = $15,
                trailer_site = $16,
                thumbnail = $17,
                release_season = $18
            WHERE anilist_id = $19
        "#,
            )
            .bind(anilist_entries[anilist_index].mean_score)
            .bind(anilist_entries[anilist_index].banner_image.clone())
            .bind(
                anilist_entries[anilist_index]
                    .cover_image
                    .as_ref()
                    .map(|a| a.color.clone()),
            )
            .bind(
                anilist_entries[anilist_index]
                    .cover_image
                    .as_ref()
                    .map(|a| a.medium.clone()),
            )
            .bind(
                anilist_entries[anilist_index]
                    .cover_image
                    .as_ref()
                    .map(|a| a.large.clone()),
            )
            .bind(
                anilist_entries[anilist_index]
                    .cover_image
                    .as_ref()
                    .map(|a| a.extra_large.clone()),
            )
            .bind(anilist_entries[anilist_index].format.as_ref().map(|a| a.clone() as i16))
            .bind(anilist_entries[anilist_index].genres.as_ref())
            .bind(anilist_entries[anilist_index].source.as_ref().map(|a| a.clone() as i16))
            .bind(
                anilist_entries[anilist_index]
                    .studios
                    .as_ref()
                    .map(|a| a.nodes.iter().map(|a| a.id).collect::<Vec<i32>>()),
            )
            .bind(anilist_entries[anilist_index].studios.as_ref().map(|a| {
                a.nodes
                    .iter()
                    .map(|a| a.name.clone())
                    .collect::<Vec<Option<String>>>()
            }))
            .bind(anilist_entries[anilist_index].studios.as_ref().map(|a| {
                a.nodes
                    .iter()
                    .map(|a| a.site_url.clone())
                    .collect::<Vec<Option<URL>>>()
            }))
            .bind(
                anilist_entries[anilist_index]
                    .tags
                    .as_ref()
                    .map(|a| a.iter().map(|a| a.id.clone()).collect::<Vec<TagID>>()),
            )
            .bind(anilist_entries[anilist_index].tags.as_ref().map(|a| {
                a.iter()
                    .map(|a| a.name.clone())
                    .collect::<Vec<Option<String>>>()
            }))
            .bind(
                anilist_entries[anilist_index]
                    .trailer
                    .as_ref()
                    .map(|a| a.id.clone()),
            )
            .bind(
                anilist_entries[anilist_index]
                    .trailer
                    .as_ref()
                    .map(|a| a.site.clone()),
            )
            .bind(
                anilist_entries[anilist_index]
                    .trailer
                    .as_ref()
                    .map(|a| a.thumbnail.clone()),
            )
            .bind(anilist_entries[anilist_index].season.as_ref().map(|a| a.clone() as i16))
            .bind(anilist_entries[anilist_index].id)
            .execute(&database.pool)
            .await
            .unwrap(); // 'id' is the identifier for the specific anime entry you're updating

            simple_entry_index += 1;
            anilist_index += 1;
        } else if AnilistID::from(entries[simple_entry_index].anilist_id.unwrap())
            < anilist_entries[anilist_index].id
        {
            simple_entry_index += 1;
        } else {
            anilist_index += 1;
        }
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

    //migrate_database(&shared_state.database).await;

    let allowed_origins = [
        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:5173".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:8000".parse::<HeaderValue>().unwrap(),
        "http://localhost:8000".parse::<HeaderValue>().unwrap(),
        format!("http://{}:5173", &shared_state.ip)
            .parse::<HeaderValue>()
            .unwrap(),
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
                .allow_headers([AUTHORIZATION, ACCEPT]),
        )
        .with_state(shared_state.clone()); // Enable CORS

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}
