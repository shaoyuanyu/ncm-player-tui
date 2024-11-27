pub mod track;

use crate::track::Track;
use std::time::Duration;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Repeat {
    On,
    Off,
    One,
}

impl Default for Repeat {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Default)]
pub struct MediaState {
    pub current_track: Option<Track>,
    pub current_track_progress: Option<Duration>,
    pub playing: bool,
    pub stopped: bool,
    pub shuffle: bool,
    pub repeat: Repeat,
}
