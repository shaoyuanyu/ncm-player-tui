use crate::config::Command;
use crate::ui::Controller;
use crate::{NCM_CLIENT, PLAYER};
use anyhow::Result;
use ncm_client::model::Song;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::palette::tailwind;
use ratatui::widgets::{
    Block, Borders, Cell, HighlightSpacing, List, ListItem, ListState, Row, Table, TableState,
};
use ratatui::Frame;

const PANEL_SELECTED_BORDER_STYLE: Style = Style::new().fg(tailwind::RED.c800);
const ITEM_SELECTED_STYLE: Style = Style::new()
    .bg(tailwind::RED.c400)
    .add_modifier(Modifier::BOLD);
const LYRIC_FOCUSED_STYLE: Style = Style::new()
    .fg(tailwind::RED.c600)
    .add_modifier(Modifier::BOLD);

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
    playlist_name: String,
    playlist_table_rows: Vec<Row<'a>>,
    playlist_table_state: TableState,
    //
    song: Option<Song>,
    song_lyric_list_items: Vec<ListItem<'a>>,
    song_lyric_list_state: ListState,

    // view
    playlist_table: Table<'a>,
    song_lyric_list: List<'a>,
}

impl<'a> MainScreen<'a> {
    pub fn new(_normal_style: &Style) -> Self {
        let song_lyric_list_items = vec![ListItem::new(Text::from(vec![
            Line::from("选中音乐后回车播放").centered(),
            Line::from("也可在`列表播放`或`随机播放`模式下输入\":start\"开始自动播放").centered(),
        ]))];

        Self {
            current_focus_panel: FocusPanel::PlaylistOutside,
            playlist_name: String::new(),
            playlist_table_rows: Vec::new(),
            playlist_table_state: TableState::new(),
            song: None,
            song_lyric_list_items,
            song_lyric_list_state: ListState::default(),
            playlist_table: Table::default(),
            song_lyric_list: List::default(),
        }
    }
}

impl<'a> Controller for MainScreen<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        let mut result = Ok(false);

        let player_guard = PLAYER.lock().await;

        // playlist
        let current_playlist_name = player_guard.current_playlist_name();
        let current_playlist = player_guard.current_playlist();

        if self.playlist_name != *current_playlist_name {
            self.playlist_name = current_playlist_name.clone();
            //
            self.playlist_table_rows = current_playlist
                .iter()
                .map(|song| {
                    Row::from_iter(vec![
                        Cell::new(song.name.clone()),
                        Cell::new(song.singer.clone()),
                        Cell::new(song.album.clone()),
                        Cell::new(format!(
                            "{:02}:{:02}",
                            song.duration.clone() / 60000,
                            song.duration.clone() % 60000 / 1000
                        )),
                    ])
                })
                .collect();

            // 更新 playlist_table 的 selected，防止悬空
            self.playlist_table_state.select(None);

            result = Ok(true);
        }

        if self.playlist_table_state.selected() == None && !self.playlist_table_rows.is_empty() {
            self.playlist_table_state.select(Some(0));
            result = Ok(true);
        }

        // song
        if self.song == *player_guard.current_song() {
            // 歌曲仍在播放，当前歌词行需更新；或者无歌曲正在播放
            // current_focus_panel 不为 LyricInside 时，自动更新当前歌词行
            // current_focus_panel 为 LyricInside 时，根据用户选择选中歌词行
            if self.current_focus_panel != FocusPanel::LyricInside {
                if self.song_lyric_list_state.selected() != player_guard.current_lyric_line_index()
                {
                    self.song_lyric_list_state
                        .select(player_guard.current_lyric_line_index());

                    result = Ok(true);
                }
            }
        } else {
            // 切换到新歌
            self.song = player_guard.current_song().clone();
            // 更新歌词 ListItem
            if let Some(lyrics) = player_guard.current_song_lyrics() {
                // 有歌词
                self.song_lyric_list_items = lyrics
                    .iter()
                    .map(|lyric_line| {
                        let mut lines: Vec<Line> = Vec::new();
                        lines.push(Line::from(lyric_line.lyric_line.to_owned()).centered());
                        if let Some(trans_lyric_line) = lyric_line.trans_lyric_line.as_ref() {
                            lines.push(Line::from(trans_lyric_line.to_owned()).centered());
                        }
                        // TODO: 添加罗马音显示设置
                        // if let Some(roman_lyric_line) = lyric_line.roman_lyric_line.as_ref() {
                        //     lines.push(Line::from(roman_lyric_line.to_owned()).centered());
                        // }
                        ListItem::new(Text::from(lines))
                    })
                    .collect();
            } else {
                // 无歌词（纯音乐或网络异常）
                self.song_lyric_list_items = Vec::new();
                self.song_lyric_list_items.push(ListItem::new(Text::from(
                    Line::from("无歌词，请欣赏").centered(),
                )));
            }

            // 更新 song_ui selected，防止悬空
            self.song_lyric_list_state.select(None);

            result = Ok(true);
        }

        if self.song_lyric_list_state.selected() == None && !self.song_lyric_list_items.is_empty() {
            self.song_lyric_list_state.select(Some(0));
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
                        Command::Down => {
                            // 直接使用 select_next() 存在越界问题
                            if let (Some(selected), list_len) = (
                                self.playlist_table_state.selected(),
                                self.playlist_table_rows.len(),
                            ) {
                                if selected < list_len - 1 {
                                    self.playlist_table_state.select_next();
                                }
                            }
                        }
                        Command::Up => self.playlist_table_state.select_previous(),
                        _ => {} // never happen
                    }
                }
                FocusPanel::LyricInside => {
                    match cmd {
                        Command::Down => {
                            // 直接使用 select_next() 存在越界问题
                            if let (Some(selected), list_len) = (
                                self.song_lyric_list_state.selected(),
                                self.song_lyric_list_items.len(),
                            ) {
                                if selected < list_len - 1 {
                                    self.song_lyric_list_state.select_next();
                                }
                            }
                        }
                        Command::Up => self.song_lyric_list_state.select_previous(),
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
                            self.playlist_table_state.selected().unwrap_or(0),
                            NCM_CLIENT.lock().await,
                        )
                        .await?;
                }
                FocusPanel::LyricInside => {
                    // 跳转到对应编号的时间戳处播放
                    let index = self.song_lyric_list_state.selected().unwrap_or(0);
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
            Command::WhereIsThisSong => {
                if let Some(index) = PLAYER.lock().await.current_song_index() {
                    self.playlist_table_state.select(Some(index));
                    self.current_focus_panel = FocusPanel::PlaylistInside;
                }
            }
            Command::GoToTop | Command::GoToBottom => {
                if self.current_focus_panel == FocusPanel::PlaylistInside {
                    match cmd {
                        Command::GoToTop => self.playlist_table_state.select_first(),
                        Command::GoToBottom => {
                            // 使用 select_last() 会越界
                            self.playlist_table_state
                                .select(Some(self.playlist_table_rows.len() - 1));
                        }
                        _ => {} // never happen
                    }
                } else if self.current_focus_panel == FocusPanel::LyricInside {
                    match cmd {
                        Command::GoToTop => self.song_lyric_list_state.select_first(),
                        Command::GoToBottom => {
                            // 使用 select_last() 会越界
                            self.song_lyric_list_state
                                .select(Some(self.song_lyric_list_items.len() - 1));
                        }
                        _ => {} // never happen
                    }
                }
            }
            Command::SearchForward(keywords) => {
                if let Some(selected) = self.playlist_table_state.selected() {
                    if let Some(next_index) = PLAYER
                        .lock()
                        .await
                        .search_forward_playlist(selected, keywords)
                    {
                        self.playlist_table_state.select(Some(next_index));
                        self.current_focus_panel = FocusPanel::PlaylistInside;
                    }
                }
            }
            Command::SearchBackward(keywords) => {
                if let Some(selected) = self.playlist_table_state.selected() {
                    if let Some(next_index) = PLAYER
                        .lock()
                        .await
                        .search_backward_playlist(selected, keywords)
                    {
                        self.playlist_table_state.select(Some(next_index));
                        self.current_focus_panel = FocusPanel::PlaylistInside;
                    }
                }
            }
            _ => {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        //
        self.update_playlist_view(style);

        //
        self.update_song_lyric_view(style);
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        // 分为左右两个面板
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunk);

        // 在左半屏渲染 playlist_table
        let mut playlist_table_state = self.playlist_table_state.clone();
        frame.render_stateful_widget(&self.playlist_table, chunks[0], &mut playlist_table_state);

        // 在右半屏渲染 current_song
        let mut song_lyric_list_state = self.song_lyric_list_state.clone();
        // 歌词居中
        self.correct_offset_to_make_lyric_centered(
            &mut song_lyric_list_state,
            chunks[1].height as usize,
        );
        //
        frame.render_stateful_widget(&self.song_lyric_list, chunks[1], &mut song_lyric_list_state);
    }
}

/// private
impl<'a> MainScreen<'a> {
    #[inline]
    fn update_playlist_view(&mut self, _style: &Style) {
        let header_style = Style::default().fg(tailwind::WHITE).bg(tailwind::RED.c300);

        let mut playlist_table = Table::new(
            self.playlist_table_rows.clone(),
            [
                Constraint::Min(40),
                Constraint::Min(15),
                Constraint::Min(15),
                Constraint::Length(6),
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(Text::from("曲名")),
                Cell::new("歌手/乐手"),
                Cell::new("专辑"),
                Cell::new("时长"),
            ])
            .style(header_style)
            .height(1),
        )
        .block({
            let mut block = Block::default()
                .title(format!("Playlist: {}\u{1F4DC}", self.playlist_name.clone()))
                .borders(Borders::ALL);
            if self.current_focus_panel == FocusPanel::PlaylistOutside {
                block = block.border_style(PANEL_SELECTED_BORDER_STYLE);
            }

            block
        });

        // highlight
        if self.current_focus_panel == FocusPanel::PlaylistInside {
            playlist_table = playlist_table
                .row_highlight_style(ITEM_SELECTED_STYLE)
                .highlight_symbol(">")
        }

        self.playlist_table = playlist_table;
    }

    #[inline]
    fn update_song_lyric_view(&mut self, style: &Style) {
        let mut song_lyric_list = List::new(self.song_lyric_list_items.clone()).style(*style);

        // block
        song_lyric_list = match self.song.clone() {
            Some(song) => song_lyric_list.block({
                let mut block = Block::default()
                    .title(Line::from(format!("\u{1F3B5}{}", song.name)).left_aligned())
                    .title(Line::from(format!("\u{1F3A4}{}", song.singer)).right_aligned())
                    .title_bottom(Line::from(format!("\u{1F4DA}{}", song.album)).centered())
                    .borders(Borders::ALL);
                if self.current_focus_panel == FocusPanel::LyricOutside {
                    block = block.border_style(PANEL_SELECTED_BORDER_STYLE);
                }

                block
            }),
            None => song_lyric_list.block({
                let mut block = Block::default()
                    .title("\u{1F3B6}pick a song to play".to_string())
                    .borders(Borders::ALL);
                if self.current_focus_panel == FocusPanel::LyricOutside {
                    block = block.border_style(PANEL_SELECTED_BORDER_STYLE);
                }

                block
            }),
        };
        // highlight
        song_lyric_list = if self.current_focus_panel == FocusPanel::LyricInside {
            song_lyric_list.highlight_style(ITEM_SELECTED_STYLE)
        } else {
            song_lyric_list
                .highlight_style(LYRIC_FOCUSED_STYLE)
                .highlight_spacing(HighlightSpacing::WhenSelected)
        };

        self.song_lyric_list = song_lyric_list;
    }

    /// 修正 offset 以使歌词居中
    #[inline]
    fn correct_offset_to_make_lyric_centered(
        &self,
        lyric_list_state: &mut ListState,
        available_line_count: usize,
    ) {
        if self.song_lyric_list_items.len() > 1 {
            let current_index = lyric_list_state.selected().unwrap_or(0);
            // 一句歌词所占行数（带翻译的歌词会占多行）
            let lyric_line_count = self
                .song_lyric_list_items
                .get(current_index)
                .unwrap()
                .height();
            let half_line_count = available_line_count / lyric_line_count / 2;
            let near_top_line = 0 + half_line_count;
            let near_bottom_line = if self.song_lyric_list_items.len() - 1 >= 2 * half_line_count {
                self.song_lyric_list_items.len() - 1 - half_line_count
            } else {
                half_line_count
            };
            // 修正 offset
            if current_index >= near_top_line {
                if current_index >= near_bottom_line {
                    // 接近底部时取消滚动，不居中
                    *lyric_list_state.offset_mut() = near_bottom_line - half_line_count;
                } else {
                    // 动态居中
                    *lyric_list_state.offset_mut() = current_index - half_line_count;
                }
            }
        }
    }
}
