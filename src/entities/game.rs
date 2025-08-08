use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Game {
    pub title_id: String,
    pub title_name: String,
    pub title_version: String,
    pub mod_data_location: PathBuf,
    pub mod_download_urls: Vec<String>,
}
