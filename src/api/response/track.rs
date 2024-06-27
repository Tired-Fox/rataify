use crate::impl_paged;
use chrono::{DateTime, Duration, Local};
use serde::Deserialize;
use crate::api::Uri;

use super::{
    deserialize_added_at, deserialize_duration, deserialize_duration_seconds,
    deserialize_timestamp, Album, ExternalIds, ExternalUrls, IntoUserTopItemType, Restrictions,
    SimplifiedArtist,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Track {
    /// The album on which the track appears. The album object includes a link in href to full information about the album.
    pub album: Album,
    /// The artists who performed the track. Each artist object includes a link in href to more detailed information about the artist.
    pub artists: Vec<SimplifiedArtist>,
    /// A list of countries in which the track can be played, identified by their [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2).
    #[serde(default = "Vec::new")]
    pub available_markets: Vec<String>,
    /// The disc number (usually 1 unless the album consists of more than one disc).
    pub disc_number: u8,
    /// The track length in milliseconds.
    #[serde(rename = "duration_ms", deserialize_with = "deserialize_duration")]
    pub duration: chrono::Duration,
    /// Whether or not the track has explicit lyrics ( true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// Known external IDs for the track.
    pub external_ids: ExternalIds,
    /// Known external URLs for this track.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the track.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the track.
    pub id: String,
    /// Part of the response when [Track Relinking](https://developer.spotify.com/documentation/web-api/concepts/track-relinking) is applied. If true, the track is playable in the given market. Otherwise false.
    #[serde(default = "bool::default")]
    pub is_playable: bool,
    /// Part of the response when [Track Relinking](https://developer.spotify.com/documentation/web-api/concepts/track-relinking) is applied, and the requested track has been replaced with different track. The track in the linked_from object contains information about the originally requested track.
    pub linked_from: Option<Box<Track>>,
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,
    /// The name of the track.
    pub name: String,
    /// A link to a 30 second preview (MP3 format) of the track.
    pub preview_url: Option<String>,
    /// The number of the track. If an album has several discs, the track number is the number on the specified disc.
    pub track_number: u8,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the track.
    pub uri: Uri,
    /// Whether or not the track is from a local file.
    pub is_local: bool,
}
impl IntoUserTopItemType for Track {
    fn into_top_item_type() -> &'static str {
        "tracks"
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SimplifiedTrack {
    /// The artists who performed the track. Each artist object includes a link in href to more detailed information about the artist.
    pub artists: Vec<SimplifiedArtist>,
    /// A list of the countries in which the track can be played, identified by their [ISO 3166-1 alpha-2 country code](http://en.wikipedia.org/wiki/ISO_3166-1_alpha-2).
    #[serde(default = "Vec::new")]
    pub available_markets: Vec<String>,
    /// The disc number (usually 1 unless the album consists of more than one disc).
    pub disc_number: u8,
    /// The track length in milliseconds.
    #[serde(rename = "duration_ms", deserialize_with = "deserialize_duration")]
    pub duration: chrono::Duration,
    /// Whether or not the track has explicit lyrics ( true = yes it does; false = no it does not OR unknown).
    pub explicit: bool,
    /// Known external URLs for this track.
    pub external_urls: ExternalUrls,
    /// A link to the Web API endpoint providing full details of the track.
    pub href: String,
    /// The [Spotify ID](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the track.
    pub id: String,
    /// Part of the response when [Track Relinking](https://developer.spotify.com/documentation/web-api/concepts/track-relinking) is applied. If true, the track is playable in the given market. Otherwise false.
    #[serde(default = "bool::default")]
    pub is_playable: bool,
    /// Part of the response when [Track Relinking](https://developer.spotify.com/documentation/web-api/concepts/track-relinking) is applied, and the requested track has been replaced with different track. The track in the linked_from object contains information about the originally requested track.
    pub linked_from: Option<Box<Track>>,
    /// Included in the response when a content restriction is applied.
    pub restrictions: Option<Restrictions>,
    /// The name of the track.
    pub name: String,
    /// A link to a 30 second preview (MP3 format) of the track.
    pub preview_url: Option<String>,
    /// The number of the track. If an album has several discs, the track number is the number on the specified disc.
    pub track_number: u8,
    /// The [Spotify URI](https://developer.spotify.com/documentation/web-api/concepts/spotify-uris-ids) for the track.
    pub uri: Uri,
    /// Whether or not the track is from a local file.
    pub is_local: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SavedTrack {
    /// The date and time the track was saved. Timestamps are returned in ISO 8601 format as Coordinated Universal Time (UTC) with a zero offset: YYYY-MM-DDTHH:MM:SSZ.
    #[serde(deserialize_with = "deserialize_added_at")]
    pub added_at: DateTime<Local>,
    pub track: Track,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SavedTracks {
    /// A link to the Web API endpoint returning the full result of the request
    pub href: String,
    /// The maximum number of items in the response (as set in the query or by default).
    pub limit: usize,
    /// URL to the next page of items. ( `null` if none)
    pub next: Option<String>,
    /// The offset of the items returned (as set in the query or by default)
    pub offset: usize,
    /// URL to the previous page of items. ( `null` if none)
    pub previous: Option<String>,
    /// The total number of items available to return
    pub total: usize,
    /// The user's saved tracks
    pub items: Vec<SavedTrack>,
}
impl_paged!(SavedTracks<SavedTrack>);

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AudioFeatures {
    /// A confidence measure from 0.0 to 1.0 of whether the track is acoustic. 1.0 represents high confidence the track is acoustic.
    pub acousticness: f32,
    /// A URL to access the full audio analysis of this track. An access token is required to access this data.
    pub analysis_url: String,
    /// Danceability describes how suitable a track is for dancing based on a combination of musical elements including tempo, rhythm stability, beat strength, and overall regularity. A value of 0.0 is least danceable and 1.0 is most danceable.
    pub danceability: f32,
    /// The duration of the track.
    #[serde(rename = "duration_ms", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    /// Energy is a measure from 0.0 to 1.0 and represents a perceptual measure of intensity and activity. Typically, energetic tracks feel fast, loud, and noisy. For example, death metal has high energy, while a Bach prelude scores low on the scale. Perceptual features contributing to this attribute include dynamic range, perceived loudness, timbre, onset rate, and general entropy.
    pub energy: f32,
    /// The Spotify ID for the track.
    pub id: String,
    /// Predicts whether a track contains no vocals. "Ooh" and "aah" sounds are treated as instrumental in this context. Rap or spoken word tracks are clearly "vocal". The closer the instrumentalness value is to 1.0, the greater likelihood the track contains no vocal content. Values above 0.5 are intended to represent instrumental tracks, but confidence is higher as the value approaches 1.0.
    pub instrumentalness: f32,
    /// The key the track is in. Integers map to pitches using standard Pitch Class notation. E.g. 0 = C, 1 = C♯/D♭, 2 = D, and so on. If no key was detected, the value is -1.
    pub key: i8,
    /// Detects the presence of an audience in the recording. Higher liveness values represent an increased probability that the track was performed live. A value above 0.8 provides strong likelihood that the track is live.
    pub liveness: f32,
    /// The overall loudness of a track in decibels (dB). Loudness values are averaged across the entire track and are useful for comparing relative loudness of tracks. Loudness is the quality of a sound that is the primary psychological correlate of physical strength (amplitude). Values typically range between -60 and 0 db.
    pub loudness: f32,
    /// Mode indicates the modality (major or minor) of a track, the type of scale from which its melodic content is derived. Major is represented by 1 and minor is 0.
    pub mode: u8,
    /// Speechiness detects the presence of spoken words in a track. The more exclusively speech-like the recording (e.g. talk show, audio book, poetry), the closer to 1.0 the attribute value. Values above 0.66 describe tracks that are probably made entirely of spoken words. Values between 0.33 and 0.66 describe tracks that may contain both music and speech, either in sections or layered, including such cases as rap music. Values below 0.33 most likely represent music and other non-speech-like tracks.
    pub speechiness: f32,
    /// The overall estimated tempo of a track in beats per minute (BPM). In musical terminology, tempo is the speed or pace of a given piece and derives directly from the average beat duration.
    pub tempo: f32,
    /// An estimated time signature. The time signature (meter) is a notational convention to specify how many beats are in each bar (or measure). The time signature ranges from 3 to 7 indicating time signatures of "3/4", to "7/4".
    pub time_signature: u8,
    /// A link to the Web API endpoint providing full details of the track.
    pub track_href: String,
    /// The Spotify URI for the track.
    pub uri: String,
    /// A measure from 0.0 to 1.0 describing the musical positiveness conveyed by a track. Tracks with high valence sound more positive (e.g. happy, cheerful, euphoric), while tracks with low valence sound more negative (e.g. sad, depressed, angry).
    pub valence: f32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AudioAnalysisMeta {
    /// The version of the Analyzer used to analyze this track.
    pub analyzer_version: String,
    /// The platform used to read the track's audio data.
    pub platform: String,
    /// A detailed status code for this track. If analysis data is missing, this code may explain why.
    pub detailed_status: String,
    /// The return code of the analyzer process. 0 if successful, 1 if any errors occurred.
    pub status_code: u8,
    /// The Unix timestamp (in seconds) at which this track was analyzed.
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub timestamp: DateTime<Local>,
    /// The amount of time taken to analyze this track.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub analysis_time: Duration,
    /// The method used to read the track's audio data.
    pub input_process: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AudioAnalysisTrack {
    /// The exact number of audio samples analyzed from this track. See also analysis_sample_rate.
    pub num_samples: usize,
    /// Length of the track in seconds.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub duration: Duration,
    /// This field will always contain the empty string.
    pub sample_md5: String,
    /// An offset to the start of the region of the track that was analyzed. (As the entire track is analyzed, this should always be 0.)
    #[serde(default)]
    pub offset_seconds: usize,
    /// The length of the region of the track was analyzed, if a subset of the track was analyzed. (As the entire track is analyzed, this should always be 0.)
    #[serde(default)]
    pub window_seconds: usize,
    /// The sample rate used to decode and analyze this track. May differ from the actual sample rate of this track available on Spotify.
    pub analysis_sample_rate: usize,
    /// The number of channels used for analysis. If 1, all channels are summed together to mono before analysis.
    pub analysis_channels: u8,
    /// The time, in seconds, at which the track's fade-in period ends. If the track has no fade-in, this will be 0.0.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub end_of_fade_in: Duration,
    /// The time, in seconds, at which the track's fade-out period starts. If the track has no fade-out, this should match the track's length.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub start_of_fade_out: Duration,
    /// The overall loudness of a track in decibels (dB). Loudness values are averaged across the entire track and are useful for comparing relative loudness of tracks. Loudness is the quality of a sound that is the primary psychological correlate of physical strength (amplitude). Values typically range between -60 and 0 db.
    pub loudness: f32,
    /// The overall estimated tempo of a track in beats per minute (BPM). In musical terminology, tempo is the speed or pace of a given piece and derives directly from the average beat duration.
    pub tempo: f32,
    /// The confidence, from 0.0 to 1.0, of the reliability of the tempo.
    pub tempo_confidence: f32,
    /// An estimated time signature. The time signature (meter) is a notational convention to specify how many beats are in each bar (or measure). The time signature ranges from 3 to 7 indicating time signatures of "3/4", to "7/4".
    pub time_signature: u8,
    /// The confidence, from 0.0 to 1.0, of the reliability of the time_signature.
    pub time_signature_confidence: f32,
    /// The key the track is in. Integers map to pitches using standard Pitch Class notation. E.g. 0 = C, 1 = C♯/D♭, 2 = D, and so on. If no key was detected, the value is -1.
    pub key: i8,
    /// The confidence, from 0.0 to 1.0, of the reliability of the key.
    pub key_confidence: f32,
    /// Mode indicates the modality (major or minor) of a track, the type of scale from which its melodic content is derived. Major is represented by 1 and minor is 0.
    pub mode: u8,
    /// The confidence, from 0.0 to 1.0, of the reliability of the mode.
    pub mode_confidence: f32,
    /// An [Echo Nest Musical Fingerprint (ENMFP)](https://academiccommons.columbia.edu/doi/10.7916/D8Q248M4) codestring for this track.
    pub codestring: String,
    /// A version number for the Echo Nest Musical Fingerprint format used in the codestring field.
    pub code_version: f32,
    /// An [EchoPrint](https://github.com/spotify/echoprint-codegen) codestring for this track.
    pub echoprintstring: String,
    /// A version number for the EchoPrint format used in the echoprintstring field.
    pub echoprint_version: f32,
    /// A [Synchstring](https://github.com/echonest/synchdata) for this track.
    pub synchstring: String,
    /// A version number for the Synchstring used in the synchstring field.
    pub synch_version: f32,
    /// A Rhythmstring for this track. The format of this string is similar to the Synchstring.
    pub rhythmstring: String,
    /// A version number for the Rhythmstring used in the rhythmstring field.
    pub rhythm_version: f32,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct Bar {
    /// The starting point (in seconds) of the time interval.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub start: Duration,
    /// he duration (in seconds) of the time interval.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub duration: Duration,
    /// The confidence, from 0.0 to 1.0, of the reliability of the interval.
    pub confidence: f32,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct Beat {
    /// The starting point (in seconds) of the time interval.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub start: Duration,
    /// he duration (in seconds) of the time interval.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub duration: Duration,
    /// The confidence, from 0.0 to 1.0, of the reliability of the interval.
    pub confidence: f32,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct Section {
    /// The starting point (in seconds) of the time interval.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub start: Duration,
    /// he duration (in seconds) of the time interval.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub duration: Duration,
    /// The confidence, from 0.0 to 1.0, of the reliability of the interval.
    pub confidence: f32,
    /// The overall loudness of the section in decibels (dB). Loudness values are useful for comparing relative loudness of sections within tracks.
    pub loudness: f32,
    /// The overall estimated tempo of the section in beats per minute (BPM). In musical terminology, tempo is the speed or pace of a given piece and derives directly from the average beat duration.
    pub tempo: f32,
    /// The confidence, from 0.0 to 1.0, of the reliability of the tempo. Some tracks contain tempo changes or sounds which don't contain tempo (like pure speech) which would correspond to a low value in this field.
    pub tempo_confidence: f32,
    /// The estimated overall key of the section. The values in this field ranging from 0 to 11 mapping to pitches using standard Pitch Class notation (E.g. 0 = C, 1 = C♯/D♭, 2 = D, and so on). If no key was detected, the value is -1.
    pub key: i8,
    /// The confidence, from 0.0 to 1.0, of the reliability of the key. Songs with many key changes may correspond to low values in this field.
    pub key_confidence: f32,
    /// Indicates the modality (major or minor) of a section, the type of scale from which its melodic content is derived. This field will contain a 0 for "minor", a 1 for "major", or a -1 for no result. Note that the major key (e.g. C major) could more likely be confused with the minor key at 3 semitones lower (e.g. A minor) as both keys carry the same pitches.
    pub mode: u8,
    /// The confidence, from 0.0 to 1.0, of the reliability of the mode.
    pub mode_confidence: f32,
    /// An estimated time signature. The time signature (meter) is a notational convention to specify how many beats are in each bar (or measure). The time signature ranges from 3 to 7 indicating time signatures of "3/4", to "7/4".
    pub time_signature: u8,
    /// The confidence, from 0.0 to 1.0, of the reliability of the time_signature.
    pub time_signature_confidence: f32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Segment {
    /// The starting point (in seconds) of the segment.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub start: Duration,
    /// The duration (in seconds) of the segment.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub duration: Duration,
    /// The confidence, from 0.0 to 1.0, of the reliability of the segmentation. Segments of the song which are difficult to logically segment (e.g: noise) may correspond to low values in this field.
    pub confidence: f32,
    /// The onset loudness of the segment in decibels (dB). Combined with loudness_max and loudness_max_time, these components can be used to describe the "attack" of the segment.
    pub loudness_start: f32,
    /// The peak loudness of the segment in decibels (dB). Combined with loudness_start and loudness_max_time, these components can be used to describe the "attack" of the segment.
    pub loudness_max: f32,
    /// The segment-relative offset of the segment peak loudness in seconds. Combined with loudness_start and loudness_max, these components can be used to desctibe the "attack" of the segment.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub loudness_max_time: Duration,
    /// The offset loudness of the segment in decibels (dB). This value should be equivalent to the loudness_start of the following segment.
    pub loudness_end: f32,
    /// Pitch content is given by a “chroma” vector, corresponding to the 12 pitch classes C, C#, D to B, with values ranging from 0 to 1 that describe the relative dominance of every pitch in the chromatic scale. For example a C Major chord would likely be represented by large values of C, E and G (i.e. classes 0, 4, and 7).
    ///
    /// Vectors are normalized to 1 by their strongest dimension, therefore noisy sounds are likely represented by values that are all close to 1, while pure tones are described by one value at 1 (the pitch) and others near 0. As can be seen below, the 12 vector indices are a combination of low-power spectrum values at their respective pitch frequencies.
    pub pitches: Vec<f32>,
    /// Timbre is the quality of a musical note or sound that distinguishes different types of musical instruments, or voices. It is a complex notion also referred to as sound color, texture, or tone quality, and is derived from the shape of a segment’s spectro-temporal surface, independently of pitch and loudness. The timbre feature is a vector that includes 12 unbounded values roughly centered around 0. Those values are high level abstractions of the spectral surface, ordered by degree of importance.
    ///
    /// For completeness however, the first dimension represents the average loudness of the segment; second emphasizes brightness; third is more closely correlated to the flatness of a sound; fourth to sounds with a stronger attack; etc. See an image below representing the 12 basis functions (i.e. template segments). timbre basis functions
    ///
    /// The actual timbre of the segment is best described as a linear combination of these 12 basis functions weighted by the coefficient values: timbre = c1 x b1 + c2 x b2 + ... + c12 x b12, where c1 to c12 represent the 12 coefficients and b1 to b12 the 12 basis functions as displayed below. Timbre vectors are best used in comparison with each other.
    pub timbre: Vec<f32>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct Tatum {
    /// The starting point of the time interval.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub start: Duration,
    /// he duration of the time interval.
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub duration: Duration,
    /// The confidence, from 0.0 to 1.0, of the reliability of the interval.
    pub confidence: f32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AudioAnalysis {
    pub meta: AudioAnalysisMeta,
    pub track: AudioAnalysisTrack,
    /// The time intervals of the bars throughout the track. A bar (or measure) is a segment of time defined as a given number of beats.
    pub bars: Vec<Bar>,
    /// The time intervals of beats throughout the track. A beat is the basic time unit of a piece of music; for example, each tick of a metronome. Beats are typically multiples of tatums.
    pub beats: Vec<Beat>,
    /// Sections are defined by large variations in rhythm or timbre, e.g. chorus, verse, bridge, guitar solo, etc. Each section contains its own descriptions of tempo, key, mode, time_signature, and loudness.
    pub sections: Vec<Section>,
    /// Sections are defined by large variations in rhythm or timbre, e.g. chorus, verse, bridge, guitar solo, etc. Each section contains its own descriptions of tempo, key, mode, time_signature, and loudness.
    pub segments: Vec<Segment>,
    /// A tatum represents the lowest regular pulse train that a listener intuitively infers from the timing of perceived musical events (segments).
    pub tatums: Vec<Tatum>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct RecommendationSeed {
    /// The number of tracks available after min_* and max_* filters have been applied.
    #[serde(rename = "afterFilteringSize")]
    pub after_filtering_size: usize,
    /// The number of available seed tracks after relinking for regional availability.
    #[serde(rename = "afterRelinkingSize")]
    pub after_relinking_size: usize,
    /// A link to the full track or artist data for this seed. For tracks this will be a link to a Track Object. For artists a link to an Artist Object. For genre seeds, this value will be null.
    pub href: String,
    /// The id used to select this seed. This will be the same as the string used in the seed_artists, seed_tracks or seed_genres parameter.
    pub id: String,
    /// The initial pool size for this recommendation seed.
    #[serde(rename = "initialPoolSize")]
    pub initial_pool_size: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Recommendations {
    /// An array of recommendation seed objects.
    pub seeds: Vec<RecommendationSeed>,
    /// An array of track object (simplified) ordered according to the parameters supplied.
    pub tracks: Vec<SimplifiedTrack>,
}
