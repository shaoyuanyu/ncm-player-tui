use crate::config::Command::SwitchPlayMode;
use crate::config::ScreenEnum;
use anyhow::{anyhow, Result};
use ncm_play::PlayMode;

#[derive(Clone, Debug)]
pub enum Command {
    Quit,
    GotoScreen(ScreenEnum),
    EnterCommand,
    Logout,
    PlayOrPause,
    SetVolume(f64),
    SwitchPlayMode(PlayMode),
    StartPlay,
    NextSong,
    PrevSong,
    SearchForward(Vec<String>),
    SearchBackward(Vec<String>),
    RefreshPlaylist,

    Down,
    Up,
    NextPanel,
    PrevPanel,
    Esc,
    /// Enter，优先执行进入某菜单的功能，无可进入（所选项为单曲）时播放
    EnterOrPlay,
    /// Alt + Enter，优先执行播放功能，所选项为菜单则对其执行 StartPlay
    Play,
    WhereIsThisSong,
    GoToTop,
    GoToBottom,

    Nop,
}

impl Command {
    pub fn parse(cmd_str: &str) -> Result<Self> {
        let mut tokens = cmd_str.split_whitespace();

        match tokens.next() {
            Some("q" | "quit" | "exit") => Ok(Self::Quit),
            Some("screen") => match tokens.next() {
                Some("1" | "main") => Ok(Self::GotoScreen(ScreenEnum::Main)),
                Some("2" | "playlist" | "playlists") => Ok(Self::GotoScreen(ScreenEnum::Playlists)),
                Some("0" | "help") => Ok(Self::GotoScreen(ScreenEnum::Help)),
                Some(other) => Err(anyhow!("screen: Invalid screen identifier: {}", other)),
                None => Err(anyhow!("screen: Missing argument SCREEN_ID")),
            },
            Some("h" | "help") => Ok(Self::GotoScreen(ScreenEnum::Help)),
            Some("l" | "login") => Ok(Self::GotoScreen(ScreenEnum::Login)),
            Some("logout") => Ok(Self::Logout),
            Some("vol" | "volume") => match tokens.next() {
                Some(num) => {
                    if let Ok(vol) = num.parse::<f64>() {
                        Ok(Self::SetVolume(vol / 100.0))
                    } else {
                        Err(anyhow!("volume: Invalid argument NUMBER"))
                    }
                },
                None => Err(anyhow!("volume: Missing argument NUMBER")),
            },
            Some("mute") => Ok(Self::SetVolume(0.0)),
            Some("mode") => match tokens.next() {
                Some("single") => Ok(SwitchPlayMode(PlayMode::Single)),
                Some("sr" | "single-repeat") => Ok(SwitchPlayMode(PlayMode::SingleRepeat)),
                Some("lr" | "list-repeat") => Ok(SwitchPlayMode(PlayMode::ListRepeat)),
                Some("s" | "shuf" | "shuffle") => Ok(SwitchPlayMode(PlayMode::Shuffle)),
                Some(other) => Err(anyhow!("switch: Invalid play mode identifier: {}", other)),
                None => Err(anyhow!("switch: Missing argument PLAY_MODE")),
            },
            Some("next") => Ok(Self::NextSong),
            Some("prev" | "previous") => Ok(Self::PrevSong),
            Some("start") => Ok(Self::StartPlay),
            Some("where") => match tokens.next() {
                Some("this") => Ok(Self::WhereIsThisSong),
                Some(other) => Err(anyhow!("where: Invalid argument '{}'", other)),
                None => Err(anyhow!("where: Missing argument")),
            },
            Some("top") => Ok(Self::GoToTop),
            Some("bottom") => Ok(Self::GoToBottom),
            Some("/") => {
                let mut keywords = Vec::new();
                while let Some(keyword) = tokens.next() {
                    keywords.push(keyword.to_string());
                }
                Ok(Self::SearchForward(keywords))
            },
            Some("?") => {
                let mut keywords = Vec::new();
                while let Some(keyword) = tokens.next() {
                    keywords.push(keyword.to_string());
                }
                Ok(Self::SearchBackward(keywords))
            },
            Some(other) => Err(anyhow!("Invalid command: {}", other)),
            None => Ok(Self::Nop),
        }
    }
}
