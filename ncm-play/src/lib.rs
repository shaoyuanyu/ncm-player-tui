pub mod config;
pub use config::*;

use anyhow::{anyhow, Result};
use gstreamer::ClockTime;
use gstreamer_play::{gst, Play, PlayVideoRenderer};
use log::{debug, trace};
use ncm_api::model::Songlist;
use ncm_api::{
    model::{Lyrics, Song},
    NcmClient,
};
use rand::{thread_rng, Rng};
use tokio::sync::MutexGuard;

pub struct Player {
    play: Play,
    //
    play_state: PlayState,
    play_mode: PlayMode,
    //
    volume: f64,
    //
    playlist_candidates: Vec<Songlist>,
    //
    current_playlist_name: String,
    current_playlist: Vec<Song>, // TODO: 优化为指针
    //
    play_index_history_stack: Vec<usize>, // 历史记录，保存播放的歌曲在 playlist 中的 index，栈顶为当前播放
    //
    current_song_index: Option<usize>,
    current_song: Option<Song>,
    //
    current_song_lyrics: Option<Lyrics>,
    current_lyric_line_index: Option<usize>,
}

impl Player {
    pub fn new() -> Self {
        gst::init().expect("Failed to initialize GST");

        let play = Play::new(None::<PlayVideoRenderer>);

        let mut config = play.config();
        config.set_user_agent(
            "User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:100.0) Gecko/20100101 Firefox/100.0",
        );
        config.set_position_update_interval(250);
        config.set_seek_accurate(true);
        play.set_config(config).unwrap();

        let volume = 0.2;
        play.set_volume(volume);

        Self {
            play,
            play_state: PlayState::Stopped,
            play_mode: PlayMode::Shuffle,
            volume,
            playlist_candidates: Vec::new(),
            current_playlist_name: String::new(),
            current_playlist: Vec::new(),
            play_index_history_stack: Vec::new(),
            current_song_index: None,
            current_song: None,
            current_song_lyrics: None,
            current_lyric_line_index: None,
        }
    }
}

/// setter & getter
impl Player {
    pub fn set_volume(&mut self, mut volume: f64) {
        if volume > 1.0 {
            volume = 1.0;
        } else if volume < 0.0 {
            volume = 0.0;
        }
        self.volume = volume;
        self.play.set_volume(volume);
    }

    pub fn mute(&mut self) {
        self.volume = 0.0;
        self.play.set_volume(0.0);
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }

    pub fn is_playing(&self) -> bool {
        self.play_state == PlayState::Playing
    }

    pub fn play_mode(&self) -> String {
        self.play_mode.to_string()
    }

    pub fn set_play_mode(&mut self, mode: PlayMode) {
        self.play_mode = mode;
    }

    pub fn duration(&self) -> Option<ClockTime> {
        self.play.duration()
    }

    pub fn position(&self) -> Option<ClockTime> {
        self.play.position()
    }

    pub fn current_playlist_name(&self) -> &String {
        &self.current_playlist_name
    }

    pub fn current_playlist(&self) -> &Vec<Song> {
        &self.current_playlist
    }

    pub fn current_song(&self) -> &Option<Song> {
        &self.current_song
    }

    pub fn current_song_index(&self) -> Option<usize> {
        self.current_song_index.clone()
    }

    pub fn current_song_lyrics(&self) -> Option<Lyrics> {
        self.current_song_lyrics.clone()
    }

    pub fn current_lyric_line_index(&self) -> Option<usize> {
        self.current_lyric_line_index
    }

    pub fn set_playlist_candidates(&mut self, candidates: Vec<Songlist>) {
        self.playlist_candidates = candidates;
    }

    pub fn playlist_candidates(&self) -> &Vec<Songlist> {
        &self.playlist_candidates
    }
}

/// playlist
impl Player {
    /// 切换播放列表
    pub async fn switch_playlist<'c>(
        &mut self,
        playlist_candidate_index: usize,
        ncm_client_guard: MutexGuard<'c, NcmClient>,
    ) -> Result<()> {
        if let Some(songlist) = self.playlist_candidates.get_mut(playlist_candidate_index) {
            debug!("{:?}", songlist);

            //
            ncm_client_guard.load_songlist_songs(songlist).await?;

            //
            self.current_playlist_name = songlist.name.clone();
            self.current_playlist = songlist.songs.clone();
            self.play_index_history_stack = Vec::new();
            self.current_song_index = if self.current_playlist.is_empty() {
                None
            } else {
                Some(0)
            };

            Ok(())
        } else {
            Err(anyhow!("no playlist candidates found"))
        }
    }

    /// 向后搜索歌单（向上方搜索）
    pub fn search_backward_playlist(
        &mut self,
        start_index: usize,
        keywords: Vec<String>,
    ) -> Option<usize> {
        if start_index < self.current_playlist.len() {
            let playlist_backward_iter = self.current_playlist[0..start_index]
                .iter()
                .enumerate()
                .rev();

            if let Some(offset) = search_in_iter(playlist_backward_iter, keywords) {
                return Some(offset);
            }
        }

        None
    }

    /// 向前搜索歌单（向下方搜索）
    pub fn search_forward_playlist(
        &mut self,
        start_index: usize,
        keywords: Vec<String>,
    ) -> Option<usize> {
        if start_index + 1 < self.current_playlist.len() {
            let playlist_backward_iter =
                self.current_playlist[start_index + 1..].iter().enumerate();

            if let Some(offset) = search_in_iter(playlist_backward_iter, keywords) {
                return Some(start_index + 1 + offset);
            }
        }

        None
    }
}

#[inline]
fn search_in_iter<'c, I>(mut playlist_iter: I, keywords: Vec<String>) -> Option<usize>
where
    I: Iterator<Item = (usize, &'c Song)>,
{
    while let Some((index, song)) = playlist_iter.next() {
        let mut key_matched = true;
        for keyword in keywords.iter() {
            if !song
                .name
                .clone()
                .to_ascii_lowercase()
                .contains(keyword.to_ascii_lowercase().as_str())
            {
                key_matched = false;
                break;
            }
        }
        if key_matched {
            return Some(index);
        }
    }

    None
}

/// 播放相关
impl Player {
    /// 切换播放/暂停
    pub fn play_or_pause(&mut self) {
        if self.play_state == PlayState::Playing {
            self.play.pause();
            self.play_state = PlayState::Paused;
        } else if self.play_state == PlayState::Paused {
            self.play.play();
            self.play_state = PlayState::Playing;
        }
    }

    /// 自动播放
    pub async fn auto_play<'c>(
        &mut self,
        ncm_client_guard: MutexGuard<'c, NcmClient>,
    ) -> Result<()> {
        // 判断一首歌是否播放完
        if self.play_state == PlayState::Playing {
            if let (Some(position), Some(duration)) = (self.position(), self.duration()) {
                let position_msec = position.mseconds();
                let duration_msec = duration.mseconds();

                if duration_msec - position_msec <= 10 {
                    self.play_state = PlayState::Ended;
                }
            }
        }

        if self.play_state == PlayState::Playing {
            // 当前歌曲仍在播放，推进歌词
            self.auto_lyric_forward();
        } else if self.play_state == PlayState::Ended {
            // 播放下一首
            self.update_next_to_play();
            self.play_next(ncm_client_guard).await?;
        }

        Ok(())
    }

    /// 立刻播放指定歌曲
    pub async fn play_particularly_now<'c>(
        &mut self,
        index_to_play: usize,
        ncm_client_guard: MutexGuard<'c, NcmClient>,
    ) -> Result<()> {
        if index_to_play < self.current_playlist.len() {
            self.play_state = PlayState::Playing;
            self.current_song_index = Some(index_to_play);
            self.current_song = Some(self.current_playlist[index_to_play].clone());

            self.play_next(ncm_client_guard).await?;
        }

        Ok(())
    }

    /// 根据当前模式开始播放
    pub async fn start_play<'c>(
        &mut self,
        ncm_client_guard: MutexGuard<'c, NcmClient>,
    ) -> Result<()> {
        if !self.current_playlist.is_empty() {
            match self.play_mode {
                PlayMode::ListRepeat => {
                    self.current_song_index = Some(0);
                    self.current_song = Some(self.current_playlist[0].clone());
                    self.play_next(ncm_client_guard).await?;
                    Ok(())
                }
                PlayMode::Shuffle => {
                    let index = thread_rng().gen_range(0..self.current_playlist.len());
                    self.current_song_index = Some(index);
                    self.current_song = Some(self.current_playlist[index].clone());
                    self.play_next(ncm_client_guard).await?;
                    Ok(())
                }
                _ => Err(anyhow!("start命令只在`列表循环`和`随机播放`模式下有效")),
            }
        } else {
            Err(anyhow!("请先选择歌单"))
        }
    }

    /// 立刻播放下一首
    pub async fn play_next_song_now<'c>(
        &mut self,
        ncm_client_guard: MutexGuard<'c, NcmClient>,
    ) -> Result<()> {
        if self.play_state == PlayState::Playing
            || self.play_state == PlayState::Paused
            || self.play_state == PlayState::Ended
        {
            // 当前单曲播放半秒后才可以切换到下一首，留出缓冲时间，防止切换过快
            if let Some(position) = self.position() {
                if position.mseconds() >= 500 {
                    self.update_next_to_play();

                    debug!("[{:?}] {:?}, ", self.current_song_index, self.current_song);

                    self.play_next(ncm_client_guard).await?;
                }
            }
        }

        Ok(())
    }

    /// 立刻播放上一首
    pub async fn play_prev_song_now<'c>(
        &mut self,
        ncm_client_guard: MutexGuard<'c, NcmClient>,
    ) -> Result<()> {
        // 当前单曲播放半秒后才可以切换到上一首，留出缓冲时间，防止切换过快
        if let Some(position) = self.position() {
            if position.mseconds() >= 500 {
                // 出栈历史记录
                if let Some(current_song_index) = self.play_index_history_stack.pop() {
                    if let Some(prev_song_index) = self.play_index_history_stack.pop() {
                        // 播放上一首
                        self.current_song_index = Some(prev_song_index);
                        self.current_song = Some(self.current_playlist[prev_song_index].clone());
                        self.play_next(ncm_client_guard).await?;
                    } else {
                        // 无上一首（当前为第一首播放）
                        self.play_index_history_stack.push(current_song_index);
                    }
                }
            }
        }

        Ok(())
    }

    /// 跳转到所给编号的时间戳处播放
    pub async fn seek_to_timestamp_with_index(&mut self, index: usize) -> Result<()> {
        if self.play_state == PlayState::Playing
            || self.play_state == PlayState::Paused
            || self.play_state == PlayState::Ended
        {
            if let Some(current_song_lyrics) = self.current_song_lyrics.as_ref() {
                if index < current_song_lyrics.len() {
                    self.current_lyric_line_index = Some(index);
                    let timestamp = current_song_lyrics[index].timestamp;
                    self.play.seek(ClockTime::from_mseconds(timestamp));
                }
            }
        }

        Ok(())
    }
}

/// private
impl Player {
    /// 根据模式更新下一首播放的歌曲
    /// 更新 self.current_song & self.current_song_index
    fn update_next_to_play(&mut self) {
        self.current_song = match self.play_mode {
            PlayMode::Single => None,
            PlayMode::SingleRepeat => self.current_song.clone(),
            PlayMode::ListRepeat => {
                if let Some(mut index) = self.current_song_index {
                    index += 1;
                    if index >= self.current_playlist.len() {
                        index = 0;
                    }
                    self.current_song_index = Some(index);
                    Some(self.current_playlist[index].clone())
                } else {
                    None
                }
            }
            PlayMode::Shuffle => {
                if let Some(mut index) = self.current_song_index {
                    index = thread_rng().gen_range(0..self.current_playlist.len());
                    self.current_song_index = Some(index);
                    Some(self.current_playlist[index].clone())
                } else {
                    None
                }
            }
        };
    }

    /// 播放下一首
    async fn play_next<'c>(&mut self, ncm_client_guard: MutexGuard<'c, NcmClient>) -> Result<()> {
        if let Some(mut song) = self.current_song.clone() {
            // 检查歌曲是否可获取（版权/会员/...限制）
            if ncm_client_guard.check_song_availability(song.id).await? {
                // 获取歌曲 uri
                ncm_client_guard.load_song_url(&mut song).await?;

                // 更新当前歌曲信息
                self.current_song = Some(song.clone());

                if let Some(url) = song.song_url {
                    // 入栈播放历史
                    if let Some(index) = self.current_song_index {
                        self.play_index_history_stack.push(index);
                    }

                    // 获取歌词
                    self.update_current_song_lyrics(ncm_client_guard).await?;

                    // 播放
                    self.play_new_song_by_uri(url.as_str()).await;

                    // 播放状态
                    self.play_state = PlayState::Playing;

                    debug!("play next song: {:?}", self.current_song);
                }
            } else {
                // 更新播放状态为 Ended ，以便继续寻找下一首
                self.play_state = PlayState::Ended;
            }
        } else {
            // 播放状态
            self.play_state = PlayState::Stopped;
        }

        Ok(())
    }

    async fn play_new_song_by_uri(&mut self, uri: &str) {
        self.play.stop();
        self.play.set_uri(Some(uri));
        self.play.set_volume(self.volume);
        self.play.play();

        // 缓冲 500 ms ，防止出现切换到下一首歌但 gstreamer 端还未更新完成，这会引起歌词快进现象
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    async fn update_current_song_lyrics<'c>(
        &mut self,
        ncm_client_guard: MutexGuard<'c, NcmClient>,
    ) -> Result<()> {
        if let Some(current_song) = self.current_song.as_ref() {
            if let Ok(lyrics) = ncm_client_guard.get_song_lyrics(current_song.id).await {
                if !lyrics.is_empty() {
                    self.current_song_lyrics = Some(lyrics);
                    self.current_lyric_line_index = Some(0);

                    return Ok(());
                }
            }
        }

        // 无歌词（纯音乐或网络异常）
        self.current_song_lyrics = None;
        self.current_lyric_line_index = None;

        Ok(())
    }

    fn auto_lyric_forward(&mut self) {
        if let (Some(current_lyric_line_index), Some(current_song_lyrics)) = (
            self.current_lyric_line_index,
            self.current_song_lyrics.as_ref(),
        ) {
            if let Some(current_position) = self.position() {
                if current_lyric_line_index + 1 < current_song_lyrics.len() {
                    let next_timestamp =
                        current_song_lyrics[current_lyric_line_index + 1].timestamp;

                    if current_position.mseconds() >= next_timestamp {
                        trace!(
                            "[auto lyric forward] current msec: {}, next timestamp: {}",
                            current_position.mseconds(),
                            next_timestamp
                        );

                        self.current_lyric_line_index = Some(current_lyric_line_index + 1);
                    }
                }
            }
        }
    }
}
