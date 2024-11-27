mod app;
pub mod screen;
pub mod widget;

pub use app::App;

use anyhow::Result;
use ratatui::prelude::*;
use crate::config::Command;

trait Controller {
    // handle_event() is often a part of updating model
    async fn handle_event(&mut self, cmd: Command) -> Result<()>;

    /// return false if no model changed (not need to redraw)
    async fn update_model(&mut self) -> Result<bool>;

    fn update_view(&mut self, style: &Style);

    fn draw(&self, frame: &mut Frame, chunk: Rect);
}
