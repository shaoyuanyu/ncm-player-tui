mod lyric_panel;
mod playlist_panel;
mod songlist_candidates_panel;

pub use lyric_panel::*;
pub use playlist_panel::*;
pub use songlist_candidates_panel::*;

#[derive(PartialEq)]
/// 面板是否被聚焦，聚焦在面板整体还是面板内
pub enum PanelFocusedStatus {
    Outside,
    Inside,
    Nop,
}
