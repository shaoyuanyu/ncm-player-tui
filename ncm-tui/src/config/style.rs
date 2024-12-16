use ratatui::prelude::{Modifier, Style};
use ratatui::style::palette::tailwind;

pub const PANEL_SELECTED_BORDER_STYLE: Style = Style::new().fg(tailwind::RED.c700).add_modifier(Modifier::BOLD);

pub const ITEM_SELECTED_STYLE: Style = Style::new().bg(tailwind::RED.c400).add_modifier(Modifier::BOLD);

pub const LYRIC_FOCUSED_STYLE: Style = Style::new().fg(tailwind::RED.c600).add_modifier(Modifier::BOLD);

pub const TABLE_HEADER_STYLE: Style = Style::new().fg(tailwind::WHITE).bg(tailwind::RED.c300);
