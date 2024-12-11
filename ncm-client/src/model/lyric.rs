use serde::{Deserialize, Serialize};

pub type Lyrics = Vec<LyricLine>;

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct LyricLine {
    /// 时间戳（ms）
    pub timestamp: u64,
    pub lyric_line: String,
    pub trans_lyric_line: Option<String>,
    pub roman_lyric_line: Option<String>,
}
