use chrono::{DateTime, Utc};
use serde::de::Error;
use serde::Deserialize;

use crate::model::paginate::{Paginate, parse_pagination};
use crate::model::player::Track;

#[derive(Debug, Deserialize)]
pub struct PagedItem {
    pub track: Track,
    pub added_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PagedTracks {
    pub href: String,
    pub limit: usize,
    pub offset: usize,
    /// Link to get next page of tracks
    #[serde(deserialize_with = "parse_pagination")]
    pub next: Option<Paginate>,
    /// Link to get previous page of tracks
    #[serde(deserialize_with = "parse_pagination")]
    pub previous: Option<Paginate>,
    pub total: Option<u64>,
    pub items: Vec<PagedItem>,
}

impl PagedTracks {
    /// Index of the last track in the items.
    ///
    /// This value is based on the offset and the page size of the items.
    pub fn last_index(&self) -> usize {
        self.offset + self.limit
    }
}

#[derive(Debug, Deserialize)]
pub struct Recommendations {
    pub seeds: Vec<RecommendationSeed>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RecommendationSeed {
    #[serde(rename = "afterFilteringSize")]
    pub after_filter_size: u32,
    #[serde(rename = "afterRelinkingSize")]
    pub after_relinking_size: u32,
    pub href: String,
    pub id: String,
    #[serde(rename = "initialPoolSize")]
    pub initial_pool_size: u32,
    #[serde(rename = "type")]
    pub _type: String,
}

fn s_to_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: serde::Deserializer<'de>,
{
    let ms: i64 = Deserialize::deserialize(deserializer)?;
    DateTime::from_timestamp(ms, 0).ok_or(Error::custom("Invalid timestamp"))
}

/// Audio Features of a track
#[derive(Debug, Deserialize, Clone)]
pub struct AudioFeatures {
    /// 0.0 - 1.0 confidence measure of whether the track is acoustic
    pub acousticness: f64,
    pub analysis_url: String,
    pub danceability: f64,
    pub duration_ms: u64,
    pub energy: f64,
    pub id: String,
    pub instrumentalness: f64,
    pub key: u64,
    pub liveness: f64,
    pub loudness: f64,
    pub mode: u64,
    pub speechiness: f64,
    pub tempo: f64,
    pub time_signature: u64,
    pub track_href: String,
    /// `audio_features`
    #[serde(rename = "type")]
    pub _type: String,
    pub uri: String,
    pub valence: f64,

}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioAnalysisMeta {
    pub analyzer_version: String,
    pub platform: String,
    pub detailed_status: String,
    pub status_code: u64,
    #[serde(deserialize_with = "s_to_datetime")]
    pub timestamp: DateTime<Utc>,
    pub analysis_time: f64,
    pub input_process: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioAnalysisTrack {
    pub num_samples: u64,
    pub duration: f64,
    pub sample_md5: String,
    pub offset_seconds: f64,
    pub window_seconds: f64,
    pub analysis_sample_rate: u64,
    pub analysis_channels: u64,
    pub end_of_fade_in: f64,
    pub start_of_fade_out: f64,
    pub loudness: f64,
    pub tempo: f64,
    pub tempo_confidence: f64,
    pub time_signature: u64,
    pub time_signature_confidence: f64,
    pub key: u64,
    pub key_confidence: f64,
    pub mode: u64,
    pub mode_confidence: f64,
    pub codestring: String,
    pub code_version: f64,
    pub echoprintstring: String,
    pub echoprint_version: f64,
    pub synchstring: String,
    pub synch_version: f64,
    pub rhythmstring: String,
    pub rhythm_version: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioAnalysisSegments {
    pub start: f64,
    pub duration: f64,
    pub confidence: f64,
    pub loudness_start: f64,
    pub loudness_max: f64,
    pub loudness_end: f64,
    pub pitches: Vec<f64>,
    pub timbre: Vec<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioAnalysisInterval {
    pub start: f64,
    pub duration: f64,
    pub confidence: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioAnalysisSections {
    pub start: f64,
    pub duration: f64,
    pub confidence: f64,
    pub loudness: f64,
    pub tempo: f64,
    pub tempo_confidence: f64,
    pub key: u64,
    pub key_confidence: f64,
    pub mode: u64,
    pub mode_confidence: f64,
    pub time_signature: u64,
    pub time_signature_confidence: f64,
}

/// Audio Analysis
#[derive(Debug, Deserialize, Clone)]
pub struct AudioAnalysis {
    pub meta: AudioAnalysisMeta,
    pub track: AudioAnalysisTrack,
    pub bars: Vec<AudioAnalysisInterval>,
    pub beats: Vec<AudioAnalysisInterval>,
    pub sections: Vec<AudioAnalysisSections>,
    pub segments: Vec<AudioAnalysisSegments>,
    pub tatums: Vec<AudioAnalysisInterval>,
}
