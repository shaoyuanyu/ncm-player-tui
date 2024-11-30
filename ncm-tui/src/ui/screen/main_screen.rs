use crate::config::Command;
use crate::ui::widget::UIList;
use crate::ui::Controller;
use crate::{NCM_API, PLAYER};
use anyhow::Result;
use ncm_api::SongInfo;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::{RED, SLATE};
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListItem};
use ratatui::Frame;
use std::mem;

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const LYRIC_FOCUSED_STYLE: Style = Style::new().fg(RED.c600).add_modifier(Modifier::BOLD);

#[derive(PartialEq)]
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
    //
    playlist_name: String,
    playlist_items: Vec<ListItem<'a>>,
    //
    current_song_info: Option<SongInfo>,
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
            playlist_items: Vec::new(),
            current_song_info: None,
            current_song_lyric_items: Vec::new(),
            playlist_ui: UIList::default(),
            song_ui: UIList::default(),
        }
    }
}

impl<'a> Controller for MainScreen<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        let mut result = Ok(false);

        let player_guard = PLAYER.lock().await;

        // username
        if self.user_name.is_empty() {
            if let Some(login_info) = NCM_API.lock().await.login_info() {
                self.user_name = login_info.nickname;
            }

            result = Ok(true);
        }

        // playlist
        if self.playlist_name != *player_guard.current_playlist_name_ref() {
            self.playlist_name = player_guard.current_playlist_name_ref().clone();
            self.playlist_items = player_guard
                .current_playlist()
                .iter()
                .map(|song| ListItem::new(song.name.clone()))
                .collect();

            // 更新 playlist_ui selected，防止悬空
            self.playlist_ui.state.select(None);
        }

        // lyric
        if self.current_song_info == *player_guard.current_song_info_ref() {
            // 歌曲仍在播放，当前歌词行需更新；或者无歌曲正在播放
            // current_focus_panel 不为 LyricInside 时，自动更新当前歌词行
            // current_focus_panel 为 LyricInside 时，根据用户选择选中歌词行
            if self.current_focus_panel != FocusPanel::LyricInside {
                if self.song_ui.state.selected() != player_guard.current_song_lyric_index() {
                    self.song_ui
                        .state
                        .select(player_guard.current_song_lyric_index());

                    result = Ok(true);
                }
            }
        } else {
            // 切换到新歌
            self.current_song_info = player_guard.current_song_info_ref().clone();
            // 更新歌词 ListItem
            if let Some(lyrics) = player_guard.current_song_lyrics() {
                // 有歌词
                self.current_song_lyric_items = lyrics
                    .iter()
                    .map(|lyric| {
                        if lyric.1 != None {
                            // 有翻译
                            ListItem::new(Text::from(vec![
                                Line::from(lyric.0.clone()).centered(),
                                Line::from(lyric.1.clone().unwrap()).centered(),
                            ]))
                        } else {
                            // 无翻译
                            ListItem::new(Text::from(Line::from(lyric.0.clone()).centered()))
                        }
                    })
                    .collect();
            } else {
                // 无歌词（纯音乐或网络异常）
                self.current_song_lyric_items = Vec::new();
                self.current_song_lyric_items.push(ListItem::new(Text::from(
                    Line::from("无歌词，请欣赏").centered(),
                )));
            }

            // 更新 song_ui selected，防止悬空
            self.song_ui.state.select(None);

            result = Ok(true);
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
                    PLAYER
                        .lock()
                        .await
                        .play_particularly_now(
                            self.playlist_ui.state.selected().unwrap_or(0),
                            NCM_API.lock().await,
                        )
                        .await?;
                }
                FocusPanel::LyricInside => {
                    // 跳转到对应编号的时间戳处播放
                    let index = self.song_ui.state.selected().unwrap_or(0);
                    PLAYER
                        .lock()
                        .await
                        .seek_to_timestamp_with_index(index)
                        .await?;
                    //
                    self.current_focus_panel = FocusPanel::LyricOutside;
                }
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
                            .title(format!("Playlist: {}\u{1F4DC}", self.playlist_name.clone()))
                            .title_bottom(
                                Line::from(format!("User: {}\u{1F3A7}", self.user_name.clone()))
                                    .right_aligned(),
                            )
                            .borders(Borders::ALL),
                    )
                    .style(*style);
                list = match self.current_focus_panel {
                    FocusPanel::PlaylistInside => {
                        list.highlight_style(SELECTED_STYLE).highlight_symbol(">")
                    }
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
                list = if self.current_focus_panel == FocusPanel::LyricInside {
                    list.highlight_style(SELECTED_STYLE)
                } else {
                    list.highlight_style(LYRIC_FOCUSED_STYLE)
                        .highlight_spacing(HighlightSpacing::WhenSelected)
                };
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
        if self.current_song_lyric_items.len() > 1 {
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
            let near_bottom_line =
                if self.current_song_lyric_items.len() - 1 - half_line_count >= half_line_count {
                    self.current_song_lyric_items.len() - 1 - half_line_count
                } else {
                    half_line_count
                };
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
