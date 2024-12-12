use crate::config::Command;
use crate::ui::panel::{LyricPanel, PanelFocusedStatus, PlaylistPanel};
use crate::ui::Controller;
use anyhow::Result;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::Frame;

#[derive(PartialEq)]
enum Panels {
    Playlist,
    Lyric,
}

#[derive(PartialEq)]
enum FocusPanel {
    PlaylistOutside,
    PlaylistInside,
    LyricOutside,
    LyricInside,
}

pub struct MainScreen<'a> {
    current_focus_panel: FocusPanel,
    //
    playlist_panel: PlaylistPanel<'a>,
    lyric_panel: LyricPanel<'a>,
}

impl<'a> MainScreen<'a> {
    pub fn new(_normal_style: &Style) -> Self {
        Self {
            current_focus_panel: FocusPanel::PlaylistOutside,
            playlist_panel: PlaylistPanel::new(PanelFocusedStatus::Outside),
            lyric_panel: LyricPanel::new(PanelFocusedStatus::Nop),
        }
    }
}

impl<'a> Controller for MainScreen<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        let mut result = Ok(false);

        // playlist
        if self.playlist_panel.update_model().await? {
            result = Ok(true);
        }

        // song
        if self.lyric_panel.update_model().await? {
            result = Ok(true);
        }

        result
    }

    async fn handle_event(&mut self, cmd: Command) -> Result<bool> {
        use Command::*;
        use FocusPanel::*;

        match (cmd.clone(), &self.current_focus_panel) {
            //
            (Esc, PlaylistInside) => {
                self.focus_panel_outside(Panels::Playlist);
            }
            (Esc, LyricInside) => {
                self.focus_panel_outside(Panels::Lyric);
            }
            //
            (Down | Up, PlaylistOutside) => {
                self.focus_panel_inside(Panels::Playlist);
            }
            (Down | Up, LyricOutside) => {
                self.focus_panel_inside(Panels::Lyric);
            }
            (Down | Up, PlaylistInside) => {
                self.playlist_panel.handle_event(cmd).await?;
            }
            (Down | Up, LyricInside) => {
                self.lyric_panel.handle_event(cmd).await?;
            }
            //
            (NextPanel, PlaylistOutside) => {
                self.focus_panel_outside(Panels::Lyric);
            }
            (PrevPanel, LyricOutside) => {
                self.focus_panel_outside(Panels::Playlist);
            }
            //
            (Play, PlaylistOutside) => {
                self.focus_panel_inside(Panels::Playlist);
            }
            (Play, LyricOutside) => {
                self.focus_panel_inside(Panels::Lyric);
            }
            (Play, PlaylistInside) => {
                self.playlist_panel.handle_event(cmd).await?;
            }
            (Play, LyricInside) => {
                self.lyric_panel.handle_event(cmd).await?;
                self.focus_panel_outside(Panels::Lyric);
            }
            //
            (WhereIsThisSong, _) => {
                self.playlist_panel.handle_event(cmd).await?;
                self.focus_panel_inside(Panels::Playlist);
            }
            //
            (GoToTop | GoToBottom, PlaylistOutside | PlaylistInside) => {
                self.playlist_panel.handle_event(cmd).await?;
                self.focus_panel_inside(Panels::Playlist);
            }
            (GoToTop | GoToBottom, LyricOutside | LyricInside) => {
                self.lyric_panel.handle_event(cmd).await?;
                self.focus_panel_inside(Panels::Lyric);
            }
            //
            (SearchForward(_) | SearchBackward(_), _) => {
                self.playlist_panel.handle_event(cmd).await?;
                self.focus_panel_inside(Panels::Playlist);
            }
            //
            (_, _) => return Ok(false),
        }

        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        self.playlist_panel.update_view(style);

        self.lyric_panel.update_view(style);
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        // 分为左右两个面板
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunk);

        // 在左半屏渲染 playlist_table
        self.playlist_panel.draw(frame, chunks[0]);

        // 在右半屏渲染 current_song
        self.lyric_panel.draw(frame, chunks[1]);
    }
}

/// private
impl<'a> MainScreen<'a> {
    fn focus_panel_outside(&mut self, to_panel: Panels) {
        match to_panel {
            Panels::Playlist => {
                self.current_focus_panel = FocusPanel::PlaylistOutside;
                self.playlist_panel.focused_status = PanelFocusedStatus::Outside;
                self.lyric_panel.focused_status = PanelFocusedStatus::Nop;
            }
            Panels::Lyric => {
                self.current_focus_panel = FocusPanel::LyricOutside;
                self.playlist_panel.focused_status = PanelFocusedStatus::Nop;
                self.lyric_panel.focused_status = PanelFocusedStatus::Outside;
            }
        }
    }

    fn focus_panel_inside(&mut self, to_panel: Panels) {
        match to_panel {
            Panels::Playlist => {
                self.current_focus_panel = FocusPanel::PlaylistInside;
                self.playlist_panel.focused_status = PanelFocusedStatus::Inside;
                self.lyric_panel.focused_status = PanelFocusedStatus::Nop;
            }
            Panels::Lyric => {
                self.current_focus_panel = FocusPanel::LyricInside;
                self.playlist_panel.focused_status = PanelFocusedStatus::Nop;
                self.lyric_panel.focused_status = PanelFocusedStatus::Inside;
            }
        }
    }
}
