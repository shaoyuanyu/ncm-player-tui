use ratatui::widgets::{List, ListState};

#[derive(Default)]
pub struct UIList<'a> {
    /// The TUI List widget
    pub list: List<'a>,

    /// The List widget state
    pub state: ListState,
}
