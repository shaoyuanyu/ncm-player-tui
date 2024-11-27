use anyhow::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::widgets::{Paragraph};
use crate::config::Command;
use crate::ui::Controller;
use crate::ui::widget::UIList;

enum Panel {
    Playlists,
    Tracks,
}

pub struct PlaylistScreen<'a> {
    //
    playlist: Vec<String>,

    //
    playlist_list: UIList<'a>,
    track_page: Paragraph<'a>,
    panel: Panel,
}

impl<'a> PlaylistScreen<'a> {
}

impl<'a> Controller for PlaylistScreen<'a> {
    async fn handle_event(&mut self, _cmd: Command) -> Result<()> {
        todo!()
    }

    async fn update_model(&mut self) -> Result<bool> {
        todo!()
    }

    fn update_view(&mut self, _style: &Style) {
        todo!()
    }

    fn draw(&self, _frame: &mut Frame, _chunk: Rect) {
        todo!()
    }
}