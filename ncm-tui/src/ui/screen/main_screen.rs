use std::mem;
use crate::ui::Controller;
use anyhow::Result;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;
use ratatui::style::palette::tailwind::SLATE;
use crate::config::Command;
use crate::NCM_API;
use crate::ui::widget::UIList;

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub struct MainScreen<'a> {
    // model
    user_name: String,
    playlist_name: String,
    playlist: Vec<String>, // TODO: Playlist
    playlist_items: Vec<ListItem<'a>>,
    current_song: String, // TODO: Song

    // view
    playlist_ui: UIList<'a>,
    song_ui: Paragraph<'a>,
}

impl<'a> MainScreen<'a> {
    pub fn new(normal_style: &Style) -> Self {
        Self {
            user_name: String::new(),
            playlist_name: String::new(),
            playlist: Vec::new(),
            playlist_items: Vec::new(),
            current_song: String::new(),
            playlist_ui: UIList::default(),
            song_ui: Paragraph::new("this is a song")
                .block(Block::default().title("Song name").borders(Borders::ALL))
                .style(*normal_style),
        }
    }

    pub fn update_playlist_model(&mut self, play_list_name: String, playlist: Vec<String>) {
        self.playlist_name = play_list_name.to_string();
        self.playlist = playlist;
        self.playlist_items = self.playlist
            .iter()
            .map(|song| ListItem::new(song.clone()))
            .collect();
    }
}

impl<'a> Controller for MainScreen<'a> {
    async fn handle_event(&mut self, cmd: Command) -> Result<()> {
        let list_len = self.playlist_ui.list.len();
        if list_len == 0 { return Ok(()); }

        let list_state = &mut self.playlist_ui.state;
        let mut selected = list_state.selected().unwrap_or_default();

        match cmd {
            Command::Up => {
                if selected == 0 {
                    selected = list_len - 1;
                } else {
                    selected -= 1;
                }
            },
            Command::Down => {
                if selected == list_len - 1 {
                    selected = 0;
                } else {
                    selected += 1;
                }
            },
            _ => {},
        };

        self.playlist_ui.state.select(Some(selected));

        Ok(())
    }

    async fn update_model(&mut self) -> Result<bool> {
        let mut result = Ok(false);

        // username
        if self.user_name.is_empty() {
            if let Some(login_info) = NCM_API.lock().await.login_info() {
                self.user_name = login_info.nickname;
            }

            result = Ok(true);
        }

        result
    }

    fn update_view(&mut self, style: &Style) {
        self.playlist_ui = UIList {
            list: List::new(self.playlist_items.clone())
                .block(
                    Block::default()
                        .title(format!("Playlist: {}", self.playlist_name.clone()))
                        .title_bottom(format!("User: {}", self.user_name.clone()))
                        .borders(Borders::ALL)
                )
                .highlight_style(SELECTED_STYLE)
                .highlight_symbol(">")
                .style(*style),
            state: mem::take(&mut self.playlist_ui.state),
        };
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        // Split the screen into left and right halves
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunk);

        // 在左半屏渲染 playlist
        let mut playlist_ui_state = self.playlist_ui.state.clone();
        frame.render_stateful_widget(&self.playlist_ui.list, chunks[0], &mut playlist_ui_state);

        // 在右半屏渲染 current_song
        frame.render_widget(&self.song_ui, chunks[1]);
    }
}
