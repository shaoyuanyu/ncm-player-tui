use crate::config::style::*;
use crate::config::Command;
use crate::ui::panel::PanelFocusedStatus;
use crate::ui::Controller;
use crate::{ncm_client, player};
use ncm_api::model::Song;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::{Margin, Style};
use ratatui::style::palette::tailwind;
use ratatui::widgets::{Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};
use ratatui::Frame;

pub struct PlaylistPanel<'a> {
    // model
    pub focused_status: PanelFocusedStatus, // 聚焦状态交给父 screen 管理，面板自身只读不写
    //
    playlist_name: String,
    playlist_table_rows: Vec<Row<'a>>,
    playlist_table_state: TableState,
    scrollbar_state: ScrollbarState,

    // view
    playlist_table: Table<'a>,
}

impl<'a> PlaylistPanel<'a> {
    pub fn new(focused_status: PanelFocusedStatus) -> Self {
        Self {
            focused_status,
            playlist_name: String::new(),
            playlist_table_rows: Vec::new(),
            playlist_table_state: TableState::new(),
            scrollbar_state: ScrollbarState::new(0),
            playlist_table: Table::default(),
        }
    }
}

impl<'a> PlaylistPanel<'a> {
    /// 根据当前播放列表更新 model
    async fn update_model_by_current_playlist(&mut self) -> anyhow::Result<()> {
        let player_guard = player.lock().await;
        let current_playlist_name = player_guard.current_playlist_name();

        if self.playlist_name != *current_playlist_name {
            let current_playlist = player_guard.current_playlist();

            self.set_model(current_playlist_name, current_playlist);
        }

        Ok(())
    }

    /// 手动设置 model
    ///
    /// 在 main_screen 由 self.update_model_by_current_playlist() 调用，在 playlist_screen 由外部直接调用
    pub fn set_model(&mut self, playlist_name: &String, playlist: &Vec<Song>) {
        self.playlist_name = playlist_name.clone();
        self.playlist_table_rows = playlist
            .iter()
            .map(|song| {
                Row::from_iter(vec![
                    Cell::new(song.name.clone()),
                    Cell::new(song.singer.clone()),
                    Cell::new(song.album.clone()),
                    Cell::new(format!("{:02}:{:02}", song.duration.clone() / 60000, song.duration.clone() % 60000 / 1000)),
                ])
            })
            .collect();

        // 更新 playlist_table 的 selected，防止悬空
        self.playlist_table_state.select(None);

        self.scrollbar_state = ScrollbarState::new(self.playlist_table_rows.len());
    }
}

impl<'a> Controller for PlaylistPanel<'a> {
    async fn update_model(&mut self) -> anyhow::Result<bool> {
        let mut result = Ok(false);

        if self.playlist_table_state.selected() == None && !self.playlist_table_rows.is_empty() {
            self.playlist_table_state.select(Some(0));
            self.scrollbar_state.first();
            result = Ok(true);
        }

        result
    }

    async fn handle_event(&mut self, cmd: Command) -> anyhow::Result<bool> {
        match cmd {
            Command::Down => {
                // 直接使用 select_next() 存在越界问题
                if let (Some(selected), list_len) = (self.playlist_table_state.selected(), self.playlist_table_rows.len()) {
                    if selected < list_len - 1 {
                        self.playlist_table_state.select_next();
                        self.scrollbar_state.next();
                    }
                }
            },
            Command::Up => {
                self.playlist_table_state.select_previous();
                self.scrollbar_state.prev();
            },
            Command::EnterOrPlay | Command::Play => {
                player
                    .lock()
                    .await
                    .play_particularly_now(self.playlist_table_state.selected().unwrap_or(0), ncm_client.lock().await)
                    .await?;
            },
            Command::WhereIsThisSong => {
                if let Some(index) = player.lock().await.current_song_index() {
                    self.playlist_table_state.select(Some(index));
                    self.scrollbar_state = self.scrollbar_state.position(index);
                }
            },
            Command::GoToTop => {
                self.playlist_table_state.select_first();
                self.scrollbar_state.first();
            },
            Command::GoToBottom => {
                // 使用 select_last() 会越界
                self.playlist_table_state.select(Some(self.playlist_table_rows.len() - 1));
                self.scrollbar_state.last();
            },
            Command::SearchForward(keywords) => {
                if let Some(selected) = self.playlist_table_state.selected() {
                    if let Some(next_index) = player.lock().await.search_forward_playlist(selected, keywords) {
                        self.playlist_table_state.select(Some(next_index));
                        self.scrollbar_state = self.scrollbar_state.position(next_index);
                    }
                }
            },
            Command::SearchBackward(keywords) => {
                if let Some(selected) = self.playlist_table_state.selected() {
                    if let Some(next_index) = player.lock().await.search_backward_playlist(selected, keywords) {
                        self.playlist_table_state.select(Some(next_index));
                        self.scrollbar_state = self.scrollbar_state.position(next_index);
                    }
                }
            },
            Command::RefreshPlaylist => {
                self.update_model_by_current_playlist().await?;
            },
            _ => {},
        }

        Ok(true)
    }

    fn update_view(&mut self, _style: &Style) {
        let header_style = Style::default().fg(tailwind::WHITE).bg(tailwind::RED.c300);

        let mut playlist_table = Table::new(self.playlist_table_rows.clone(), [Constraint::Min(40), Constraint::Min(15), Constraint::Min(15), Constraint::Length(6)])
            .header(
                Row::new(vec![Cell::new("曲名"), Cell::new("歌手/乐手"), Cell::new("专辑"), Cell::new("时长")])
                    .style(header_style)
                    .height(1),
            )
            .block({
                let mut block = Block::default().title(format!("Playlist: {}\u{1F4DC}", self.playlist_name.clone())).borders(Borders::ALL);
                if self.focused_status == PanelFocusedStatus::Outside {
                    block = block.border_style(PANEL_SELECTED_BORDER_STYLE);
                }

                block
            });

        // highlight
        if self.focused_status == PanelFocusedStatus::Inside {
            playlist_table = playlist_table.row_highlight_style(ITEM_SELECTED_STYLE).highlight_symbol(">")
        }

        self.playlist_table = playlist_table;
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let mut playlist_table_state = self.playlist_table_state.clone();
        frame.render_stateful_widget(&self.playlist_table, chunk, &mut playlist_table_state);

        // 渲染 scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(None)
            .begin_symbol(None)
            .end_symbol(None)
            .thumb_style(tailwind::ROSE.c800);
        let scrollbar_area = chunk.inner(Margin { vertical: 1, horizontal: 0 });
        let mut scrollbar_state = self.scrollbar_state.clone();
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}
