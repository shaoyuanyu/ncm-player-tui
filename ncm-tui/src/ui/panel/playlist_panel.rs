use crate::config::Command;
use crate::ui::panel::{PanelFocusedStatus, ITEM_SELECTED_STYLE, PANEL_SELECTED_BORDER_STYLE};
use crate::ui::Controller;
use crate::{NCM_CLIENT, PLAYER};
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::{Style, Text};
use ratatui::style::palette::tailwind;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;

pub struct PlaylistPanel<'a> {
    // model
    pub focused_status: PanelFocusedStatus, // 聚焦状态交给父 screen 管理，面板自身只读不写
    //
    playlist_name: String,
    playlist_table_rows: Vec<Row<'a>>,
    playlist_table_state: TableState,

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
            playlist_table: Table::default(),
        }
    }
}

impl<'a> Controller for PlaylistPanel<'a> {
    async fn update_model(&mut self) -> anyhow::Result<bool> {
        let mut result = Ok(false);
        let player_guard = PLAYER.lock().await;

        let current_playlist_name = player_guard.current_playlist_name();

        if self.playlist_name != *current_playlist_name {
            let current_playlist = player_guard.current_playlist();

            self.playlist_name = current_playlist_name.clone();
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

        result
    }

    async fn handle_event(&mut self, cmd: Command) -> anyhow::Result<bool> {
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
            Command::Up => {
                self.playlist_table_state.select_previous();
            }
            Command::Play => {
                PLAYER
                    .lock()
                    .await
                    .play_particularly_now(
                        self.playlist_table_state.selected().unwrap_or(0),
                        NCM_CLIENT.lock().await,
                    )
                    .await?;
            }
            Command::WhereIsThisSong => {
                if let Some(index) = PLAYER.lock().await.current_song_index() {
                    self.playlist_table_state.select(Some(index));
                }
            }
            Command::GoToTop => {
                self.playlist_table_state.select_first();
            }
            Command::GoToBottom => {
                // 使用 select_last() 会越界
                self.playlist_table_state
                    .select(Some(self.playlist_table_rows.len() - 1));
            }
            Command::SearchForward(keywords) => {
                if let Some(selected) = self.playlist_table_state.selected() {
                    if let Some(next_index) = PLAYER
                        .lock()
                        .await
                        .search_forward_playlist(selected, keywords)
                    {
                        self.playlist_table_state.select(Some(next_index));
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
                    }
                }
            }
            _ => {}
        }

        Ok(true)
    }

    fn update_view(&mut self, _style: &Style) {
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
            if self.focused_status == PanelFocusedStatus::Outside {
                block = block.border_style(PANEL_SELECTED_BORDER_STYLE);
            }

            block
        });

        // highlight
        if self.focused_status == PanelFocusedStatus::Inside {
            playlist_table = playlist_table
                .row_highlight_style(ITEM_SELECTED_STYLE)
                .highlight_symbol(">")
        }

        self.playlist_table = playlist_table;
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let mut playlist_table_state = self.playlist_table_state.clone();
        frame.render_stateful_widget(&self.playlist_table, chunk, &mut playlist_table_state);
    }
}
