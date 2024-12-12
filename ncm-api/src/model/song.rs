use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct Song {
    /// 歌名
    pub name: String,
    /// 歌曲 id
    pub id: u64,
    /// 歌手
    pub singer: String,
    /// 歌手 id
    pub singer_id: u64,
    /// 专辑
    pub album: String,
    /// 专辑 id
    pub album_id: u64,
    /// 歌曲时长
    pub duration: u64,
    /// 歌曲链接
    pub song_url: Option<String>,
}
