use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Settings {
    pub use_remote_api: bool,
    pub remote_api_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            use_remote_api: false,
            remote_api_url: String::from("https://ncm-api-wine.vercel.app/"),
        }
    }
}
