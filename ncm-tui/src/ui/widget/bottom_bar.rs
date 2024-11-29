use crate::config::Command;
use crate::ui::Controller;
use crate::PLAYER;
use anyhow::Result;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Constraint, Direction, Style};
use ratatui::style::palette::tailwind;
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

//
const CHAR_FLAG_PLAY: char = '\u{f040a}';
const CHAR_FLAG_PAUSE: char = '\u{f03e4}';
//
const CHAR_FLAG_SHUFFLE: char = '\u{f049d}';
const CHAR_FLAG_SHUFFLE_OFF: char = '\u{f049e}';
//
const CHAR_FLAG_REPEAT: char = '\u{f0456}';
const CHAR_FLAG_REPEAT_OFF: char = '\u{f0457}';
const CHAR_FLAG_REPEAT_ONCE: char = '\u{f0458}';

pub struct BottomBar<'a> {
    // model
    info_bar_text: Text<'a>,
    //
    playback_ratio: f64,
    playback_label: String,
    //
    volume: f64,

    // view
    info_bar: Paragraph<'a>,
    playback_bar: Gauge<'a>,
    volume_bar: Gauge<'a>,
}

impl<'a> BottomBar<'a> {
    pub fn new(_normal_style: &Style) -> Self {
        Self {
            info_bar_text: Text::default(),
            playback_ratio: 0.0,
            playback_label: String::new(),
            volume: 0.0,
            info_bar: Paragraph::default(),
            playback_bar: Gauge::default(),
            volume_bar: Gauge::default(),
        }
    }
}

impl<'a> Controller for BottomBar<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        let player_guard = PLAYER.lock().await;

        // info_bar
        self.info_bar_text = Text::from(format!(
            "  {}  {}  {}  |  {}  ",
            '\u{f0456}',
            if player_guard.is_playing() {
                CHAR_FLAG_PAUSE
            } else {
                CHAR_FLAG_PLAY
            },
            '\u{f049d}',
            player_guard.play_state()
        ));

        // playback_bar
        if let (Some(player_position), Some(player_duration)) =
            (player_guard.position(), player_guard.duration())
        {
            self.playback_ratio =
                player_position.mseconds() as f64 / player_duration.mseconds() as f64;
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

        // volume_bar
        self.volume = player_guard.volume();

        // bottom_bar 一直保持更新
        Ok(true)
    }

    async fn handle_event(&mut self, _cmd: Command) -> Result<bool> {
        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        self.info_bar = Paragraph::new(self.info_bar_text.clone())
            .block(Block::default().borders(Borders::ALL))
            .style(*style);

        self.playback_bar = Gauge::default()
            .block(Block::default().borders(Borders::ALL).style(*style))
            .gauge_style(tailwind::PINK.c300)
            .ratio(self.playback_ratio)
            .label(self.playback_label.clone());

        self.volume_bar = Gauge::default()
            .block(
                Block::default()
                    .title("Volume")
                    .borders(Borders::ALL)
                    .style(*style),
            )
            .gauge_style(tailwind::BLUE.c400)
            .ratio(self.volume);
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let bottom_bar_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(26),
                    Constraint::Min(3),
                    Constraint::Length(26),
                ]
                .as_ref(),
            )
            .split(chunk);

        // info_bar
        frame.render_widget(&self.info_bar, bottom_bar_chunks[0]);

        // playback_bar
        frame.render_widget(&self.playback_bar, bottom_bar_chunks[1]);

        // volume_bar
        frame.render_widget(&self.volume_bar, bottom_bar_chunks[2]);
    }
}
