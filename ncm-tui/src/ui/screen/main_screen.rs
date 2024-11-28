use crate::config::Command;
use crate::ui::widget::UIList;
use crate::ui::Controller;
use crate::NCM_API;
use anyhow::Result;
use ncm_api::SongInfo;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::SLATE;
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListItem};
use ratatui::Frame;
use std::mem;

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

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
    current_song_lyric_timestamps: Option<Vec<u64>>,
    current_song_lyric_lines: Option<Vec<String>>,
    current_song_lyric_items: Vec<ListItem<'a>>,

    // view
    playlist_ui: UIList<'a>,
    // song_ui: Paragraph<'a>,
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
            current_song_lyric_lines: None,
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
        let current_song_lyric = ncm_api_guard.song_lyric(song_info).await?;
        let mut current_song_lyric_timestamps = Vec::new();
        let mut current_song_lyric_lines = Vec::new();
        for (lyric_timestamp, lyric_line) in current_song_lyric {
            current_song_lyric_timestamps.push(lyric_timestamp);
            current_song_lyric_lines.push(lyric_line);
        }
        self.current_song_lyric_timestamps = Some(current_song_lyric_timestamps);
        self.current_song_lyric_lines = Some(current_song_lyric_lines.clone());
        self.current_song_lyric_items = current_song_lyric_lines
            .iter()
            .map(move |lyric_line| ListItem::new(Line::from(lyric_line.clone()).centered()))
            .collect();

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

        if self.playlist_ui.state.selected() == None {
            self.playlist_ui.state.select(Some(0));
            result = Ok(true);
        }

        if self.song_ui.state.selected() == None {
            self.song_ui.state.select(Some(10)); // TODO: 计算居中歌词
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
                    let list_len = self.playlist_ui.list.len();
                    if list_len == 0 {
                        return Ok(false);
                    }
                    let list_state = &mut self.playlist_ui.state;
                    let mut selected = list_state.selected().unwrap_or_default();

                    selected = switch_line(&cmd, selected, list_len);

                    self.playlist_ui.state.select(Some(selected));
                }
                FocusPanel::LyricInside => {
                    let list_len = self.song_ui.list.len();
                    if list_len == 0 {
                        return Ok(false);
                    }
                    let list_state = &mut self.song_ui.state;
                    let mut selected = list_state.selected().unwrap_or_default();

                    selected = switch_line_no_back(&cmd, selected, list_len);

                    self.song_ui.state.select(Some(selected));
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
                            .title_bottom(format!("User: {}", self.user_name.clone()))
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
                list = match self.current_focus_panel {
                    FocusPanel::LyricInside => list
                        .highlight_style(SELECTED_STYLE)
                        .highlight_spacing(HighlightSpacing::WhenSelected),
                    _ => list,
                };
                list
            },
            state: mem::take(&mut self.song_ui.state),
        }
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        // Split the screen into left and right halves
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunk);

        // 在左半屏渲染 playlist
        let mut playlist_ui_state = self.playlist_ui.state.clone();
        frame.render_stateful_widget(&self.playlist_ui.list, chunks[0], &mut playlist_ui_state);

        // 在右半屏渲染 current_song
        let mut song_ui_state = self.song_ui.state.clone();
        frame.render_stateful_widget(&self.song_ui.list, chunks[1], &mut song_ui_state);
    }
}

fn switch_line(cmd: &Command, mut selected: usize, list_len: usize) -> usize {
    match cmd {
        Command::Up => {
            if selected == 0 {
                selected = list_len - 1;
            } else {
                selected -= 1;
            }
        }
        Command::Down => {
            if selected == list_len - 1 {
                selected = 0;
            } else {
                selected += 1;
            }
        }
        _ => {} // won't happen
    }

    selected
}

fn switch_line_no_back(cmd: &Command, mut selected: usize, list_len: usize) -> usize {
    match cmd {
        Command::Up => {
            if selected != 0 {
                selected -= 1;
            }
        }
        Command::Down => {
            if selected != list_len - 1 {
                selected += 1;
            }
        }
        _ => {} // won't happen
    }

    selected
}
