use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "ncm-tui";

pub struct Path {
    // 一级目录
    pub data: PathBuf,
    pub config: PathBuf,
    pub cache: PathBuf,

    // 二级目录
    pub login_cookie: PathBuf,
    pub lyrics: PathBuf,
}

impl Path {
    pub fn new() -> Self {
        let data = dirs_next::data_dir().unwrap().join(APP_NAME);
        if !data.exists() {
            fs::create_dir(&data).expect("Couldn't create data dir.");
        }

        let config = dirs_next::config_dir().unwrap().join(APP_NAME);
        if !config.exists() {
            fs::create_dir(&config).expect("Couldn't create config dir.");
        }

        let cache = dirs_next::cache_dir().unwrap().join(APP_NAME);
        if !cache.exists() {
            fs::create_dir(&cache).expect("Couldn't create cache dir.");
        }

        let login_cookie = data.clone().join("cookies.json");

        let lyrics = data.clone().join("lyrics");
        if !lyrics.exists() {
            fs::create_dir(&lyrics).expect("Couldn't create lyrics dir.");
        }

        Self {
            data,
            config,
            cache,
            login_cookie,
            lyrics,
        }
    }
}
