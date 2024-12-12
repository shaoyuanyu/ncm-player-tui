use std::fmt;

#[derive(Clone, PartialEq)]
pub enum PlayState {
    /// 未进入播放
    Stopped,

    /// 暂停
    Paused,

    /// 播放中
    Playing,

    /// 一首歌播放结束
    Ended,
}

#[derive(Clone)]
pub enum PlayMode {
    Single,
    SingleRepeat,
    ListRepeat,
    Shuffle,
}

impl fmt::Display for PlayMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayMode::Single => write!(f, "单曲播放"),
            PlayMode::SingleRepeat => write!(f, "单曲循环"),
            PlayMode::ListRepeat => write!(f, "列表循环"),
            PlayMode::Shuffle => write!(f, "随机播放"),
        }
    }
}
