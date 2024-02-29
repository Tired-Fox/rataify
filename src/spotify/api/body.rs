use serde::Serialize;

/// Offset used inside of `POST /me/player/play`
///
/// ```
/// use rataify::spotify::api::body::Offset;
/// let _ = Offset::Position(0);
/// let _ = Offset::Uri("spotify:track:4iV5W9uYEdYUVa79Axb7Rh".to_string());
/// ```
///
/// ```json
/// { "position": 0 }
/// { "uri": "spotify:track:4iV5W9uYEdYUVa79Axb7Rh" }
/// ```
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Offset {
    Position(u32),
    Uri(String),
}

impl Default for Offset {
    fn default() -> Self {
        Offset::Position(0)
    }
}

/// `POST /me/player/play`
///
/// ```
/// use rataify::spotify::api::body::{StartPlayback, Offset};
/// let _ = StartPlayback {
///     position: 0,
///     context_uri: Some("spotify:album:4iV5W9uYEdYUVa79Axb7Rh".to_string()),
///     offset: Some(Offset::Position(1)),
///     uris: vec!["spotify:album:4iV5W9uYEdYUVa79Axb7Rh".to_string()],
/// };
/// ```
/// ```json
/// {
///     "position_ms": 0,
///     "context_uri": "spotify:album:4iV5W9uYEdYUVa79Axb7Rh",
///     "offset": {
///         "position": 1
///     },
///     "uris": [
///         "spotify:album:4iV5W9uYEdYUVa79Axb7Rh"
///     ]
/// }
/// ```
#[derive(Default, Serialize)]
pub struct StartPlayback {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_uri: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub uris: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<Offset>,

    #[serde(rename = "position_ms")]
    pub position: u32
}

/// `PUT /me/player`
///
/// ```
/// use rataify::spotify::api::body::TransferPlayback;
/// let _ = TransferPlayback {
///     device_ids: ["spotify:track:4iV5W9uYEdYUVa79Axb7Rh".to_string()],
///     play: Some(true),
/// };
/// ```
///
/// ```json
/// {
///     "device_ids": [
///         "spotify:track:4iV5W9uYEdYUVa79Axb7Rh"
///     ],
///     "play": true
/// }
/// ```
#[derive(Default, Serialize)]
pub struct TransferPlayback {
    pub device_ids: [String; 1],

    #[serde(skip_serializing_if = "Option::is_none")]
    pub play: Option<bool>,
}