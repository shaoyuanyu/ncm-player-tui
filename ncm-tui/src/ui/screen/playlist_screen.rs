use crate::config::{Command, ScreenEnum};
use crate::ui::panel::{PanelFocusedStatus, PlaylistCandidatePanel, PlaylistPanel};
use crate::ui::Controller;
use crate::{command_queue, ncm_client, player};
use log::debug;
use ncm_api::model::Songlist;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Style;
use ratatui::Frame;

#[derive(PartialEq)]
enum Panels {
    PlaylistCandidate,
    PlaylistContent,
}

#[derive(PartialEq)]
enum FocusPanel {
    PlaylistCandidateOutside,
    PlaylistCandidateInside,
    PlaylistContentOutside,
    PlaylistContentInside,
}

pub struct PlaylistsScreen<'a> {
    current_focus_panel: FocusPanel,
    //
    current_selected_songlist: Option<Songlist>,
    //
    playlist_candidate_panel: PlaylistCandidatePanel<'a>,
    playlist_content_panel: PlaylistPanel<'a>,
}

impl<'a> PlaylistsScreen<'a> {
    pub fn new(_normal_style: &Style) -> Self {
        Self {
            current_focus_panel: FocusPanel::PlaylistCandidateOutside,
            current_selected_songlist: None,
            playlist_candidate_panel: PlaylistCandidatePanel::new(PanelFocusedStatus::Outside),
            playlist_content_panel: PlaylistPanel::new(PanelFocusedStatus::Nop),
        }
    }
}

impl<'a> Controller for PlaylistsScreen<'a> {
    async fn update_model(&mut self) -> anyhow::Result<bool> {
        let mut result = Ok(false);

        // playlist candidate
        if self.playlist_candidate_panel.update_model().await? {
            result = Ok(true);
        }

        // playlist content
        if self.playlist_content_panel.update_model().await? {
            result = Ok(true);
        }

        result
    }

    async fn handle_event(&mut self, cmd: Command) -> anyhow::Result<bool> {
        use Command::*;
        use FocusPanel::*;

        match (cmd.clone(), &self.current_focus_panel) {
            //
            (Esc, PlaylistCandidateInside) => {
                self.focus_panel_outside(Panels::PlaylistCandidate);
            }
            (Esc, PlaylistContentInside) => {
                self.focus_panel_outside(Panels::PlaylistContent);
            }
            //
            (Down | Up, PlaylistCandidateOutside) => {
                self.focus_panel_inside(Panels::PlaylistCandidate);
            }
            (Down | Up, PlaylistContentOutside) => {
                self.focus_panel_inside(Panels::PlaylistContent);
            }
            (Down | Up, PlaylistCandidateInside) => {
                self.playlist_candidate_panel.handle_event(cmd).await?;
            }
            (Down | Up, PlaylistContentInside) => {
                self.playlist_content_panel.handle_event(cmd).await?;
            }
            //
            (NextPanel, PlaylistCandidateOutside) => {
                self.focus_panel_outside(Panels::PlaylistContent);
            }
            (PrevPanel, PlaylistContentOutside) => {
                self.focus_panel_outside(Panels::PlaylistCandidate);
            }
            //
            (EnterOrPlay, PlaylistCandidateOutside) => {
                self.focus_panel_inside(Panels::PlaylistCandidate);
            }
            (EnterOrPlay, PlaylistContentOutside) => {
                self.focus_panel_inside(Panels::PlaylistContent);
            }
            (EnterOrPlay, PlaylistCandidateInside) => {
                // 加载歌单
                if let Some(mut selected_songlist) =
                    self.playlist_candidate_panel.get_selected_songlist()
                {
                    ncm_client
                        .lock()
                        .await
                        .load_songlist_songs(&mut selected_songlist)
                        .await?;

                    self.playlist_content_panel
                        .set_model(&selected_songlist.name, &selected_songlist.songs);

                    self.current_selected_songlist = Some(selected_songlist);
                }
            }
            (EnterOrPlay, PlaylistContentInside) => {
                // TODO: 开始播放
            }
            //
            (Play, PlaylistCandidateInside) => {
                debug!("shift + enter");

                if let Some(selected_songlist_index) =
                    self.playlist_candidate_panel.get_selected_songlist_index()
                {
                    // 切换当前播放列表
                    player
                        .lock()
                        .await
                        .switch_playlist(selected_songlist_index, ncm_client.lock().await)
                        .await?;

                    // 返回 main_screen ，刷新 playlist 显示，开始自动播放
                    debug!("switch playlist finished");
                    let mut command_queue_guard = command_queue.lock().await;
                    command_queue_guard.push_back(GotoScreen(ScreenEnum::Main));
                    command_queue_guard.push_back(RefreshPlaylist);
                    command_queue_guard.push_back(StartPlay);
                    drop(command_queue_guard);
                }
            }
            //
            (GoToTop | GoToBottom, PlaylistCandidateOutside | PlaylistCandidateInside) => {
                self.playlist_candidate_panel.handle_event(cmd).await?;
            }
            (GoToTop | GoToBottom, PlaylistContentOutside | PlaylistContentInside) => {
                self.playlist_content_panel.handle_event(cmd).await?;
                self.focus_panel_inside(Panels::PlaylistContent);
            }
            //
            (
                SearchForward(_) | SearchBackward(_),
                PlaylistCandidateOutside | PlaylistCandidateInside,
            ) => {
                self.playlist_candidate_panel.handle_event(cmd).await?;
            }
            (
                SearchForward(_) | SearchBackward(_),
                PlaylistContentOutside | PlaylistContentInside,
            ) => {
                self.playlist_content_panel.handle_event(cmd).await?;
                self.focus_panel_inside(Panels::PlaylistContent);
            }
            //
            (_, _) => {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        self.playlist_candidate_panel.update_view(style);

        self.playlist_content_panel.update_view(style);
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        // 分为左右两个面板
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(chunk);

        // 在左半屏渲染 playlist_candidate_panel
        self.playlist_candidate_panel.draw(frame, chunks[0]);

        // 在右半屏渲染 playlist_content_panel
        self.playlist_content_panel.draw(frame, chunks[1]);
    }
}

/// private
impl<'a> PlaylistsScreen<'a> {
    fn focus_panel_outside(&mut self, to_panel: Panels) {
        match to_panel {
            Panels::PlaylistCandidate => {
                self.current_focus_panel = FocusPanel::PlaylistCandidateOutside;
                self.playlist_candidate_panel.focused_status = PanelFocusedStatus::Outside;
                self.playlist_content_panel.focused_status = PanelFocusedStatus::Nop;
            }
            Panels::PlaylistContent => {
                self.current_focus_panel = FocusPanel::PlaylistContentOutside;
                self.playlist_candidate_panel.focused_status = PanelFocusedStatus::Nop;
                self.playlist_content_panel.focused_status = PanelFocusedStatus::Outside;
            }
        }
    }

    fn focus_panel_inside(&mut self, to_panel: Panels) {
        match to_panel {
            Panels::PlaylistCandidate => {
                self.current_focus_panel = FocusPanel::PlaylistCandidateInside;
                self.playlist_candidate_panel.focused_status = PanelFocusedStatus::Inside;
                self.playlist_content_panel.focused_status = PanelFocusedStatus::Nop;
            }
            Panels::PlaylistContent => {
                self.current_focus_panel = FocusPanel::PlaylistContentInside;
                self.playlist_candidate_panel.focused_status = PanelFocusedStatus::Nop;
                self.playlist_content_panel.focused_status = PanelFocusedStatus::Inside;
            }
        }
    }
}
