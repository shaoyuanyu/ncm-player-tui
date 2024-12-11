pub mod account;
pub mod lyric;
pub mod song;
pub mod songlist;

pub use account::*;
pub use lyric::*;
pub use song::*;
pub use songlist::*;

pub trait FromJson {
    type SelfType;

    fn from_json(value: serde_json::Value) -> anyhow::Result<Self::SelfType>;
}
