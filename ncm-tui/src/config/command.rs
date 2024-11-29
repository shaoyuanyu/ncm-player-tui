use crate::config::ScreenEnum;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub enum Command {
    Quit,
    GotoScreen(ScreenEnum),
    EnterCommand,
    Logout,
    PlayOrPause,
    SetVolume(f64),
    //
    Down,
    Up,
    NextPanel,
    PrevPanel,
    Esc,
    Play,
    //
    Stop,
    TogglePlay,
    ToggleShuffle,
    ToggleRepeat,
    QueueAndPlay,
    GotoTop,
    GotoBottom,

    NewPlaylist(Option<String>),
    PlaylistAdd,
    SelectPlaylist,
    PrevTrack,
    NextTrack,

    AddPath(PathBuf),
    PlayTrack(PathBuf),

    Nop,
}

impl Command {
    pub fn parse(cmd_str: &str) -> Result<Self> {
        let mut tokens = cmd_str.split_whitespace();

        match tokens.next() {
            Some("q" | "quit" | "exit") => Ok(Self::Quit),
            Some("s" | "shuf" | "shuffle") => Ok(Self::ToggleShuffle),
            Some("r" | "rep" | "repeat") => Ok(Self::ToggleRepeat),
            Some("screen") => match tokens.next() {
                Some("1" | "main") => Ok(Self::GotoScreen(ScreenEnum::Main)),
                // Some("2" | "playlist" | "playlists") => Ok(Self::GotoScreen(ScreenEnum::Playlists)),
                Some("0" | "help") => Ok(Self::GotoScreen(ScreenEnum::Help)),
                Some(other) => Err(anyhow!("screen: Invalid screen identifier: {}", other)),
                None => Err(anyhow!("screen: Missing argument SCREEN_ID")),
            },
            Some("h" | "help") => Ok(Self::GotoScreen(ScreenEnum::Help)),
            Some("a" | "add") => match cmd_str.split_once(' ') {
                Some((_, p)) => Ok(Self::AddPath(p.into())),
                None => Err(anyhow!("add: Missing argument PATH")),
            },
            Some("n" | "new-playlist") => match cmd_str.split_once(' ') {
                Some((_, name)) => Ok(Self::NewPlaylist(Some(name.into()))),
                None => Ok(Self::NewPlaylist(None)),
            },
            Some("p" | "play") => match cmd_str.split_once(' ') {
                Some((_, path)) => Ok(Self::PlayTrack(path.into())),
                None => Err(anyhow!("play: Missing argument PATH")),
            },
            Some("l" | "login") => Ok(Self::GotoScreen(ScreenEnum::Login)),
            Some("logout") => Ok(Self::Logout),
            Some("vol" | "volume") => match tokens.next() {
                Some(num) => {
                    if let Ok(vol) = num.parse::<f64>() {
                        Ok(Self::SetVolume(vol / 100.0))
                    } else {
                        Err(anyhow!("volume: Invalid argument NUMBER"))
                    }
                }
                _ => Err(anyhow!("volume: Missing argument NUMBER")),
            },
            Some(other) => Err(anyhow!("Invalid command: {}", other)),
            None => Ok(Self::Nop),
        }
    }
}
