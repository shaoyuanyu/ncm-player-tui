use crate::config::Command;
use crate::player;
use crate::ui::Controller;
use anyhow::Result;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Constraint, Direction, Style};
use ratatui::style::palette::tailwind;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

pub struct BottomBar<'a> {
    // model
    info_bar_text: Text<'a>,
    //
    playback_ratio: f64,
    playback_label: String,
    song_name: Option<String>,
    singer_name: Option<String>,
    song_quality_level: Option<String>,
    //
    volume: f64,

    // view
    control_bar: Paragraph<'a>,
    playback_bar: Gauge<'a>,
    volume_bar: Gauge<'a>,
}

impl<'a> BottomBar<'a> {
    pub fn new(_normal_style: &Style) -> Self {
        Self {
            info_bar_text: Text::default(),
            playback_ratio: 0.0,
            playback_label: String::new(),
            song_name: None,
            singer_name: None,
            song_quality_level: None,
            volume: 0.0,
            control_bar: Paragraph::default(),
            playback_bar: Gauge::default(),
            volume_bar: Gauge::default(),
        }
    }
}

impl<'a> Controller for BottomBar<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        let player_guard = player.lock().await;

        // control_bar
        self.info_bar_text = Text::from(Line::from(format!(" {}  |  {}  ", player_guard.play_mode(), if player_guard.is_playing() { '\u{f03e4}' } else { '\u{f040a}' },)).centered());

        // playback_bar
        if let (Some(player_position), Some(player_duration)) = (player_guard.position(), player_guard.duration()) {
            self.playback_ratio = if player_position.mseconds() as f64 / player_duration.mseconds() as f64 <= 1.0 {
                player_position.mseconds() as f64 / player_duration.mseconds() as f64
            } else {
                1.0
            };
            self.playback_label = format!(
                "{:02}:{:02}/{:02}:{:02}",
                player_position.minutes(),
                player_position.seconds() % 60,
                player_duration.minutes(),
                player_duration.seconds() % 60,
            );
        } else {
            self.playback_ratio = 0.0;
            self.playback_label = String::from("--:--/--:--");
        };
        if let Some(song) = player_guard.current_song().clone() {
            self.song_name = Some(song.name.clone());
            self.singer_name = Some(song.singer.clone());
            self.song_quality_level = Some(song.quality_level.clone());
        }

        // volume_bar
        self.volume = player_guard.volume();

        // bottom_bar 一直保持更新
        Ok(true)
    }

    async fn handle_event(&mut self, _cmd: Command) -> Result<bool> {
        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        self.control_bar = Paragraph::new(self.info_bar_text.clone()).block(Block::default().borders(Borders::ALL)).style(*style);

        self.playback_bar = Gauge::default()
            .block({
                let mut block = Block::default().borders(Borders::ALL).style(*style);
                if let (Some(song_name), Some(artist_name), Some(song_quality_level)) = (self.song_name.clone(), self.singer_name.clone(), self.song_quality_level.clone()) {
                    block = block
                        .title_top(Line::from(format!("{}", song_name)).centered())
                        .title_bottom(Line::from(format!("{}", artist_name)).centered())
                        .title_bottom(Line::from(format!("音质:{}", song_quality_level)).right_aligned());
                }
                block
            })
            .gauge_style(tailwind::PINK.c300)
            .ratio(self.playback_ratio)
            .label(self.playback_label.clone());

        self.volume_bar = Gauge::default()
            .block(Block::default().title("Volume").borders(Borders::ALL).style(*style))
            .gauge_style(tailwind::BLUE.c400)
            .ratio(self.volume);
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let bottom_bar_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(20), Constraint::Min(10), Constraint::Length(20)].as_ref())
            .split(chunk);

        // control_bar
        frame.render_widget(&self.control_bar, bottom_bar_chunks[0]);

        // playback_bar
        frame.render_widget(&self.playback_bar, bottom_bar_chunks[1]);

        // volume_bar
        frame.render_widget(&self.volume_bar, bottom_bar_chunks[2]);
    }
}
