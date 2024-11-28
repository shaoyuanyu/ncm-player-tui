use crate::config::Command;
use crate::ui::widget::UIList;
use crate::ui::Controller;
use crate::{NCM_API, PLAYER};
use anyhow::Result;
use itertools::Itertools;
use ncm_api::SongInfo;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::{RED, SLATE};
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListItem};
use ratatui::Frame;
use std::mem;

const PLAYLIST_SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const LYRIC_SELECTED_STYLE: Style = Style::new().fg(RED.c600).add_modifier(Modifier::BOLD);

pub enum FocusPanel {
    PlaylistOutside,
    PlaylistInside,
    LyricOutside,
    LyricInside,
}

pub struct MainScreen<'a> {
    // model
    current_focus_panel: FocusPanel,
    //
    user_name: String,
    playlist_name: String,
    playlist: Vec<SongInfo>, // TODO: Playlist
    playlist_items: Vec<ListItem<'a>>,
    //
    current_song_info: Option<SongInfo>,
    current_song_lyric_timestamps: Option<Vec<u64>>, // 单位: ms
    current_song_lyric_timestamp_index: Option<usize>,
    current_song_lyric_items: Vec<ListItem<'a>>,

    // view
    playlist_ui: UIList<'a>,
    song_ui: UIList<'a>,
}

impl<'a> MainScreen<'a> {
    pub fn new(_normal_style: &Style) -> Self {
        Self {
            current_focus_panel: FocusPanel::PlaylistOutside,
            user_name: String::new(),
            playlist_name: String::new(),
            playlist: Vec::new(),
            playlist_items: Vec::new(),
            current_song_info: None,
            current_song_lyric_timestamps: None,
            current_song_lyric_timestamp_index: None,
            current_song_lyric_items: Vec::new(),
            playlist_ui: UIList::default(),
            song_ui: UIList::default(),
        }
    }
}

impl<'a> MainScreen<'a> {
    pub fn update_playlist_model(&mut self, play_list_name: String, playlist: Vec<SongInfo>) {
        self.playlist_name = play_list_name.to_string();
        self.playlist = playlist;
        self.playlist_items = self
            .playlist
            .iter()
            .map(|song| ListItem::new(song.name.clone()))
            .collect();
    }

    async fn play_song(&mut self, mut song_info: SongInfo) -> Result<()> {
        let ncm_api_guard = NCM_API.lock().await;

        // 更新 song url
        song_info.song_url = ncm_api_guard.get_song_url(song_info.id).await?.url;
        self.current_song_info = Some(song_info.clone());

        // 更新歌词
        if let Ok(current_song_lyric) = ncm_api_guard.song_lyric(song_info.clone()).await {
            let mut current_song_lyric_timestamps = Vec::new();
            let mut current_song_lyric_lines = Vec::new();
            for (lyric_timestamp, lyric_line) in current_song_lyric {
                current_song_lyric_timestamps.push(lyric_timestamp);
                current_song_lyric_lines.push(lyric_line);
            }
            //
            let mut current_song_lyric_timestamps_formated = Vec::new();
            self.current_song_lyric_items = Vec::new();
            //
            for (prev, curr, next) in current_song_lyric_timestamps
                .iter()
                .zip(current_song_lyric_lines.iter())
                .tuple_windows()
            {
                if curr.0 == next.0 {
                    // 下句是本句的翻译
                    current_song_lyric_timestamps_formated.push(curr.0.clone());
                    self.current_song_lyric_items
                        .push(ListItem::new(Text::from(vec![
                            Line::from(format!("{}", curr.1)).centered(),
                            Line::from(format!("{}", next.1)).centered(),
                        ])));
                } else if prev.0 == curr.0 {
                    // 本句是上句的翻译
                    // do nothing
                } else {
                    // 无翻译
                    current_song_lyric_timestamps_formated.push(curr.0.clone());
                    self.current_song_lyric_items.push(ListItem::new(Text::from(
                        Line::from(format!("{}", curr.1)).centered(),
                    )));
                }
            }
            //
            self.current_song_lyric_timestamp_index = Some(0);
            self.current_song_lyric_timestamps = Some(current_song_lyric_timestamps_formated);
        } else {
            // 获取歌词失败（纯音乐或网络波动）
            self.current_song_lyric_timestamp_index = Some(0);
            self.current_song_lyric_timestamps = Some(vec![0]);
            self.current_song_lyric_items = Vec::new();
            self.current_song_lyric_items.push(ListItem::new(Text::from(
                Line::from("无歌词，请欣赏").centered(),
            )));
        }

        // player
        let player_guard = PLAYER.lock().await;
        player_guard.stop();
        player_guard.set_uri(Some(song_info.song_url.as_str()));
        player_guard.set_volume(0.2);
        player_guard.play();

        Ok(())
    }
}

impl<'a> Controller for MainScreen<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        let mut result = Ok(false);

        // username
        if self.user_name.is_empty() {
            if let Some(login_info) = NCM_API.lock().await.login_info() {
                self.user_name = login_info.nickname;
            }

            result = Ok(true);
        }

        // lyric 歌词推进
        if let Some(current_player_position) = PLAYER.lock().await.position() {
            if let Some(current_lyric_index) = self.current_song_lyric_timestamp_index {
                // 注意歌词 index 越界问题
                if current_lyric_index
                    < self.current_song_lyric_timestamps.as_ref().unwrap().len() - 1
                {
                    let next_lyric_timestamp = self.current_song_lyric_timestamps.as_ref().unwrap()
                        [current_lyric_index + 1]
                        .clone();

                    // 切换到下一句歌词
                    if current_player_position.mseconds() >= next_lyric_timestamp {
                        self.current_song_lyric_timestamp_index = self
                            .current_song_lyric_timestamp_index
                            .map(|index| index + 1);

                        self.song_ui
                            .state
                            .select(Some(self.current_song_lyric_timestamp_index.unwrap()));

                        result = Ok(true);
                    }
                }
            }
        }

        result
    }

    async fn handle_event(&mut self, cmd: Command) -> Result<bool> {
        match cmd {
            Command::Esc => match self.current_focus_panel {
                FocusPanel::PlaylistInside => {
                    self.current_focus_panel = FocusPanel::PlaylistOutside;
                }
                FocusPanel::LyricInside => {
                    self.current_focus_panel = FocusPanel::LyricOutside;
                }
                _ => {
                    return Ok(false);
                }
            },
            Command::Down | Command::Up => match self.current_focus_panel {
                FocusPanel::PlaylistOutside => {
                    self.current_focus_panel = FocusPanel::PlaylistInside;
                }
                FocusPanel::LyricOutside => {
                    self.current_focus_panel = FocusPanel::LyricInside;
                }
                FocusPanel::PlaylistInside => {
                    match cmd {
                        Command::Down => self.playlist_ui.state.select_next(),
                        Command::Up => self.playlist_ui.state.select_previous(),
                        _ => {} // never happen
                    }
                }
                FocusPanel::LyricInside => {
                    match cmd {
                        Command::Down => self.song_ui.state.select_next(),
                        Command::Up => self.song_ui.state.select_previous(),
                        _ => {} // never happen
                    }
                }
            },
            Command::NextPanel => match self.current_focus_panel {
                FocusPanel::PlaylistOutside => {
                    self.current_focus_panel = FocusPanel::LyricOutside;
                }
                FocusPanel::LyricOutside => {
                    return Ok(false);
                }
                _ => {
                    return Ok(false);
                }
            },
            Command::PrevPanel => match self.current_focus_panel {
                FocusPanel::PlaylistOutside => {
                    return Ok(false);
                }
                FocusPanel::LyricOutside => {
                    self.current_focus_panel = FocusPanel::PlaylistOutside;
                }
                _ => {
                    return Ok(false);
                }
            },
            Command::Play => match self.current_focus_panel {
                FocusPanel::PlaylistInside => {
                    self.play_song(
                        self.playlist
                            .get(self.playlist_ui.state.selected().unwrap_or(0))
                            .unwrap()
                            .clone(),
                    )
                    .await?;
                }
                FocusPanel::LyricInside => {}
                FocusPanel::PlaylistOutside => {
                    self.current_focus_panel = FocusPanel::PlaylistInside;
                }
                FocusPanel::LyricOutside => {
                    self.current_focus_panel = FocusPanel::LyricInside;
                }
            },
            _ => {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        self.playlist_ui = UIList {
            list: {
                let mut list = List::new(self.playlist_items.clone())
                    .block(
                        Block::default()
                            .title(format!("Playlist: {}", self.playlist_name.clone()))
                            .title_bottom(Line::from(format!("User: {}", self.user_name.clone())).right_aligned())
                            .borders(Borders::ALL),
                    )
                    .style(*style);
                list = match self.current_focus_panel {
                    FocusPanel::PlaylistInside => list
                        .highlight_style(PLAYLIST_SELECTED_STYLE)
                        .highlight_symbol(">"),
                    _ => list,
                };
                list
            },
            state: mem::take(&mut self.playlist_ui.state),
        };
        if self.playlist_ui.state.selected() == None {
            self.playlist_ui.state.select(Some(0));
        }

        self.song_ui = UIList {
            list: {
                let mut list = List::new(self.current_song_lyric_items.clone()).style(*style);
                list = match self.current_song_info.clone() {
                    Some(current_song_info) => list.block(
                        Block::default()
                            .title(
                                Line::from(format!("\u{1F3B5}{}", current_song_info.name))
                                    .left_aligned(),
                            )
                            .title(
                                Line::from(format!("\u{1F3A4}{}", current_song_info.singer))
                                    .right_aligned(),
                            )
                            .title_bottom(
                                Line::from(format!("\u{1F4DA}{}", current_song_info.album))
                                    .centered(),
                            )
                            .borders(Borders::ALL),
                    ),
                    None => list.block(
                        Block::default()
                            .title("\u{1F3B6}pick a song to play".to_string())
                            .borders(Borders::ALL),
                    ),
                };
                // list = match self.current_focus_panel {
                //     FocusPanel::LyricInside => list
                //         .highlight_style(LYRIC_SELECTED_STYLE)
                //         .highlight_spacing(HighlightSpacing::WhenSelected),
                //     _ => list,
                // };
                list = list
                    .highlight_style(LYRIC_SELECTED_STYLE)
                    .highlight_spacing(HighlightSpacing::WhenSelected);
                list
            },
            state: mem::take(&mut self.song_ui.state),
        };
        if self.song_ui.state.selected() == None {
            self.song_ui.state.select(Some(0));
        }
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        // 分为左右两个面板
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunk);

        // 在左半屏渲染 playlist
        let mut playlist_ui_state = self.playlist_ui.state.clone();
        frame.render_stateful_widget(&self.playlist_ui.list, chunks[0], &mut playlist_ui_state);

        // 在右半屏渲染 current_song
        let mut song_ui_state = self.song_ui.state.clone();
        // 歌词居中
        if !self.current_song_lyric_items.is_empty() && self.current_song_lyric_items.len() != 1 {
            let current_index = song_ui_state.selected().unwrap();
            // 可显示行数
            let available_line_count = chunks[1].height as usize;
            // 一句歌词所占行数（带翻译的歌词会占2行）
            let lyric_line_count = self
                .current_song_lyric_items
                .get(current_index)
                .unwrap()
                .height();
            let half_line_count = available_line_count / lyric_line_count / 2;
            let near_top_line = 0 + half_line_count;
            let near_bottom_line = self.current_song_lyric_items.len() - 1 - half_line_count;
            // 修正 offset
            if current_index >= near_top_line {
                if current_index >= near_bottom_line {
                    // 接近底部时取消滚动，不居中
                    *song_ui_state.offset_mut() = near_bottom_line - half_line_count;
                } else {
                    // 动态居中
                    *song_ui_state.offset_mut() = current_index - half_line_count;
                }
            }
        }
        frame.render_stateful_widget(&self.song_ui.list, chunks[1], &mut song_ui_state);
    }
}
