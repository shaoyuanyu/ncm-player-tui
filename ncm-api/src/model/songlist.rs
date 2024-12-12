use crate::model::song::Song;
use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct Songlist {
    /// 歌单名
    pub name: String,
    /// 歌单 id
    pub id: u64,
    /// 歌单内的歌曲
    pub songs: Vec<Song>,
}
