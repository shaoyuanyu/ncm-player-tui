use crate::config::Command;
use crate::ncm_client;
use crate::ui::Controller;
use anyhow::Result;
use fast_qr::QRBuilder;
use log::debug;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub struct LoginScreen<'a> {
    // model
    login_url: String,             // 登录 url
    login_unikey: String,          // 登录 url 校验码
    login_qr_lines: Vec<Line<'a>>, // 登录二维码（按行）
    login_qr_status_code: usize,   // 登录二维码状态码
    login_qr_status: String,       // 登录二维码状态
    is_login_ok_refreshed: bool, // 标志控制位，控制登录完成后第一次 update_model 更新“登录成功”的信息，第二次 update_model 才进行 cookie 保存等高延迟操作
    tick_tok: usize, // 时钟记录，0~3，用于在状态显示后增加动态省略号，也用于控制发送检查二维码请求的频率

    // view
    login_page: Paragraph<'a>,
}

impl<'a> LoginScreen<'a> {
    pub fn new(normal_style: &Style) -> Self {
        let login_qr_url = String::from("");
        let login_unikey = String::from("");
        let login_qrcode_status = String::from("二维码获取中");

        let mut s = Self {
            login_url: login_qr_url,
            login_unikey,
            login_qr_lines: vec![Line::from("「...」").centered()],
            login_qr_status_code: 0,
            login_qr_status: login_qrcode_status,
            login_page: Paragraph::default(),
            is_login_ok_refreshed: false,
            tick_tok: 0,
        };
        s.update_view(normal_style);
        s
    }

    async fn create_login_qr(&mut self) -> Result<()> {
        let (qr_unikey, qr_url) = ncm_client.lock().await.get_login_qr().await?;

        self.login_unikey = qr_unikey;
        self.login_url = qr_url;
        let qrcode = QRBuilder::new(self.login_url.clone()).build()?.to_str();
        // self.login_qrcode = qrcode.clone();
        self.login_qr_lines = qrcode
            .split('\n')
            .into_iter()
            .map(|s| Line::from(s.to_owned()).centered())
            .collect();
        self.is_login_ok_refreshed = false;

        Ok(())
    }
}

impl<'a> Controller for LoginScreen<'a> {
    async fn update_model(&mut self) -> Result<bool> {
        // 时钟记录
        self.tick_tok += 1;
        if self.tick_tok == 4 {
            self.tick_tok = 0;
        }

        // 初始化
        if self.login_url == "" || self.login_unikey == "" {
            self.create_login_qr().await?;
            return Ok(true);
        }

        // 只在 tick_tok 为1时更新状态，减小 check_login_qr 请求频率
        if self.tick_tok == 1 {
            let mut ncm_client_guard = ncm_client.lock().await;

            // 检查二维码状态并更新
            self.login_qr_status_code = ncm_client_guard
                .check_login_qr(self.login_unikey.as_str())
                .await?;

            self.login_qr_status = match self.login_qr_status_code {
                800 => String::from("二维码已过期，请稍等"),
                801 => String::from("等待扫码"),
                802 => String::from("等待确认"),
                803 => String::from("登录成功，请稍等"),
                _ => String::from(""),
            };

            // 二维码过期，重新生成
            if self.login_qr_status_code == 800 {
                self.create_login_qr().await?;
                return Ok(true);
            }
        }

        // 等待扫码/确认时降低刷新率和访问频率
        if self.login_qr_status_code == 801 || self.login_qr_status_code == 802 {
            tokio::time::sleep(std::time::Duration::from_millis(350)).await;
        }

        // 登录成功
        if self.login_qr_status_code == 803 {
            if !self.is_login_ok_refreshed {
                // 登录成功后第一次 update_model
                // 不阻塞，以便 update_view 能够及时显示“登录成功”
                self.is_login_ok_refreshed = true;
            } else {
                // 登录成功后第二次 update_model
                debug!("login successfully, start cookie storing...");
                let mut ncm_client_guard = ncm_client.lock().await;
                ncm_client_guard.store_cookie();
                ncm_client_guard.check_login_status().await?;
            }
        }

        Ok(true)
    }

    async fn handle_event(&mut self, _cmd: Command) -> Result<bool> {
        Ok(false)
    }

    fn update_view(&mut self, style: &Style) {
        let login_text = Text::from(self.login_qr_lines.clone());

        self.login_page = Paragraph::new(login_text)
            .block(
                Block::default()
                    .title(Line::from("Netease Cloud Music - QR Code Login").left_aligned())
                    .title(
                        Line::from(format!(
                            "{}{}",
                            self.login_qr_status.clone(),
                            ".".repeat(self.tick_tok)
                        ))
                        .right_aligned(),
                    )
                    .title_bottom(
                        Line::from("如果无法识别二维码，可将终端背景色改为深色后再尝试")
                            .right_aligned(),
                    )
                    .borders(Borders::ALL),
            )
            .style(*style);
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        frame.render_widget(&self.login_page, chunk);
    }
}
