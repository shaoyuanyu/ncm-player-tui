use serde::{Deserialize, Serialize};

pub enum AppMode {
    Normal,
    CommandLine,
    Search(Vec<String>),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum ScreenEnum {
    Main,
    Playlists,
    Login,
    Help,
    Launch,
}
