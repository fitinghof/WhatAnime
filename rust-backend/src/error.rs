// error.rs
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sqlx::migrate::MigrateError;

pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
#[derive(Debug)]
pub enum Error {
    BadOAuth,
    NotASong,
    NotImplemented,
    BadRequest {
        url: String,
        status_code: axum::http::StatusCode,
    },
    ReqwestError(reqwest::Error),
    TowerError(tower_sessions::session::Error),
    ParseError(String),
    SqlxError(sqlx::Error),
    MigrateError(MigrateError),
    // SessionError(tower_sessions_core::session::Error)
}

impl From<MigrateError> for Error {
    fn from(value: MigrateError) -> Self {
        Self::MigrateError(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::SqlxError(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

impl From<tower_sessions::session::Error> for Error {
    fn from(value: tower_sessions::session::Error) -> Self {
        Self::TowerError(value)
    }
}
