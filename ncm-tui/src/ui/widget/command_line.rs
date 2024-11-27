use crate::ui::Controller;
use ratatui::prelude::*;
use tui_textarea::{CursorMove, TextArea};
use unicode_width::UnicodeWidthStr;
use crate::config::Command;

#[derive(Default)]
pub struct CommandLine<'a> {
    prompt: String,

    prompt_line: Line<'a>,
    pub textarea: TextArea<'a>,

    show_cursor: bool,
}

impl<'a> CommandLine<'a> {
    pub fn new(normal_style: &Style) -> Self {
        let mut cmd = CommandLine {
            prompt: String::new(),
            prompt_line: Line::default(),
            textarea: TextArea::default(),
            show_cursor: false,
        };
        cmd.update_view(normal_style);
        cmd
    }

    pub fn get_contents(&self) -> String {
        self.textarea.lines()[0].clone()
    }

    pub fn reset(&mut self) {
        self.clear_prompt();
        self.clear_contents();
    }

    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.into();
    }

    pub fn clear_prompt(&mut self) {
        self.prompt = "".into();
    }

    pub fn clear_contents(&mut self) {
        self.textarea.move_cursor(CursorMove::End);
        while self.textarea.delete_char() {}
    }

    pub fn set_cursor_visibility(&mut self, show_cursor: bool) {
        self.show_cursor = show_cursor;
    }
}

impl<'a> Controller for CommandLine<'a> {
    async fn handle_event(&mut self, _cmd: Command) -> anyhow::Result<()> {
        Ok(())
    }

    async fn update_model(&mut self) -> anyhow::Result<bool> {
        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        self.textarea.set_style(*style);
        self.textarea.set_cursor_line_style(*style);
        if !self.show_cursor {
            self.textarea
                .set_cursor_style(Style::default().add_modifier(Modifier::HIDDEN));
        }

        self.prompt_line = Line::styled(self.prompt.clone(), style.add_modifier(Modifier::BOLD));
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                #[allow(clippy::cast_possible_truncation)]
                [
                    Constraint::Max(UnicodeWidthStr::width(self.prompt.clone().as_str()) as u16),
                    Constraint::Fill(1),
                ]
                .as_ref(),
            )
            .split(chunk);

        frame.render_widget(&self.prompt_line, chunks[0]);
        frame.render_widget(&self.textarea, chunks[1]);
    }
}
