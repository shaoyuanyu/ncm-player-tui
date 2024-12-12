use crate::config::LOGO_LINES;
use crate::ui::widget::{BottomBar, CommandLine};
use crate::{
    actions,
    config::{AppMode, Command, ScreenEnum},
    ui::{screen::*, Controller},
    NCM_CLIENT, PLAYER,
};
use anyhow::Result;
use crossterm::{
    event,
    event::{Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::style::palette::tailwind;
use ratatui::widgets::Paragraph;
use std::collections::VecDeque;
use std::io::Stdout;

pub struct App<'a> {
    // model
    current_screen: ScreenEnum,
    current_mode: AppMode,
    need_re_update_view: bool,
    command_queue: VecDeque<Command>,

    // view
    main_screen: MainScreen<'a>,
    login_screen: LoginScreen<'a>,
    help_screen: HelpScreen<'a>,
    command_line: CommandLine<'a>,
    bottom_bar: BottomBar<'a>,

    // const
    terminal: Terminal<CrosstermBackend<Stdout>>,
    normal_style: Style,
}

/// public
impl<'a> App<'a> {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        let normal_style = Style::default();

        Self {
            current_screen: ScreenEnum::Launch,
            current_mode: AppMode::Normal,
            need_re_update_view: true,
            command_queue: VecDeque::new(),
            main_screen: MainScreen::new(&normal_style),
            login_screen: LoginScreen::new(&normal_style),
            help_screen: HelpScreen::new(&normal_style),
            command_line: CommandLine::new(),
            bottom_bar: BottomBar::new(&normal_style),
            terminal,
            normal_style,
        }
    }

    /// 绘制启动第一帧（网易云logo）
    pub fn draw_launch_screen(&mut self) -> Result<()> {
        let mut logo_lines = Vec::new();
        for logo_line in LOGO_LINES {
            logo_lines.push(Line::from(logo_line).centered());
        }
        let logo_lines_count = logo_lines.len();

        // 绘制
        self.terminal.draw(|frame| {
            let chunk = frame.area();

            // 竖直居中
            let available_line_count = chunk.height as usize;
            if available_line_count > logo_lines_count {
                for _ in 0..(available_line_count - logo_lines_count) / 2 {
                    logo_lines.insert(0, Line::from(""))
                }
            }

            let logo_paragraph = Paragraph::new(logo_lines)
                .bg(tailwind::RED.c500)
                .fg(tailwind::WHITE);

            frame.render_widget(&logo_paragraph, chunk);
        })?;

        Ok(())
    }

    /// cookie 登录/二维码登录后均调用
    pub async fn init_after_login(&mut self) -> Result<()> {
        // 初始化，获取用户所有歌单（缩略）和 `用户喜欢的音乐` 歌单（详细信息）
        actions::init_songlists().await?;

        // 切换到 main_screen
        self.switch_screen(ScreenEnum::Main).await;

        Ok(())
    }

    /// 尝试 cookie 登录失败后调用
    pub async fn init_after_no_login(&mut self) {
        self.switch_screen(ScreenEnum::Main).await;
    }

    pub fn restore_terminal(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;

        Ok(())
    }
}

/// Controller
impl<'a> App<'a> {
    pub async fn update_model(&mut self) -> Result<()> {
        // screen
        self.need_re_update_view = match self.current_screen {
            ScreenEnum::Help => false,
            ScreenEnum::Login => self.update_login_model().await?,
            ScreenEnum::Main => self.main_screen.update_model().await?,
            _ => false,
        };

        // bottom_bar
        self.bottom_bar.update_model().await?;

        Ok(())
    }

    pub async fn handle_event(&mut self) -> Result<bool> {
        // 解析命令
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press || key_event.kind == KeyEventKind::Repeat {
                match (&self.current_mode, key_event.code) {
                    // Normal 模式
                    (AppMode::Normal, _) => self.get_command_from_key(key_event.code),

                    // Search 模式
                    // 响应 n / N / esc / enter
                    (AppMode::Search(search_keywords), KeyCode::Char('n')) => {
                        self.command_queue
                            .push_back(Command::SearchForward(search_keywords.clone()));
                    }
                    (AppMode::Search(search_keywords), KeyCode::Char('N')) => {
                        self.command_queue
                            .push_back(Command::SearchBackward(search_keywords.clone()));
                    }
                    (AppMode::Search(_), KeyCode::Esc) => {
                        self.back_to_normal_mode();
                    }
                    (AppMode::Search(_), KeyCode::Enter | KeyCode::Char(':')) => {
                        // 返回 normal 模式，同时解析对应的命令，后续执行
                        self.back_to_normal_mode();
                        self.get_command_from_key(key_event.code);
                    }
                    (
                        AppMode::Search(_),
                        KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j'),
                    ) => {
                        // 不返回 normal 模式，同时解析对应的命令，后续执行
                        self.get_command_from_key(key_event.code);
                    }
                    (AppMode::Search(_), _) => {}

                    // CommandLine 模式
                    (AppMode::CommandLine, KeyCode::Enter) => {
                        self.parse_command();
                    }
                    (AppMode::CommandLine, KeyCode::Esc) => {
                        self.back_to_normal_mode();
                    }
                    (AppMode::CommandLine, KeyCode::Backspace) => {
                        if self.command_line.is_content_empty() {
                            self.back_to_normal_mode();
                        } else {
                            self.command_line.input(key_event);
                        }
                    }
                    (AppMode::CommandLine, _) => {
                        self.command_line.input(key_event);
                    }
                }
            }
        }

        // 执行命令
        if let Some(cmd) = self.command_queue.pop_front() {
            // app响应的事件
            match cmd.clone() {
                Command::Quit => {
                    return Ok(false);
                }
                Command::GotoScreen(to_screen) => {
                    self.switch_screen(to_screen).await;
                }
                Command::EnterCommand => {
                    self.switch_to_command_line_mode();
                }
                Command::Logout => {
                    self.login_screen = LoginScreen::new(&self.normal_style);
                    // TODO: 清除 cache
                    NCM_CLIENT.lock().await.logout().await?;
                }
                Command::PlayOrPause => {
                    PLAYER.lock().await.play_or_pause();
                }
                Command::SetVolume(vol) => {
                    PLAYER.lock().await.set_volume(vol);
                }
                Command::SwitchPlayMode(play_mode) => {
                    PLAYER.lock().await.set_play_mode(play_mode);
                }
                Command::StartPlay => {
                    if let Err(e) = PLAYER
                        .lock()
                        .await
                        .start_play(NCM_CLIENT.lock().await)
                        .await
                    {
                        // self.show_prompt(e.to_string().as_str());
                        self.command_line.set_content(e.to_string().as_str());
                    }
                }
                Command::NextSong => {
                    PLAYER
                        .lock()
                        .await
                        .play_next_song_now(NCM_CLIENT.lock().await)
                        .await?;
                }
                Command::PrevSong => {
                    PLAYER
                        .lock()
                        .await
                        .play_prev_song_now(NCM_CLIENT.lock().await)
                        .await?;
                }
                Command::SearchForward(search_keywords) => {
                    self.switch_to_search_mode(search_keywords);
                }
                Command::SearchBackward(search_keywords) => {
                    self.switch_to_search_mode(search_keywords);
                }
                _ => {}
            }

            // 需要向下传递的事件
            match cmd {
                Command::Down
                | Command::Up
                | Command::NextPanel
                | Command::PrevPanel
                | Command::Esc
                | Command::Play
                | Command::WhereIsThisSong
                | Command::GoToTop
                | Command::GoToBottom
                | Command::SearchForward(_)
                | Command::SearchBackward(_) => {
                    // 先 update_model(), 再 handle_event()
                    // 取或值
                    // 若写成 self.need_re_update_view = self.need_re_update_view || match ... {} ，match块内的方法可能不被执行
                    self.need_re_update_view = match self.current_screen {
                        ScreenEnum::Main => self.main_screen.handle_event(cmd).await?,
                        ScreenEnum::Login => self.login_screen.handle_event(cmd).await?,
                        ScreenEnum::Help => self.help_screen.handle_event(cmd).await?,
                        _ => false,
                    } || self.need_re_update_view;
                }
                _ => {}
            }
        }

        Ok(true)
    }

    pub fn update_view(&mut self) {
        // screen 只在 need_re_update_view 为 true 时更新view
        if self.need_re_update_view {
            match self.current_screen {
                ScreenEnum::Help => {}
                ScreenEnum::Login => self.login_screen.update_view(&self.normal_style),
                ScreenEnum::Main => self.main_screen.update_view(&self.normal_style),
                _ => {}
            }
        }

        // bottom_bar
        self.bottom_bar.update_view(&self.normal_style);

        // command_line
        self.command_line.update_view(&self.normal_style);
    }

    pub fn draw(&mut self) -> Result<()> {
        // Launch Screen 需要全屏绘制
        if self.current_screen == ScreenEnum::Launch {
            self.draw_launch_screen()?;
            return Ok(());
        }

        //
        self.update_view();

        //
        self.terminal.draw(|frame| {
            //
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(3),
                        Constraint::Length(3),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(frame.area());

            // render screen
            match self.current_screen {
                ScreenEnum::Help => self.help_screen.draw(frame, chunks[0]),
                ScreenEnum::Login => self.login_screen.draw(frame, chunks[0]),
                ScreenEnum::Main => self.main_screen.draw(frame, chunks[0]),
                _ => {}
            }

            // 渲染 bottom_bar
            self.bottom_bar.draw(frame, chunks[1]);

            // render command_line
            self.command_line.draw(frame, chunks[2]);
        })?;

        Ok(())
    }
}

/// private
impl<'a> App<'a> {
    fn get_command_from_key(&mut self, key_code: KeyCode) {
        let cmd = match key_code {
            KeyCode::Down => Command::Down,
            KeyCode::Char('j') => Command::Down,
            KeyCode::Up => Command::Up,
            KeyCode::Char('k') => Command::Up,
            KeyCode::Char(' ') => Command::PlayOrPause,
            KeyCode::Enter => Command::Play,
            KeyCode::Esc => Command::Esc,
            KeyCode::Right => Command::NextPanel,
            KeyCode::Char('l') => Command::NextPanel,
            KeyCode::Left => Command::PrevPanel,
            KeyCode::Char('h') => Command::PrevPanel,
            KeyCode::Char('1') => Command::GotoScreen(ScreenEnum::Main),
            KeyCode::Char('0') => Command::GotoScreen(ScreenEnum::Help),
            KeyCode::F(1) => Command::GotoScreen(ScreenEnum::Help),
            KeyCode::Char('.') | KeyCode::Char('。') => Command::NextSong,
            KeyCode::Char(',') | KeyCode::Char('，') => Command::PrevSong,
            KeyCode::Char(':') | KeyCode::Char('：') => Command::EnterCommand,
            KeyCode::Char('/') => {
                self.switch_to_search_input_mode();
                self.command_line.set_content("/ ");
                Command::Nop
            }
            KeyCode::Char('?') | KeyCode::Char('？') => {
                self.switch_to_search_input_mode();
                self.command_line.set_content("? ");
                Command::Nop
            }
            //
            KeyCode::Tab => Command::NextPanel,
            KeyCode::BackTab => Command::PrevPanel,
            KeyCode::Char('q') => Command::Quit,
            _ => Command::Nop,
        };

        self.command_queue.push_back(cmd);
    }

    fn parse_command(&mut self) {
        let input_cmd = self.command_line.get_content();

        self.back_to_normal_mode();

        match Command::parse(&input_cmd) {
            Ok(cmd) => {
                self.command_queue.push_back(cmd);
            }
            Err(e) => {
                self.command_line.set_content(format!("{e}").as_str());
            }
        }
    }

    fn back_to_normal_mode(&mut self) {
        self.current_mode = AppMode::Normal;
        self.command_line.set_to_normal_mode();
    }

    fn switch_to_command_line_mode(&mut self) {
        self.current_mode = AppMode::CommandLine;
        self.command_line.set_to_command_line_mode();
    }

    fn switch_to_search_mode(&mut self, search_keywords: Vec<String>) {
        self.current_mode = AppMode::Search(search_keywords);
        self.command_line.set_to_search_mode()
    }

    /// 输入搜索命令时特殊的混合模式
    fn switch_to_search_input_mode(&mut self) {
        self.current_mode = AppMode::CommandLine;
        self.command_line.set_to_search_mode();
    }

    async fn update_login_model(&mut self) -> Result<bool> {
        //
        let need_redraw = self.login_screen.update_model().await?;

        if NCM_CLIENT.lock().await.is_login() {
            // 登录成功
            self.init_after_login().await?;
            Ok(true)
        } else {
            Ok(need_redraw)
        }
    }

    async fn switch_screen(&mut self, to_screen: ScreenEnum) {
        // 已登录状态不能切换到 login_screen
        let ncm_client_guard = NCM_CLIENT.lock().await;
        if to_screen == ScreenEnum::Login && ncm_client_guard.is_login() {
            if let Some(login_account) = ncm_client_guard.login_account() {
                self.command_line.set_content(
                    format!(
                        "正在使用`{}`账号，请先使用`logout`命令登出当前账号",
                        login_account.nickname
                    )
                    .as_str(),
                );
            } else {
                self.command_line
                    .set_content("请先使用`logout`命令登出当前账号");
            }

            return;
        }
        drop(ncm_client_guard);

        // 切换到 main_screen 时显示提示
        if to_screen == ScreenEnum::Main {
            self.command_line.set_content("按0或F1键查看help页面");
        }

        // 切换到 main_screen 时释放当前屏幕（节省内存开销）
        // TODO

        self.need_re_update_view = true;
        self.current_screen = to_screen;
    }
}
