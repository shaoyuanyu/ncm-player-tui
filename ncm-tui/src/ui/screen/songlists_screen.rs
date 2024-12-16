use crate::config::{Command, ScreenEnum};
use crate::ui::panel::{PanelFocusedStatus, PlaylistPanel, SonglistsPanel};
use crate::ui::Controller;
use crate::{command_queue, ncm_client, player};
use log::debug;
use ncm_api::model::Songlist;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Style;
use ratatui::Frame;

#[derive(PartialEq)]
enum Panels {
    SonglistCandidates,
    SonglistContent,
}

#[derive(PartialEq)]
enum FocusPanel {
    SonglistCandidatesOutside,
    SonglistCandidatesInside,
    SonglistContentOutside,
    SonglistContentInside,
}

pub struct SonglistsScreen<'a> {
    current_focus_panel: FocusPanel,
    //
    current_selected_songlist: Option<Songlist>,
    //
    songlist_candidates_panel: SonglistsPanel<'a>,
    songlist_content_panel: PlaylistPanel<'a>,
}

impl<'a> SonglistsScreen<'a> {
    pub fn new(_normal_style: &Style) -> Self {
        Self {
            current_focus_panel: FocusPanel::SonglistCandidatesOutside,
            current_selected_songlist: None,
            songlist_candidates_panel: SonglistsPanel::new(PanelFocusedStatus::Outside),
            songlist_content_panel: PlaylistPanel::new(PanelFocusedStatus::Nop),
        }
    }
}

impl<'a> Controller for SonglistsScreen<'a> {
    async fn update_model(&mut self) -> anyhow::Result<bool> {
        let mut result = Ok(false);

        // songlist candidates
        if self.songlist_candidates_panel.update_model().await? {
            result = Ok(true);
        }

        // songlist content
        if self.songlist_content_panel.update_model().await? {
            result = Ok(true);
        }

        result
    }

    async fn handle_event(&mut self, cmd: Command) -> anyhow::Result<bool> {
        use Command::*;
        use FocusPanel::*;

        match (cmd.clone(), &self.current_focus_panel) {
            //
            (Esc, SonglistCandidatesInside) => {
                self.focus_panel_outside(Panels::SonglistCandidates);
            },
            (Esc, SonglistContentInside) => {
                self.focus_panel_outside(Panels::SonglistContent);
            },

            //
            (Down | Up, SonglistCandidatesOutside) => {
                self.focus_panel_inside(Panels::SonglistCandidates);
            },
            (Down | Up, SonglistContentOutside) => {
                self.focus_panel_inside(Panels::SonglistContent);
            },
            (Down | Up, SonglistCandidatesInside) => {
                self.songlist_candidates_panel.handle_event(cmd).await?;
            },
            (Down | Up, SonglistContentInside) => {
                self.songlist_content_panel.handle_event(cmd).await?;
            },

            //
            (NextPanel, SonglistCandidatesOutside) => {
                self.focus_panel_outside(Panels::SonglistContent);
            },
            (PrevPanel, SonglistContentOutside) => {
                self.focus_panel_outside(Panels::SonglistCandidates);
            },

            //
            (EnterOrPlay, SonglistCandidatesOutside) => {
                self.focus_panel_inside(Panels::SonglistCandidates);
            },
            (EnterOrPlay, SonglistContentOutside) => {
                self.focus_panel_inside(Panels::SonglistContent);
            },
            (EnterOrPlay, SonglistCandidatesInside) => {
                // 加载歌单
                if let Some(mut selected_songlist) = self.songlist_candidates_panel.get_selected_songlist() {
                    ncm_client.lock().await.load_songlist_songs(&mut selected_songlist).await?;

                    self.songlist_content_panel.set_model(&selected_songlist.name, &selected_songlist.songs);

                    self.current_selected_songlist = Some(selected_songlist);
                }
            },
            // 切换歌单并从选中歌曲开始播放
            (EnterOrPlay | Play, SonglistContentInside) => {
                if let Some(selected_songlist_index) = self.songlist_candidates_panel.get_selected_songlist_index() {
                    debug!("切换到 {} 号歌单", selected_songlist_index);

                    // 切换当前播放列表
                    player.lock().await.switch_playlist(selected_songlist_index, ncm_client.lock().await).await?;

                    // 播放选中歌曲
                    self.songlist_content_panel.handle_event(cmd).await?;

                    // 返回 main_screen ，刷新播放列表显示
                    let mut command_queue_guard = command_queue.lock().await;
                    command_queue_guard.push_back(GotoScreen(ScreenEnum::Main));
                    command_queue_guard.push_back(RefreshPlaylist);
                    command_queue_guard.push_back(WhereIsThisSong);
                    drop(command_queue_guard);
                }
            },
            // 切换歌单并开始播放
            (Play, SonglistCandidatesInside) => {
                if let Some(selected_songlist_index) = self.songlist_candidates_panel.get_selected_songlist_index() {
                    debug!("切换到 {} 号歌单", selected_songlist_index);

                    // 切换当前播放列表
                    player.lock().await.switch_playlist(selected_songlist_index, ncm_client.lock().await).await?;

                    // 开始自动播放，返回 main_screen ，刷新播放列表显示
                    let mut command_queue_guard = command_queue.lock().await;
                    command_queue_guard.push_back(StartPlay);
                    command_queue_guard.push_back(GotoScreen(ScreenEnum::Main));
                    command_queue_guard.push_back(RefreshPlaylist);
                    command_queue_guard.push_back(WhereIsThisSong);
                    drop(command_queue_guard);
                }
            },

            //
            (GoToTop | GoToBottom, SonglistCandidatesOutside | SonglistCandidatesInside) => {
                self.songlist_candidates_panel.handle_event(cmd).await?;
            },
            (GoToTop | GoToBottom, SonglistContentOutside | SonglistContentInside) => {
                self.songlist_content_panel.handle_event(cmd).await?;
                self.focus_panel_inside(Panels::SonglistContent);
            },

            //
            (SearchForward(_) | SearchBackward(_), SonglistCandidatesOutside | SonglistCandidatesInside) => {
                self.songlist_candidates_panel.handle_event(cmd).await?;
            },
            (SearchForward(_) | SearchBackward(_), SonglistContentOutside | SonglistContentInside) => {
                self.songlist_content_panel.handle_event(cmd).await?;
                self.focus_panel_inside(Panels::SonglistContent);
            },

            //
            (_, _) => {
                return Ok(false);
            },
        }

        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        self.songlist_candidates_panel.update_view(style);

        self.songlist_content_panel.update_view(style);
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        // 分为左右两个面板
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(chunk);

        // 在左半屏渲染 songlist_candidates_panel
        self.songlist_candidates_panel.draw(frame, chunks[0]);

        // 在右半屏渲染 songlist_content_panel
        self.songlist_content_panel.draw(frame, chunks[1]);
    }
}

/// private
impl<'a> SonglistsScreen<'a> {
    fn focus_panel_outside(&mut self, to_panel: Panels) {
        match to_panel {
            Panels::SonglistCandidates => {
                self.current_focus_panel = FocusPanel::SonglistCandidatesOutside;
                self.songlist_candidates_panel.focused_status = PanelFocusedStatus::Outside;
                self.songlist_content_panel.focused_status = PanelFocusedStatus::Nop;
            },
            Panels::SonglistContent => {
                self.current_focus_panel = FocusPanel::SonglistContentOutside;
                self.songlist_candidates_panel.focused_status = PanelFocusedStatus::Nop;
                self.songlist_content_panel.focused_status = PanelFocusedStatus::Outside;
            },
        }
    }

    fn focus_panel_inside(&mut self, to_panel: Panels) {
        match to_panel {
            Panels::SonglistCandidates => {
                self.current_focus_panel = FocusPanel::SonglistCandidatesInside;
                self.songlist_candidates_panel.focused_status = PanelFocusedStatus::Inside;
                self.songlist_content_panel.focused_status = PanelFocusedStatus::Nop;
            },
            Panels::SonglistContent => {
                self.current_focus_panel = FocusPanel::SonglistContentInside;
                self.songlist_candidates_panel.focused_status = PanelFocusedStatus::Nop;
                self.songlist_content_panel.focused_status = PanelFocusedStatus::Inside;
            },
        }
    }
}
