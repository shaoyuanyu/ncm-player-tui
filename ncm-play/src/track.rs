use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Track {
    /// Track name from metadata, if no name is present, filename will be
    /// displayed instead
    pub title: Option<String>,

    /// Artist name from metadata if present
    pub artist: String,

    /// Album name from metadata if present
    pub album: String,

    /// Year from metadata if present
    pub year: Option<u32>,

    /// Track number if present
    pub number: Option<u32>,

    /// Track duration
    pub length: Duration,

    /// Path to the audio file
    pub file_path: String,
}
