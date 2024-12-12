mod app;
mod panel;
pub mod screen;
pub mod widget;

pub use app::App;

use crate::config::Command;
use anyhow::Result;
use ratatui::prelude::*;

trait Controller {
    /// model 未变化的情况下返回 false，此时程序无需重新更新 view
    async fn update_model(&mut self) -> Result<bool>;

    /// handle_event() 通常是 model 更新逻辑的一部分，为简化代码从 update_model() 中独立出来
    ///
    /// 主程序内先执行 update_model() 再执行 handle_event()
    ///
    /// model 未变化的情况下返回 false，此时程序无需重新更新 view
    async fn handle_event(&mut self, cmd: Command) -> Result<bool>;

    /// 从 model 更新 view
    fn update_view(&mut self, style: &Style);

    /// 渲染到屏幕
    fn draw(&self, frame: &mut Frame, chunk: Rect);
}
