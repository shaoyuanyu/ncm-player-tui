use serde::{Deserialize, Serialize};

pub enum AppMode {
    Normal,
    CommandEntry,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum ScreenEnum {
    Main,
    // Playlist,
    Login,
    Help,
}
