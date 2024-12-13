use serde::{Deserialize, Serialize};

pub type Lyrics = Vec<LyricLine>;

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct LyricLine {
    /// 时间戳（ms）
    #[serde(rename = "s")]
    pub timestamp: u64,

    /// 歌词行
    #[serde(rename = "l")]
    pub lyric_line: String,

    /// 翻译歌词行
    #[serde(rename = "t")]
    pub trans_lyric_line: Option<String>,

    /// 罗马音歌词行
    #[serde(rename = "r")]
    pub roman_lyric_line: Option<String>,
}
