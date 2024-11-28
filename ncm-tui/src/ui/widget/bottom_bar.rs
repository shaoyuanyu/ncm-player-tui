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

    // view
    info_bar: Paragraph<'a>,
    playback_bar: Gauge<'a>,
}

impl<'a> BottomBar<'a> {
    pub fn new(_normal_style: &Style) -> Self {
        let info_bar = Paragraph::default();

        let playback_bar = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(tailwind::PINK.c300)
            .ratio(0.0)
            .label("--:--/--:--");

        Self {
            info_bar_text: Text::default(),
            info_bar,
            playback_bar,
        }
    }
}

impl<'a> Controller for BottomBar<'a> {
    /// bottom_bar 由于更新频繁，略去从 model 到 view 的流程，直接在 update_model() 阶段更新
    async fn update_model(&mut self) -> Result<bool> {
        let player_guard = PLAYER.lock().await;

        // info_bar
        self.info_bar_text = Text::from(
            format!(
                "  {}  {}  {}  |  {}  ",
                '\u{f0456}',
                if player_guard.is_playing() {
                    CHAR_FLAG_PAUSE
                } else {
                    CHAR_FLAG_PLAY
                },
                '\u{f049d}',
                player_guard.play_state()
            )
        );

        // playback_bar
        if let Some(player_position) = player_guard.position() {
            if let Some(player_duration) = player_guard.duration() {
                self.playback_bar = self
                    .playback_bar
                    .clone()
                    .ratio(player_position.mseconds() as f64 / player_duration.mseconds() as f64)
                    .label(format!(
                        "{:02}:{:02}/{:02}:{:02}",
                        player_position.minutes(),
                        player_position.seconds() % 60,
                        player_duration.minutes(),
                        player_duration.seconds() % 60,
                    ));
            }
        }

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
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let bottom_bar_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(26), Constraint::Min(3)].as_ref())
            .split(chunk);

        // info_bar
        frame.render_widget(&self.info_bar, bottom_bar_chunks[0]);

        // playback_bar
        frame.render_widget(&self.playback_bar, bottom_bar_chunks[1]);
    }
}
