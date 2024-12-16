use crate::config::style::*;
use crate::config::{Command, ScreenEnum};
use crate::ui::Controller;
use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::palette::tailwind;
use ratatui::text::Line;
use ratatui::widgets::Tabs;
use ratatui::Frame;
use tui_textarea::{CursorMove, TextArea};
use unicode_width::UnicodeWidthStr;

const NORMAL_TEXT: &str = " NORMAL ";
const COMMAND_TEXT: &str = " COMMAND ";
const SEARCH_TEXT: &str = " SEARCH ";

pub struct CommandLine<'a> {
    //
    show_cursor: bool,
    show_colon: bool,
    //
    current_mode: String,

    //
    mode_label: Line<'a>,
    colon_line: Line<'a>,
    interactive_area: TextArea<'a>,
    tabs: Tabs<'a>,
}

///
impl<'a> CommandLine<'a> {
    pub fn new() -> Self {
        Self {
            show_cursor: false,
            show_colon: false,
            current_mode: String::from(NORMAL_TEXT),
            mode_label: Line::default(),
            colon_line: Line::default(),
            interactive_area: TextArea::default(),
            tabs: Tabs::new(vec!["1.播放", "2.歌单", "0.help", "登录"])
                .highlight_style(ITEM_SELECTED_STYLE)
                .padding("", "")
                .select(0)
                .italic(),
        }
    }
}

/// public
impl<'a> CommandLine<'a> {
    pub fn set_to_normal_mode(&mut self) {
        self.clear_content();
        self.show_cursor = false;
        self.show_colon = false;
        self.current_mode = String::from(NORMAL_TEXT);
    }

    pub fn set_to_command_line_mode(&mut self) {
        self.clear_content();
        self.show_cursor = true;
        self.show_colon = true;
        self.current_mode = String::from(COMMAND_TEXT);
    }

    pub fn set_to_search_mode(&mut self) {
        self.clear_content();
        self.show_cursor = true;
        self.show_colon = false;
        self.current_mode = String::from(SEARCH_TEXT);
    }

    pub fn get_content(&self) -> String {
        self.interactive_area.lines()[0].clone()
    }

    pub fn set_content(&mut self, content: &str) {
        self.clear_content();
        self.interactive_area.insert_str(content);
    }

    pub fn clear_content(&mut self) {
        self.interactive_area.move_cursor(CursorMove::End);
        while self.interactive_area.delete_char() {}
    }

    pub fn is_content_empty(&self) -> bool {
        self.interactive_area.is_empty()
    }

    pub fn input(&mut self, input: KeyEvent) {
        self.interactive_area.input(input);
    }
}

///
impl<'a> Controller for CommandLine<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        Ok(true)
    }

    async fn handle_event(&mut self, cmd: Command) -> Result<bool> {
        self.tabs = match cmd {
            Command::GotoScreen(to_screen) => match to_screen {
                ScreenEnum::Main => self.tabs.to_owned().select(0),
                ScreenEnum::Songlists => self.tabs.to_owned().select(1),
                ScreenEnum::Help => self.tabs.to_owned().select(2),
                ScreenEnum::Login => self.tabs.to_owned().select(3),
                _ => self.tabs.to_owned().select(None),
            },
            _ => self.tabs.to_owned(),
        };

        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        //
        self.mode_label = Line::from(self.current_mode.clone()).bold().italic().fg(match self.current_mode.as_str() {
            NORMAL_TEXT => tailwind::RED.c600,
            COMMAND_TEXT => tailwind::YELLOW.c600,
            SEARCH_TEXT => tailwind::BLUE.c600,
            _ => tailwind::BLACK,
        });

        //
        self.colon_line = Line::from(if self.show_colon { ": " } else { "" }).style(style.add_modifier(Modifier::BOLD));

        //
        self.interactive_area.set_style(*style);
        self.interactive_area.set_cursor_line_style(*style);
        if self.show_cursor {
            self.interactive_area.set_cursor_style(Style::default().bg(tailwind::SLATE.c700));
        } else {
            self.interactive_area.set_cursor_style(Style::default().add_modifier(Modifier::HIDDEN));
        }
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(UnicodeWidthStr::width(self.current_mode.as_str()) as u16),
                Constraint::Max(UnicodeWidthStr::width(if self.show_colon { ": " } else { "" }) as u16),
                Constraint::Fill(1),
                Constraint::Max(25),
            ])
            .split(chunk);

        // mode_line
        frame.render_widget(&self.mode_label, chunks[0]);

        // colon_line
        frame.render_widget(&self.colon_line, chunks[1]);

        // interactive_area
        frame.render_widget(&self.interactive_area, chunks[2]);

        // tabs
        frame.render_widget(&self.tabs, chunks[3]);
    }
}
