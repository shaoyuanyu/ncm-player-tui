use crate::config::Command;
use crate::ui::panel::{PanelFocusedStatus, ITEM_SELECTED_STYLE, PANEL_SELECTED_BORDER_STYLE};
use crate::ui::Controller;
use crate::{ncm_client, player};
use ncm_api::model::Songlist;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

pub struct PlaylistCandidatePanel<'a> {
    // model
    pub focused_status: PanelFocusedStatus, // 聚焦状态交给父 screen 管理，面板自身只读不写
    //
    username: String,
    playlists: Vec<Songlist>,
    playlists_list_items: Vec<ListItem<'a>>,
    playlists_list_state: ListState,

    // view
    playlists_list: List<'a>,
}

impl<'a> PlaylistCandidatePanel<'a> {
    pub fn new(focused_status: PanelFocusedStatus) -> Self {
        Self {
            focused_status,
            username: String::new(),
            playlists: Vec::new(),
            playlists_list_items: Vec::new(),
            playlists_list_state: ListState::default(),
            playlists_list: List::default(),
        }
    }
}

impl<'a> PlaylistCandidatePanel<'a> {
    pub fn get_selected_songlist(&self) -> Option<Songlist> {
        if let Some(selected) = self.playlists_list_state.selected() {
            if let Some(songlist) = self.playlists.get(selected) {
                return Some(songlist.clone());
            }
        }

        None
    }

    pub fn get_selected_songlist_index(&self) -> Option<usize> {
        self.playlists_list_state.selected()
    }
}

impl<'a> Controller for PlaylistCandidatePanel<'a> {
    async fn update_model(&mut self) -> anyhow::Result<bool> {
        let mut result = Ok(false);

        if self.playlists_list_items.is_empty() {
            let player_guard = player.lock().await;
            let user_all_songlists = player_guard.playlist_candidates();

            if let Some(login_account) = ncm_client.lock().await.login_account() {
                self.username = login_account.nickname;
            }
            self.playlists = user_all_songlists.clone();
            self.playlists_list_items = user_all_songlists
                .iter()
                .map(|songlist| ListItem::new(Line::from(songlist.name.clone())))
                .collect();

            drop(player_guard);

            // 防止悬空
            self.playlists_list_state.select(None);

            result = Ok(true);
        }

        if self.playlists_list_state.selected() == None && !self.playlists_list_items.is_empty() {
            self.playlists_list_state.select(Some(0));
            result = Ok(true);
        }

        result
    }

    async fn handle_event(&mut self, cmd: Command) -> anyhow::Result<bool> {
        match cmd {
            Command::Down => {
                // 直接使用 select_next() 存在越界问题
                if let (Some(selected), list_len) = (
                    self.playlists_list_state.selected(),
                    self.playlists_list_items.len(),
                ) {
                    if selected + 1 < list_len {
                        self.playlists_list_state.select_next();
                    }
                }
            }
            Command::Up => {
                self.playlists_list_state.select_previous();
            }
            Command::EnterOrPlay => {}
            Command::GoToTop => {
                self.playlists_list_state.select_first();
            }
            Command::GoToBottom => {
                // 使用 select_last() 会越界
                self.playlists_list_state
                    .select(Some(self.playlists_list_items.len() - 1));
            }
            Command::SearchForward(_) => {}
            Command::SearchBackward(_) => {}
            _ => {}
        }

        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        let mut playlists_list = List::new(self.playlists_list_items.clone()).style(*style);

        // block
        playlists_list = playlists_list.block({
            let mut block = Block::default()
                .title(Line::from(format!("{}收藏的歌单", self.username)))
                .title_bottom(Line::from("按下 Alt + Enter 开始播放该歌单").centered())
                .borders(Borders::ALL);
            if self.focused_status == PanelFocusedStatus::Outside {
                block = block.border_style(PANEL_SELECTED_BORDER_STYLE);
            }

            block
        });

        // highlight
        if self.focused_status == PanelFocusedStatus::Inside {
            playlists_list = playlists_list.highlight_style(ITEM_SELECTED_STYLE);
        }

        self.playlists_list = playlists_list;
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let mut playlists_list_state = self.playlists_list_state.clone();
        frame.render_stateful_widget(&self.playlists_list, chunk, &mut playlists_list_state);
    }
}
