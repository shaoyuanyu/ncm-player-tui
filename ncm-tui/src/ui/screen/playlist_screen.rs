use crate::config::Command;
use crate::ui::widget::UIList;
use crate::ui::Controller;
use anyhow::Result;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub struct PlaylistScreen<'a> {
    //
    playlist: Vec<String>,

    //
    playlist_list: UIList<'a>,
    track_page: Paragraph<'a>,
}

impl<'a> PlaylistScreen<'a> {}

impl<'a> Controller for PlaylistScreen<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        todo!()
    }

    async fn handle_event(&mut self, _cmd: Command) -> Result<bool> {
        Ok(false)
    }

    fn update_view(&mut self, _style: &Style) {
        todo!()
    }

    fn draw(&self, _frame: &mut Frame, _chunk: Rect) {
        todo!()
    }
}
