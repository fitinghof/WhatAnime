use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize)]
pub struct SpotifyToken {
    pub access_token: String,
    // pub token_type: String,
    // pub scope: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
}

// {"error":"unsupported_grant_type","error_description":"grant_type parameter is missing"}
#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct SpotifyTokenError {
    pub error: String,
    pub error_description: String,
}
#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum SpotifyTokenResponse {
    Token(SpotifyToken),
    Error(SpotifyTokenError),
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Device {
    pub id: Option<String>,
    pub is_active: bool,
    pub is_private_session: bool,
    pub is_restricted: bool,
    pub name: String,
    pub r#type: String,
    pub volume_percent: Option<u32>,
    pub supports_volume: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ExternalUrls {
    pub spotify: String,
}
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Context {
    pub r#type: String,
    pub href: String,
    pub external_urls: ExternalUrls,
    pub uri: String,
}
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Image {
    pub url: String,
    pub height: u32,
    pub width: u32,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Restrictions {
    pub reason: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct SimplifiedArtist {
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub uri: String,
}
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Album {
    pub album_type: String,
    pub total_tracks: u32,
    pub available_markets: Vec<String>,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
    pub restrictions: Option<Restrictions>,
    pub r#type: String,
    pub uri: String,
    pub artists: Vec<SimplifiedArtist>,
}
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ExternalIds {
    pub isrc: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
}
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct LinkedFrom {}
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct TrackObject {
    pub album: Album,
    pub artists: Vec<SimplifiedArtist>,
    pub available_markets: Vec<String>,
    pub disc_number: u32,
    pub duration_ms: u64,
    pub explicit: bool,
    pub external_ids: ExternalIds,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub is_local: bool,
    pub is_playable: Option<bool>,
    pub linked_from: Option<LinkedFrom>,
    pub restrictions: Option<Restrictions>,
    pub name: String,
    pub popularity: u32,
    pub preview_url: Option<String>, // depreciated
    pub track_number: u32,
    pub r#type: String,
    pub uri: String,
}
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct EpisodeObject {
    somethingthatmostcertaintlyaintthere: String, // Litteraly dont care
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Item {
    TrackObject(TrackObject),
    EpisodeObject(EpisodeObject),
}
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Actions {
    pub interrupting_playback: Option<bool>,
    pub pausing: Option<bool>,
    pub resuming: Option<bool>,
    pub seeking: Option<bool>,
    pub skipping_next: Option<bool>,
    pub skipping_prev: Option<bool>,
    pub toggling_repeat_context: Option<bool>,
    pub toggling_shuffle: Option<bool>,
    pub toggling_repeat_track: Option<bool>,
    pub transfering_playback: Option<bool>,
}
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CurrentlyPlayingResponse {
    pub device: Option<Device>,
    pub repeat_state: Option<String>,
    pub shuffle_state: Option<String>,
    pub context: Option<Context>,
    pub timestamp: u64,
    pub progress_ms: u64,
    pub is_playing: Option<bool>,
    pub item: Item,
    pub currently_playing_type: Option<String>,
    pub actions: Option<Actions>,
}

pub enum CurrentlyPlayingResponses {
    Playing(CurrentlyPlayingResponse),
    NotPlaying,
    BadToken,
    Ratelimited,
}

#[derive(Deserialize, Serialize)]
pub struct SpotifyUser {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub id: String,
}
