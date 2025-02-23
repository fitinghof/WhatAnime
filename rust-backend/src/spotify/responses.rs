use serde::Deserialize;

#[derive(serde::Deserialize)]
pub struct SpotifyTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    pub expires_in: u64,
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct Device {
    pub id: Option<String>,
    pub is_active: bool,
    pub is_private_session: bool,
    pub is_restricted: bool,
    pub name: String,
    pub r#type: String,
    pub volume_percent: Option<i32>,
    pub supports_volume: bool,
}

#[derive(Deserialize)]
pub struct ExternalUrls {
    pub spotify: String
}

#[derive(Deserialize)]
pub struct Context {
    pub r#type: String,
    pub href: String,
    pub external_urls: ExternalUrls,
    pub uri: String,
}

#[derive(Deserialize)]
pub struct Image {
    pub url: String,
    pub height: i32,
    pub width: i32,
}

#[derive(Deserialize)]
pub struct Restrictions {
    pub reason: String
}

#[derive(Deserialize)]
pub struct SimplifiedArtist {
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub uri: String
}

#[derive(Deserialize)]
pub struct Album {
    pub album_type: String,
    pub total_tracks: i32,
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

#[derive(Deserialize)]
pub struct ExternalIds {
    pub isrc: String,
    pub ean: String,
    pub upc: String,
}

#[derive(Deserialize)]
pub struct LinkedFrom {}

#[derive(Deserialize)]
pub struct TrackObject {
    pub album: Album,
    pub artists: Vec<SimplifiedArtist>,
    pub available_markets: Vec<String>,
    pub disc_number: i32,
    pub duration_ms: i32,
    pub explicit: bool,
    pub external_ids: ExternalIds,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub is_playable: Option<bool>,
    pub linked_from: Option<LinkedFrom>,
    pub restrictions: Option<Restrictions>,
    pub name: String,
    pub popularity: i32,
    pub preview_url: String, // depreciated
    pub track_number: i32,
    pub r#type: String,
    pub uri: String,
    pub is_local: bool,
}

#[derive(Deserialize)]
pub struct EpisodeObject {
    // Litteraly dont care
}

#[derive(Deserialize)]
pub enum Item {
    TrackObject(TrackObject),
    EpisodeObject(EpisodeObject),
}

#[derive(Deserialize)]
pub struct Actions {
    pub interrupting_playback: bool,
    pub pausing: bool,
    pub resuming: bool,
    pub seeking: bool,
    pub skipping_next: bool,
    pub skipping_prev: bool,
    pub toggling_repeat_context: bool,
    pub toggling_shuffle: bool,
    pub toggling_repeat_track: bool,
    pub transfering_playback: bool,
}

#[derive(Deserialize)]
pub struct CurrentlyPlayingResponse {
    pub device: Device,
    pub repeat_state: String,
    pub shuffle_state: String,
    pub context: Context,
    pub timestamp: i32,
    pub progress_ms: i32,
    pub is_playing: bool,
    pub item: Item,
    pub currently_playing_type: String,
    pub actions: Actions
}