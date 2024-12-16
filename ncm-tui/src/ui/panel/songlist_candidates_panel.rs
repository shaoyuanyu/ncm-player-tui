use crate::config::style::*;
use crate::config::Command;
use crate::ui::panel::PanelFocusedStatus;
use crate::ui::Controller;
use crate::{ncm_client, player};
use ncm_api::model::Songlist;
use ratatui::layout::{Margin, Rect};
use ratatui::prelude::{Constraint, Style};
use ratatui::style::palette::tailwind;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};
use ratatui::Frame;

pub struct SonglistsPanel<'a> {
    // model
    pub focused_status: PanelFocusedStatus, // 聚焦状态交给父 screen 管理，面板自身只读不写
    //
    username: String,
    songlists: Vec<Songlist>,
    songlists_table_rows: Vec<Row<'a>>,
    songlists_table_state: TableState,
    scrollbar_state: ScrollbarState,

    // view
    songlists_table: Table<'a>,
}

impl<'a> SonglistsPanel<'a> {
    pub fn new(focused_status: PanelFocusedStatus) -> Self {
        Self {
            focused_status,
            username: String::new(),
            songlists: Vec::new(),
            songlists_table_rows: Vec::new(),
            songlists_table_state: TableState::new(),
            scrollbar_state: ScrollbarState::new(0),
            songlists_table: Table::default(),
        }
    }
}

impl<'a> SonglistsPanel<'a> {
    pub fn get_selected_songlist(&self) -> Option<Songlist> {
        if let Some(selected) = self.songlists_table_state.selected() {
            if let Some(songlist) = self.songlists.get(selected) {
                return Some(songlist.clone());
            }
        }

        None
    }

    pub fn get_selected_songlist_index(&self) -> Option<usize> {
        self.songlists_table_state.selected()
    }
}

impl<'a> Controller for SonglistsPanel<'a> {
    async fn update_model(&mut self) -> anyhow::Result<bool> {
        let mut result = Ok(false);

        if self.songlists_table_rows.is_empty() {
            let player_guard = player.lock().await;
            let user_all_songlists = player_guard.songlists();

            if let Some(login_account) = ncm_client.lock().await.login_account() {
                self.username = login_account.nickname;
            }
            self.songlists = user_all_songlists.clone();
            self.songlists_table_rows = user_all_songlists
                .iter()
                .map(|songlist| {
                    Row::from_iter(vec![
                        Cell::new(songlist.name.clone()),
                        Cell::new(songlist.creator.clone()),
                        Cell::new(format!("{:>6}", songlist.songs_count)),
                    ])
                })
                .collect();

            drop(player_guard);

            // 防止悬空
            self.songlists_table_state.select(None);

            self.scrollbar_state = ScrollbarState::new(self.songlists_table_rows.len());

            result = Ok(true);
        }

        if self.songlists_table_state.selected() == None && !self.songlists_table_rows.is_empty() {
            self.songlists_table_state.select(Some(0));
            self.scrollbar_state.first();
            result = Ok(true);
        }

        result
    }

    async fn handle_event(&mut self, cmd: Command) -> anyhow::Result<bool> {
        match cmd {
            Command::Down => {
                // 直接使用 select_next() 存在越界问题
                if let (Some(selected), list_len) = (self.songlists_table_state.selected(), self.songlists_table_rows.len()) {
                    if selected + 1 < list_len {
                        self.songlists_table_state.select_next();
                        self.scrollbar_state.next();
                    }
                }
            },
            Command::Up => {
                self.songlists_table_state.select_previous();
                self.scrollbar_state.prev();
            },
            Command::EnterOrPlay => {},
            Command::GoToTop => {
                self.songlists_table_state.select_first();
                self.scrollbar_state.first();
            },
            Command::GoToBottom => {
                // 使用 select_last() 会越界
                self.songlists_table_state.select(Some(self.songlists_table_rows.len() - 1));
                self.scrollbar_state.last();
            },
            Command::SearchForward(_) => {},
            Command::SearchBackward(_) => {},
            _ => {},
        }

        Ok(true)
    }

    fn update_view(&mut self, _style: &Style) {
        let mut songlists_table = Table::new(self.songlists_table_rows.clone(), [Constraint::Min(30), Constraint::Min(10), Constraint::Max(6)])
            .header(Row::new(vec![Cell::new("歌单"), Cell::new("创建者"), Cell::new("歌曲数")]).style(TABLE_HEADER_STYLE).height(1))
            .block({
                let mut block = Block::default()
                    .title(Line::from(format!("{}收藏的歌单", self.username)))
                    .title_bottom(Line::from("按下`Alt+Enter`开始播放选中歌单").centered())
                    .borders(Borders::ALL);
                if self.focused_status == PanelFocusedStatus::Outside {
                    block = block.border_style(PANEL_SELECTED_BORDER_STYLE);
                }

                block
            });

        // highlight
        if self.focused_status == PanelFocusedStatus::Inside {
            songlists_table = songlists_table.row_highlight_style(ITEM_SELECTED_STYLE).highlight_symbol(">")
        }

        self.songlists_table = songlists_table;
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let mut songlists_table_state = self.songlists_table_state.clone();
        frame.render_stateful_widget(&self.songlists_table, chunk, &mut songlists_table_state);

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
