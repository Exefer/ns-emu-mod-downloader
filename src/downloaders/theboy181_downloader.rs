/// TODO: Urgent: Refactor
use super::downloader_ext::TitleVersionExt;
use crate::{
    ARCHIVE_EXTENSIONS, ModDownloadDataset,
    curl_helper::BodyExt,
    entities::{
        game::Game,
        tree::{RepositoryTree, TreeEntry},
    },
};
use curl::easy::Easy;
use std::{
    fs::{File, remove_file},
    io::Write,
    path::PathBuf,
};

pub struct TheBoy181Downloader {
    client: Easy,
}

static THEBOY181_FILE: &'static str = include_str!("../../resources/theboy181.xml");
const REPOSITORY: &'static str = "theboy181/switch-ptchtxt-mods";

// TODO: Urgent: DRY

impl TheBoy181Downloader {
    pub fn new() -> Self {
        Self {
            client: Easy::new(),
        }
    }

    fn get_title_from_mod_url_path(&self, mod_url_path: &str) -> String {
        mod_url_path.split("/").next().unwrap_or_default().into()
    }

    fn get_repo_tree(&mut self) -> Result<RepositoryTree, Box<dyn std::error::Error>> {
        self.client.get(true)?;
        self.client.useragent(env!("CARGO_PKG_NAME")).unwrap();
        self.client.url(&format!(
            "https://api.github.com/repos/{REPOSITORY}/git/trees/main?recursive=1"
        ))?;

        Ok(self
            .client
            .without_body()
            .send_with_response::<RepositoryTree>()?)
    }

    fn get_mod_download_urls(
        &self,
        tree: &Vec<TreeEntry>,
        mod_url_path: &str,
        title_version: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let version_prefixes = [
            format!("{}/{}", mod_url_path, title_version),
            format!("{}/v{}", mod_url_path, title_version),
        ];

        let download_urls: Vec<String> = tree
            .iter()
            .filter(|entry| {
                let path = &entry.path;
                let is_archive = ARCHIVE_EXTENSIONS.iter().any(|ext| path.ends_with(ext));
                let prefix_matches = version_prefixes
                    .iter()
                    .any(|prefix| path.starts_with(prefix));
                is_archive && prefix_matches
            })
            .map(|entry| {
                format!(
                    "https://raw.githubusercontent.com/{REPOSITORY}/refs/heads/main/{}",
                    entry.path
                )
            })
            .collect();

        Ok(download_urls)
    }
}

impl super::mod_downloader::ModDownloader for TheBoy181Downloader {
    fn download_mods(&mut self, games: &Vec<Game>) -> Result<(), Box<dyn std::error::Error>> {
        for game in games {
            for url in &game.mod_download_urls {
                let (.., file_name) = url.rsplit_once("/").unwrap();
                let downloaded_file_path = PathBuf::from(&game.mod_data_location).join(file_name);
                let mut file = File::create(&downloaded_file_path)?;

                self.client.get(true)?;
                self.client.url(&url.replace(" ", "%20"))?;
                let mut transfer = self.client.transfer();

                transfer.write_function(|data| {
                    file.write_all(data).expect("Failed to write to file");
                    Ok(data.len())
                })?;

                transfer.perform()?;

                crate::archive::extract_archive(
                    &downloaded_file_path,
                    &downloaded_file_path.parent().unwrap().to_path_buf(),
                )?;

                remove_file(downloaded_file_path).ok();
            }
        }
        Ok(())
    }

    fn read_game_titles(&mut self) -> Result<Vec<Game>, Box<dyn std::error::Error>> {
        let dataset: ModDownloadDataset = serde_xml_rs::from_str(THEBOY181_FILE)?;
        let mod_directory_path = self.get_load_directory_path()?;
        let repo_tree = self.get_repo_tree()?;

        let games: Vec<Game> = dataset
            .iter()
            .filter_map(|entry| {
                let title_version = self.get_title_version(&entry.title_id).ok()?;
                let mod_data_location = mod_directory_path.join(&entry.title_id);

                if !mod_data_location.exists() {
                    return None;
                }

                let title_name = self.get_title_from_mod_url_path(&entry.mod_url_path);
                let mod_download_urls = self
                    .get_mod_download_urls(&repo_tree.tree, &entry.mod_url_path, &title_version)
                    .ok()?;

                Some(Game {
                    title_id: entry.title_id.clone(),
                    title_version: title_version.to_owned(),
                    title_name,
                    mod_data_location,
                    mod_download_urls,
                })
            })
            .collect();

        Ok(games)
    }
}
