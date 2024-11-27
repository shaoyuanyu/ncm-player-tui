use crate::ui::Controller;
use crate::NCM_API;
use anyhow::Result;
use fast_qr::QRBuilder;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use crate::config::Command;

pub struct LoginScreen<'a> {
    // model
    login_url: String,           // 登录 url
    login_unikey: String,        // 登录 url 校验码
    login_qrcode: String,        // 登录二维码 (从 login_url 生成)
    login_qrcode_status: String, // 登录二维码状态

    // view
    login_page: Paragraph<'a>,
}

impl<'a> LoginScreen<'a> {
    pub fn new(normal_style: &Style) -> Self {
        let login_qr_url = String::from("");
        let login_unikey = String::from("");
        let login_qrcode = String::from("「...」");
        let login_qrcode_status = String::from("二维码获取中");

        let mut s = Self {
            login_url: login_qr_url,
            login_unikey,
            login_qrcode,
            login_qrcode_status,
            login_page: Paragraph::default(),
        };
        s.update_view(normal_style);
        s
    }

    async fn create_login_qr(&mut self) -> Result<()> {
        let ncm_api_guard = NCM_API.lock().await;
        let (qr_url, qr_unikey) = ncm_api_guard.login_qr_create().await?;
        drop(ncm_api_guard);

        self.login_url = qr_url;
        self.login_unikey = qr_unikey;
        self.login_qrcode = QRBuilder::new(self.login_url.clone()).build()?.to_str();

        Ok(())
    }
}

impl<'a> Controller for LoginScreen<'a> {
    async fn handle_event(&mut self, _cmd: Command) -> Result<()> {
        Ok(())
    }

    async fn update_model(&mut self) -> Result<bool> {
        if self.login_url == "" || self.login_unikey == "" {
            self.create_login_qr().await?;
            return Ok(true);
        }

        let mut ncm_api_guard = NCM_API.lock().await;

        // 检查二维码状态并更新
        let msg = ncm_api_guard
            .login_qr_check(self.login_unikey.clone())
            .await?;

        self.login_qrcode_status = match msg.code {
            800 => String::from("二维码已过期"),
            801 => String::from("等待扫码"),
            802 => String::from("等待确认"),
            803 => String::from("登录成功"),
            _ => String::from(""),
        };

        if msg.code == 800 {
            self.create_login_qr().await?;
        }
        if msg.code == 803 {
            ncm_api_guard.init_after_new_login().await?;
        }

        Ok(true)
    }

    fn update_view(&mut self, style: &Style) {
        let login_text = Text::from(format!(
            "netease cloud music - QR code login\n\
                {}\n\
                {}",
            self.login_qrcode, self.login_qrcode_status
        ));
        self.login_page = Paragraph::new(login_text)
            .block(Block::default().title("Login").borders(Borders::ALL))
            .style(*style);
    }

    fn draw(&self, frame: &mut Frame, chunk: Rect) {
        frame.render_widget(&self.login_page, chunk);
    }
}
