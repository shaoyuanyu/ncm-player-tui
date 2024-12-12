pub mod model;
mod responses;

use crate::model::{Account, FromJson, LyricLine, Lyrics, Song, Songlist};
use crate::responses::login::*;
use anyhow::{anyhow, Result};
use chrono::Utc;
use log::{debug, error};
use regex::Regex;
use reqwest::{Client, ClientBuilder};
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use tokio::process;

pub struct NcmClient {
    api_program_path: PathBuf,
    cookie_path: PathBuf,
    lyrics_path: PathBuf,
    api_child_process: Option<process::Child>,
    http_client: Client,
    local_api_url: String,
    cookie: String,
    login_account: Option<Account>,
}

impl NcmClient {
    pub fn new(api_program_path: PathBuf, cookie_path: PathBuf, lyrics_path: PathBuf) -> Self {
        Self {
            api_program_path,
            cookie_path,
            lyrics_path,
            api_child_process: None,
            local_api_url: String::from("http://localhost:3000"),
            http_client: ClientBuilder::new()
                .no_proxy()
                .build()
                .expect("failed to build HTTP client"),
            cookie: String::new(),
            login_account: None,
        }
    }

    /// 将 nodejs 编写的 api 程序作为子进程启动，输出重定向到 stderr
    pub async fn check_api(&mut self) -> bool {
        let api_program_path = self.api_program_path.to_str().unwrap();

        let api_child_process: process::Child = process::Command::new("sh")
            .arg("-c")
            .arg(format!("node {}/app.js 1>&2", api_program_path))
            .spawn()
            .expect("Failed to spawn API child process");

        self.api_child_process = Some(api_child_process);

        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            if let Ok(response) = self.http_client.get(&self.local_api_url).send().await {
                if response.status().is_success() {
                    return true;
                }
            }
        }

        false
    }

    /// 退出客户端时，终止 api 子进程
    pub async fn exit_client(&mut self) -> Result<()> {
        match self.api_child_process.as_mut() {
            Some(api_child_process) => {
                api_child_process.kill().await?;
                api_child_process.wait().await?;
            }
            None => {}
        }

        Ok(())
    }
}

/// private
impl NcmClient {
    /// 保存 cookie
    fn store_cookie(&self) {
        match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.cookie_path)
        {
            Ok(mut cookie_file) => match cookie_file.write_all(self.cookie.clone().as_bytes()) {
                Ok(_) => debug!("cookie stored at {:?}", &self.cookie_path),
                Err(err) => error!("failed to store cookie at {:?}: {}", &self.cookie_path, err),
            },
            Err(err) => error!("{:?}", err),
        }
    }

    /// 读 cookie
    fn read_cookie(&mut self) {
        match File::open(&self.cookie_path) {
            Ok(mut cookie_file) => match cookie_file.read_to_string(&mut self.cookie) {
                Ok(_) => debug!("read cookie: {}", &self.cookie),
                Err(err) => error!("failed to read cookie at {:?}: {}", &self.cookie_path, err),
            },
            Err(err) => error!("failed to open cookie at {:?}: {}", &self.cookie_path, err),
        }
    }

    /// 缓存歌词
    fn store_lyrics_cache(&self, song_id: u64, lyrics: &Lyrics) {
        match serde_json::to_string(lyrics) {
            Ok(lyrics_json) => match fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(self.lyrics_path.clone().join(format!("{}.lyrics", song_id)))
            {
                Ok(mut lyrics_file) => match lyrics_file.write_all(lyrics_json.as_bytes()) {
                    Ok(_) => debug!("lyrics stored at {:?}", &self.lyrics_path),
                    Err(err) => {
                        error!("failed to store lyrics at {:?}: {}", &self.lyrics_path, err)
                    }
                },
                Err(err) => error!("{:?}", err),
            },
            Err(err) => error!("{:?}", err),
        }
    }

    /// 尝试读本地歌词缓存
    fn try_read_lyrics_cache(&self, song_id: u64) -> Result<Lyrics> {
        let mut lyrics_file =
            File::open(self.lyrics_path.clone().join(format!("{}.lyrics", song_id)))?;
        let mut json_data = String::new();
        lyrics_file.read_to_string(&mut json_data)?;
        let lyrics: Lyrics = serde_json::from_str(&json_data)?;
        debug!("read lyrics from cache: {:?}", lyrics);

        Ok(lyrics)
    }
}

// 登录 api
impl NcmClient {
    /// 尝试从本地读取 cookie 登录
    pub async fn try_cookie_login(&mut self) -> Result<bool> {
        self.read_cookie();
        if self.cookie.is_empty() {
            return Ok(false);
        }

        self.check_login_status().await?;
        if let Some(_) = self.login_account.as_ref() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 获取登录二维码 (url, uni_key)
    pub async fn get_login_qr(&self) -> Result<(String, String)> {
        let key_response = self
            .http_client
            .get(format!(
                "{}/login/qr/key?timestamp={}",
                &self.local_api_url,
                Utc::now().timestamp()
            ))
            .send()
            .await?
            .json::<QrResponse<QrKeyData>>()
            .await?;

        if key_response.code == 200 && key_response.data.code == 200 {
            let uni_key = key_response.data.unikey;

            let create_response = self
                .http_client
                .get(format!(
                    "{}/login/qr/create?key={}&qrimg=true&timestamp={}",
                    &self.local_api_url,
                    &uni_key,
                    Utc::now().timestamp()
                ))
                .send()
                .await?
                .json::<QrResponse<QrCreateData>>()
                .await?;

            if create_response.code == 200 {
                Ok((uni_key, create_response.data.qrurl))
            } else {
                Err(anyhow!("failed to get login qr url"))
            }
        } else {
            Err(anyhow!("failed to get login qr unikey"))
        }
    }

    /// 检查登录二维码状态
    pub async fn check_login_qr(&mut self, uni_key: &str) -> Result<usize> {
        let check_response = self
            .http_client
            .get(format!(
                "{}/login/qr/check?key={}&timestamp={}",
                &self.local_api_url,
                &uni_key,
                Utc::now().timestamp()
            ))
            .send()
            .await?
            .json::<QrCheckResponse>()
            .await?;

        if check_response.code == 803 {
            // 登录成功，保存 cookie
            self.cookie = check_response.cookie;
            self.store_cookie();
        }

        Ok(check_response.code)
    }

    /// 获取登录状态
    pub async fn check_login_status(&mut self) -> Result<()> {
        let status_response = self
            .http_client
            .post(format!("{}/login/status", &self.local_api_url))
            .form(&[("cookie", &self.cookie)])
            .send()
            .await?
            .bytes()
            .await?;

        let mut v: Value = serde_json::from_slice(&status_response)?;
        let v_profile = v["data"]["profile"].take();
        if !v_profile.is_null() {
            if let Ok(account) = Account::from_json(v_profile) {
                debug!("login, {:?}", account);
                self.login_account = Some(account);
            }
        }

        Ok(())
    }

    /// 是否登录
    pub fn is_login(&self) -> bool {
        if let Some(_) = self.login_account {
            true
        } else {
            false
        }
    }

    /// 登录的账号信息
    pub fn login_account(&self) -> Option<Account> {
        self.login_account.clone()
    }

    /// 登出
    pub async fn logout(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }
}

// 用户 api
impl NcmClient {}

// 歌单 api
impl NcmClient {
    /// 获取用户所有歌单（创建的+收藏的）
    pub async fn get_user_all_songlists(&self) -> Result<Vec<Songlist>> {
        let mut songlists: Vec<Songlist> = Vec::new();

        if let Some(login_account) = self.login_account.as_ref() {
            let user_id = login_account.user_id;

            let playlist_response = self
                .http_client
                .post(format!(
                    "{}/user/playlist?uid={}",
                    &self.local_api_url, user_id
                ))
                .form(&[("cookie", &self.cookie)])
                .send()
                .await?;

            let v_playlist: Value = serde_json::from_slice(&playlist_response.bytes().await?)?;

            // 状态码报错
            if v_playlist["code"].as_u64().unwrap() != 200 {
                return Err(anyhow!(
                    "failed to load songs into songlist, code {}",
                    v_playlist["code"].as_u64().unwrap()
                ));
            }
            // 仍有更多页
            if v_playlist["more"].as_bool().unwrap() {
                // TODO: 增加 offset ，继续获取
            }

            for playlist in v_playlist["playlist"].as_array().unwrap() {
                songlists.push(Songlist {
                    name: playlist["name"].as_str().unwrap().to_string(),
                    id: playlist["id"].as_u64().unwrap(),
                    songs: Vec::new(),
                });
            }

            debug!("songlists: {:?}", songlists);
        }

        Ok(songlists)
    }

    /// 装载歌单内的所有歌曲
    pub async fn load_songlist_songs(&self, songlist: &mut Songlist) -> Result<()> {
        songlist.songs = Vec::new();

        let mut offset = 0;

        while songlist.songs.len() % 1000 == 0 {
            let playlist_detail_response = self
                .http_client
                .post(format!(
                    "{}/playlist/track/all?id={}&limit=1000&offset={}",
                    &self.local_api_url, songlist.id, offset
                ))
                .form(&[("cookie", &self.cookie)])
                .send()
                .await?;

            offset += 1000;

            let v_playlist_detail: Value =
                serde_json::from_slice(&playlist_detail_response.bytes().await?)?;

            // 状态码报错
            if v_playlist_detail["code"].as_u64().unwrap() != 200 {
                return Err(anyhow!(
                    "failed to load songs into songlist, code {}",
                    v_playlist_detail["code"].as_u64().unwrap()
                ));
            }
            // 获取到的歌曲列表为空
            if v_playlist_detail["songs"].as_array().unwrap().is_empty() {
                break;
            }

            // 局部反序列化并装载
            for track in v_playlist_detail["songs"].as_array().unwrap() {
                let song = Song {
                    name: track["name"].as_str().unwrap().to_string(),
                    id: track["id"].as_u64().unwrap(),
                    singer: track["ar"][0]["name"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string(),
                    singer_id: track["ar"][0]["id"].as_u64().unwrap(),
                    album: track["al"]["name"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string(),
                    album_id: track["al"]["id"].as_u64().unwrap(),
                    duration: track["dt"].as_u64().unwrap(),
                    song_url: None,
                };
                songlist.songs.push(song);
            }
        }

        debug!("{:?}", songlist.songs);

        Ok(())
    }
}

// 歌曲 api
impl NcmClient {
    /// 装载歌曲 url
    pub async fn load_song_url(&self, song: &mut Song) -> Result<()> {
        song.song_url = None;

        let song_url_response = self
            .http_client
            .post(format!(
                "{}/song/url/v1?id={}&level={}",
                &self.local_api_url, song.id, "jymaster"
            ))
            .form(&[("cookie", &self.cookie)])
            .send()
            .await?;

        let v_song_url: Value = serde_json::from_slice(&song_url_response.bytes().await?)?;

        if let Some(song_url) = v_song_url["data"][0]["url"].as_str() {
            song.song_url = Some(song_url.to_string());
        }

        Ok(())
    }

    /// 获取歌曲的歌词
    pub async fn get_song_lyrics(&self, song_id: u64) -> Result<Lyrics> {
        // 优先尝试从本地缓存读取歌词
        if let Ok(lyrics) = self.try_read_lyrics_cache(song_id) {
            return Ok(lyrics);
        }

        let lyric_response = self
            .http_client
            .post(format!("{}/lyric?id={}", &self.local_api_url, song_id))
            .form(&[("cookie", &self.cookie)])
            .send()
            .await?;

        let v_lyric: Value = serde_json::from_slice(&lyric_response.bytes().await?)?;

        let lyric_text = v_lyric["lrc"]["lyric"].as_str().unwrap().to_string();
        let trans_lyric_text = v_lyric["tlyric"]["lyric"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let roman_lyric_text = v_lyric["romalrc"]["lyric"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let origin_lyric_lines: Vec<String> = lyric_text
            .split('\n')
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let origin_trans_lyric_lines: Vec<String> = trans_lyric_text
            .split('\n')
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let origin_roman_lyric_lines: Vec<String> = roman_lyric_text
            .split('\n')
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        // 编码歌词
        let lyrics = encode_lyrics(
            origin_lyric_lines,
            origin_trans_lyric_lines,
            origin_roman_lyric_lines,
        );

        debug!("lyrics encoded: {:?}", lyrics);

        // 将歌词缓存到本地
        self.store_lyrics_cache(song_id, &lyrics);

        Ok(lyrics)
    }
}

#[inline]
/// 编码并序列化歌词
fn encode_lyrics(
    origin_lyric_lines: Vec<String>,
    origin_trans_lyric_lines: Vec<String>,
    origin_roman_lyric_lines: Vec<String>,
) -> Lyrics {
    let mut lyrics: Lyrics = Vec::new();

    // 正则表达式
    let timestamp_re = Regex::new(r"\[\d+:\d+.\d+]").unwrap(); // 时间戳
    let timestamp_abnormal_re = Regex::new(r"^\[(\d+):(\d+):(\d+)]").unwrap(); // 不正常时间戳
    let timestamp_10bit_re = Regex::new(r"\[(\d+):(\d+).(\d)(\d)]").unwrap(); // 10位时间戳（小数点后ms部分只有2位）
    let timestamp_7bit_re = Regex::new(r"\[(\d+):(\d+)]").unwrap(); // 7位时间戳（无小数点及ms部分）
    let fix_line = |line: &String| -> String {
        let mut fixed = timestamp_7bit_re
            .replace_all(line, "[$1:$2.000]")
            .to_string();
        fixed = timestamp_10bit_re
            .replace_all(&fixed, "[$1:$2.0$3$4]")
            .to_string();
        fixed = timestamp_abnormal_re
            .replace_all(&fixed, "[$1:$2.$3]")
            .to_string();
        fixed.to_string()
    };

    // 修正
    let fixed_lyric_lines: Vec<String> = origin_lyric_lines.iter().map(fix_line).collect();
    let fixed_trans_lyric_lines: Vec<String> =
        origin_trans_lyric_lines.iter().map(fix_line).collect();
    let fixed_roman_lyric_lines: Vec<String> =
        origin_roman_lyric_lines.iter().map(fix_line).collect();

    // 匹配时间戳并编码
    let mut trans_lyric_line_pointer = (fixed_trans_lyric_lines.len() - 1) as isize;
    let mut roman_lyric_line_pointer = (fixed_roman_lyric_lines.len() - 1) as isize;
    //
    for lyric_line in fixed_lyric_lines.iter().rev() {
        // lyric
        if timestamp_re.is_match(lyric_line) {
            // 计算时间戳
            let timestamp = (lyric_line[1..=2].parse::<u64>().unwrap() * 60
                + lyric_line[4..=5].parse::<u64>().unwrap())
                * 1000
                + lyric_line[7..=9].parse::<u64>().unwrap_or(0) * 10;

            lyrics.push(LyricLine {
                timestamp,
                lyric_line: timestamp_re
                    .replace_all(lyric_line, "")
                    .trim_end_matches('\t')
                    .to_string(),
                trans_lyric_line: None,
                roman_lyric_line: None,
            })
        } else {
            continue;
        }

        // trans_lyric
        while trans_lyric_line_pointer >= 0 {
            if let Some(trans_lyric_line) =
                fixed_trans_lyric_lines.get(trans_lyric_line_pointer as usize)
            {
                if !timestamp_re.is_match(trans_lyric_line) {
                    trans_lyric_line_pointer -= 1;
                    continue;
                }

                if trans_lyric_line.starts_with(&lyric_line[0..=10]) {
                    if let Some(last) = lyrics.last_mut() {
                        last.trans_lyric_line = Some(
                            timestamp_re
                                .replace_all(trans_lyric_line, "")
                                .trim_end_matches('\t')
                                .to_string(),
                        );
                    }

                    trans_lyric_line_pointer -= 1;
                }

                break;
            } else {
                break;
            }
        }

        // roman_lyric
        while roman_lyric_line_pointer >= 0 {
            if let Some(roman_lyric_line) =
                fixed_roman_lyric_lines.get(roman_lyric_line_pointer as usize)
            {
                if !timestamp_re.is_match(roman_lyric_line) {
                    roman_lyric_line_pointer -= 1;
                    continue;
                }

                if roman_lyric_line.starts_with(&lyric_line[0..=10]) {
                    if let Some(last) = lyrics.last_mut() {
                        last.roman_lyric_line = Some(
                            timestamp_re
                                .replace_all(roman_lyric_line, "")
                                .trim_end_matches('\t')
                                .to_string(),
                        );
                    }

                    roman_lyric_line_pointer -= 1;
                }

                break;
            } else {
                break;
            }
        }
    }

    lyrics.reverse();
    lyrics
}
