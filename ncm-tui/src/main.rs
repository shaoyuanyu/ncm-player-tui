mod actions;
mod config;
mod ui;

use crate::config::{Command, Path};
use crate::ui::App;
use anyhow::Result;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use crossterm::{event, execute};
use lazy_static::lazy_static;
use ncm_api::NcmClient;
use ncm_play::Player;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::VecDeque;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::sleep;

const POLL_DURATION: Duration = Duration::from_millis(100);

lazy_static! {
    static ref path_config: Path = Path::new();
    static ref ncm_client: Arc<Mutex<NcmClient>> = Arc::new(Mutex::new(NcmClient::new(
        path_config.api_program.clone(),
        path_config.login_cookie.clone(),
        path_config.lyrics.clone(),
    )));
    static ref player: Arc<Mutex<Player>> = Arc::new(Mutex::new(Player::new()));
    static ref command_queue: Arc<Mutex<VecDeque<Command>>> = Arc::new(Mutex::new(VecDeque::new()));
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let app = Arc::new(Mutex::new(App::new(create_terminal()?)));

    // 绘制第一帧（launch screen）
    app.lock().await.draw_launch_screen()?;

    // 创建 NCM_API 时会默认尝试 cookie 登录，在新线程中检查 cookie 状态并初始化
    let app_2 = Arc::clone(&app);
    let ncm_client_2 = Arc::clone(&ncm_client);
    task::spawn(async move {
        sleep(Duration::from_secs(1)).await; // 给启动帧留缓冲

        if ncm_client_2.lock().await.check_api().await {
            if ncm_client_2.lock().await.try_cookie_login().await.unwrap_or(false) {
                app_2.lock().await.init_after_login().await.expect("Couldn't initialize application");
            } else {
                app_2.lock().await.init_after_no_login().await;
            }
        }
    });

    loop {
        // 检查播放情况
        player.lock().await.auto_play(ncm_client.lock().await).await?;

        // 根据 Controller 流程，先执行 update_model()，再执行 handle_event()
        app.lock().await.update_model().await?;

        if event::poll(POLL_DURATION)? {
            app.lock().await.parse_key_to_event().await?;
        }

        if !app.lock().await.handle_event().await? {
            ncm_client.lock().await.exit_client().await?;
            return app.lock().await.restore_terminal();
        }

        // 渲染
        app.lock().await.draw()?;
    }
}

fn create_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}
