mod lyric_panel;
mod playlist_candidate_panel;
mod playlist_panel;

pub use lyric_panel::*;
pub use playlist_candidate_panel::*;
pub use playlist_panel::*;

use ratatui::prelude::{Modifier, Style};
use ratatui::style::palette::tailwind;

#[derive(PartialEq)]
/// 面板是否被聚焦，聚焦在面板整体还是面板内
pub enum PanelFocusedStatus {
    Outside,
    Inside,
    Nop,
}

const PANEL_SELECTED_BORDER_STYLE: Style = Style::new().fg(tailwind::RED.c800);

const ITEM_SELECTED_STYLE: Style = Style::new()
    .bg(tailwind::RED.c400)
    .add_modifier(Modifier::BOLD);

const LYRIC_FOCUSED_STYLE: Style = Style::new()
    .fg(tailwind::RED.c600)
    .add_modifier(Modifier::BOLD);
