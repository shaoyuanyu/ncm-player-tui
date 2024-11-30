use crate::config::Command;
use crate::ui::Controller;
use anyhow::Result;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub struct HelpScreen<'a> {
    // view
    normal_mode_help_page: Paragraph<'a>,
    commandline_mode_help_page: Paragraph<'a>,
}

impl<'a> HelpScreen<'a> {
    pub fn new(normal_style: &Style) -> Self {
        let normal_mode_help_text = Text::from(format!(
            "\
            Up:                                     {}\n\
            Down:                                   {}\n\
            Play/Pause:                             {}\n\
            Previous Panel:                         {}\n\
            Next Panel:                             {}\n\
            Go To Main Screen:                      {}\n\
            Go To Help Screen (Here):               {}\n\
            Play Next Song:                         {}\n\
            *Switch To Command Line Mode:           {}\n\
            Quit:                                   {}",
            "↑ / k",
            "↓ / j",
            "\u{2423} (Space)",
            "←",
            "→",
            "1",
            "0 / F1",
            ">",
            ":",
            "q",
        ));
        let normal_mode_help_page = Paragraph::new(normal_mode_help_text)
            .block(Block::default().title("普通模式").borders(Borders::ALL))
            .style(*normal_style);

        let commandline_mode_help_text = Text::from(format!(
            "\
            Quit:                                   {}\n\
            Switch Screen:                          {}\n\
            |_                                      {}\n\
            Go To Help Screen (Here):               {}\n\
            Go To Login Screen:                     {}\n\
            Logout:                                 {}\n\
            Set Volume:                             {} (e.g. `vol 20` will set volume at 20%)\n\
            Mute:                                   {}\n\
            Set Play Mode:                          {}\n\
            |_ single play mode:                    {}\n\
            |_ single repeat mode:                  {}\n\
            |_ list repeat mode:                    {}\n\
            |_ shuffle mode:                        {}\n\
            Play Next Song:                         {}\n\
            Start Auto Play:                        {} (Only under `list repeat mode` or `shuffle mode`)\n\
            Jump To Current Song In Playlist:       {}\n\
            Jump To Top:                            {}\n\
            Jump To Bottom:                         {}",
            "q / quit / exit",
            "screen 0 / 1",
            "screen help / main",
            "h / help",
            "l / login",
            "logout",
            "vol / volume",
            "mute",
            "mode",
            "mode single",
            "mode sr / single-repeat",
            "mode lr / list-repeat",
            "mode s / shuf / shuffle",
            "next",
            "start",
            "where this",
            "top",
            "bottom",
        ));
        let commandline_mode_help_page = Paragraph::new(commandline_mode_help_text)
            .block(Block::default().title("命令行模式").borders(Borders::ALL))
            .style(*normal_style);

        Self {
            normal_mode_help_page,
            commandline_mode_help_page,
        }
    }
}

impl<'a> Controller for HelpScreen<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        Ok(false)
    }

    async fn handle_event(&mut self, _cmd: Command) -> Result<bool> {
        Ok(false)
    }

    fn update_view(&mut self, _style: &Style) {}

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunk);

        frame.render_widget(&self.normal_mode_help_page, chunks[0]);

        frame.render_widget(&self.commandline_mode_help_page, chunks[1]);
    }
}
