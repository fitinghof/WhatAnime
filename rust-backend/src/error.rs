// error.rs

use axum::response::IntoResponse;
use axum::http::StatusCode;

pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub enum Error {
    Unauthorized,
    BadRequest{url: String, status_code: axum::http::StatusCode},
    NotASong,
    ReqwestError(reqwest::Error),
    TowerError(tower_sessions::session::Error),
    ParseError(String),
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
