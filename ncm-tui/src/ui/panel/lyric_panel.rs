use crate::config::Command;
use crate::player;
use crate::ui::panel::{
    PanelFocusedStatus, ITEM_SELECTED_STYLE, LYRIC_FOCUSED_STYLE, PANEL_SELECTED_BORDER_STYLE,
};
use crate::ui::Controller;
use ncm_api::model::Song;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Style, Text};
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState};
use ratatui::Frame;

pub struct LyricPanel<'a> {
    // model
    pub focused_status: PanelFocusedStatus, // 聚焦状态交给父 screen 管理，面板自身只读不写
    //
    song: Option<Song>,
    song_lyric_list_items: Vec<ListItem<'a>>,
    song_lyric_list_state: ListState,

    // view
    song_lyric_list: List<'a>,
}

impl<'a> LyricPanel<'a> {
    pub fn new(focused_status: PanelFocusedStatus) -> Self {
        let song_lyric_list_items = vec![ListItem::new(Text::from(vec![
            Line::from("选中音乐后回车播放").centered(),
            Line::from("也可在`列表播放`或`随机播放`模式下输入\":start\"开始自动播放").centered(),
        ]))];

        Self {
            focused_status,
            song: None,
            song_lyric_list_items,
            song_lyric_list_state: ListState::default(),
            song_lyric_list: List::default(),
        }
    }
}

impl<'a> Controller for LyricPanel<'a> {
    async fn update_model(&mut self) -> anyhow::Result<bool> {
        let mut result = Ok(false);
        let player_guard = player.lock().await;

        if self.song == *player_guard.current_song() {
            // 歌曲仍在播放，当前歌词行需更新；或者无歌曲正在播放
            // 聚焦不在 LyricInside 时，自动更新当前歌词行
            // 聚焦在 LyricInside 时，根据用户选择选中歌词行
            if self.focused_status != PanelFocusedStatus::Inside {
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

    async fn handle_event(&mut self, cmd: Command) -> anyhow::Result<bool> {
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
            Command::Up => {
                self.song_lyric_list_state.select_previous();
            }
            Command::EnterOrPlay | Command::Play => {
                // 跳转到对应编号的时间戳处播放
                let index = self.song_lyric_list_state.selected().unwrap_or(0);
                player
                    .lock()
                    .await
                    .seek_to_timestamp_with_index(index)
                    .await?;
            }
            Command::GoToTop => {
                self.song_lyric_list_state.select_first();
            }
            Command::GoToBottom => {
                // 使用 select_last() 会越界
                self.song_lyric_list_state
                    .select(Some(self.song_lyric_list_items.len() - 1));
            }
            _ => {}
        }

        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        let mut song_lyric_list = List::new(self.song_lyric_list_items.clone()).style(*style);

        // block
        song_lyric_list = match self.song.clone() {
            Some(song) => song_lyric_list.block({
                let mut block = Block::default()
                    .title(Line::from(format!("\u{1F3B5}{}", song.name)).left_aligned())
                    .title(Line::from(format!("\u{1F3A4}{}", song.singer)).right_aligned())
                    .title_bottom(Line::from(format!("\u{1F4DA}{}", song.album)).centered())
                    .borders(Borders::ALL);
                if self.focused_status == PanelFocusedStatus::Outside {
                    block = block.border_style(PANEL_SELECTED_BORDER_STYLE);
                }

                block
            }),
            None => song_lyric_list.block({
                let mut block = Block::default()
                    .title("\u{1F3B6}pick a song to play".to_string())
                    .borders(Borders::ALL);
                if self.focused_status == PanelFocusedStatus::Outside {
                    block = block.border_style(PANEL_SELECTED_BORDER_STYLE);
                }

                block
            }),
        };

        // highlight
        song_lyric_list = if self.focused_status == PanelFocusedStatus::Inside {
            song_lyric_list.highlight_style(ITEM_SELECTED_STYLE)
        } else {
            song_lyric_list
                .highlight_style(LYRIC_FOCUSED_STYLE)
                .highlight_spacing(HighlightSpacing::WhenSelected)
        };

        self.song_lyric_list = song_lyric_list;
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let mut song_lyric_list_state = self.song_lyric_list_state.clone();

        // 歌词居中
        self.correct_offset_to_make_lyric_centered(
            &mut song_lyric_list_state,
            chunk.height as usize,
        );

        frame.render_stateful_widget(&self.song_lyric_list, chunk, &mut song_lyric_list_state);
    }
}

impl<'a> LyricPanel<'a> {
    #[inline]
    /// 修正 offset 以使歌词居中
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
