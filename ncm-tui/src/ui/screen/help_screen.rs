use crate::config::Command;
use crate::ui::Controller;
use anyhow::Result;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub struct HelpScreen<'a> {
    // view
    help_page: Paragraph<'a>,
}

impl<'a> HelpScreen<'a> {
    pub fn new(normal_style: &Style) -> Self {
        let help_text = Text::from(
            "Up:                \n\
             Down:              \n\
             Play/Pause:        \n\
             Previous track:    \n\
             Next track:        \n\
             Enqueue:           \n\
             Repeat:            \n\
             Shuffle:           \n\
             Goto top:          \n\
             Goto bottom:       \n\
             Next panel:        \n\
             Previous panel:    \n\
             Main screen:       \n\
             Playlist screen:   \n\
             Help screen:       \n\
             New playlist:       (Playlist screen only)\n\
             Select playlist:    (Playlist screen only)\n\
             Add to playlist:   \n\
             Quit:              ",
        );
        let help_page = Paragraph::new(help_text)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .style(*normal_style);

        Self { help_page }
    }
}

impl<'a> Controller for HelpScreen<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        Ok(false)
    }

    async fn handle_event(&mut self, _cmd: Command) -> Result<bool> {
        Ok(false)
    }

    fn update_view(&mut self, _style: &Style) {}

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        frame.render_widget(&self.help_page, chunk);
    }
}
