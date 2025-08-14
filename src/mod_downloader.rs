use crate::{
    EMU_NAME,
    curl_helper::BodyExt,
    entities::game::{Game, ModDownloadEntry},
    entities::github::GitTree,
    utils::read_lines,
};
use curl::easy::Easy;
use rayon::prelude::*;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;

struct ModPathInfo {
    title_name: String,
    title_id: String,
    title_version: String,
    relative_path: String,
}

pub struct ModDownloader {
    client: Easy,
    repository: String,
}

const MOD_SUB_DIRS: &[&str] = &["exefs", "romfs", "cheats"];
const MOD_BASE_VERSIONS: &[&str] = &["1.0", "1.0.0"];

impl ModDownloader {
    pub fn new(repository: String) -> Self {
        Self {
            repository,
            client: Easy::new(),
        }
    }

    fn get_git_tree(&mut self) -> Result<GitTree, Box<dyn std::error::Error>> {
        self.client.get(true)?;
        self.client.useragent(env!("CARGO_PKG_NAME")).unwrap();
        self.client.url(&format!(
            "https://api.github.com/repos/{}/git/trees/master?recursive=1",
            self.repository
        ))?;
        Ok(self.client.without_body().send_with_response::<GitTree>()?)
    }

    pub fn read_game_titles(&mut self) -> Result<Vec<Game>, Box<dyn std::error::Error>> {
        let load_directory = self.get_load_directory_path()?;
        let mod_dirs = self.get_mod_directories()?;
        let git_tree = self.get_git_tree()?;

        let games: Vec<Game> = mod_dirs
            .iter()
            .filter_map(|mod_dir_name| {
                let title_id = mod_dir_name.to_string_lossy();
                let title_version = self.get_title_version(&title_id).unwrap();
                let mut title_name = String::new();

                let mod_download_entries: Vec<ModDownloadEntry> = git_tree
                    .tree
                    .iter()
                    .filter(|entry| {
                        entry.type_field == "blob"
                            && MOD_SUB_DIRS
                                .iter()
                                .any(|s| entry.path.contains(&format!("/{}/", s)))
                    })
                    .filter_map(|entry| {
                        let mod_path_info = self.parse_mod_path(&entry.path);

                        if &std::ffi::OsString::from(&mod_path_info.title_id) != mod_dir_name {
                            return None;
                        }

                        if title_name.is_empty() {
                            title_name = mod_path_info.title_name.to_string();
                        }

                        if title_version.as_deref() == Some(&mod_path_info.title_version)
                            || mod_path_info.title_version == "x.x.x"
                            || (title_version.is_none()
                                && MOD_BASE_VERSIONS
                                    .contains(&mod_path_info.title_version.as_str()))
                        {
                            Some(ModDownloadEntry {
                                download_url: format!(
                                    "https://raw.githubusercontent.com/{}/refs/heads/master/{}",
                                    self.repository, entry.path
                                ),
                                mod_relative_path: mod_path_info.relative_path,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                if title_name.is_empty() {
                    return None;
                }

                Some(Game {
                    title_name,
                    title_version,
                    mod_download_entries,
                    mod_data_location: load_directory.join(mod_dir_name),
                    title_id: title_id.to_string(),
                })
            })
            .collect();
        Ok(games)
    }

    pub fn download_mods(
        &self,
        games: &Vec<Game>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let download_jobs: Vec<(&str, PathBuf)> = games
            .iter()
            .flat_map(|game| {
                game.mod_download_entries.iter().map(move |entry| {
                    let downloaded_file_path =
                        game.mod_data_location.join(&entry.mod_relative_path);
                    (entry.download_url.as_ref(), downloaded_file_path)
                })
            })
            .collect();

        download_jobs.par_iter().try_for_each(|(url, path)| {
            create_dir_all(path.parent().unwrap())?;
            let mut file = File::create(path)?;

            let mut easy = Easy::new();
            easy.get(true)?;
            easy.url(&url.replace(" ", "%20"))?;

            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                file.write_all(data)
                    .expect("Failed to write to file during download");
                Ok(data.len())
            })?;
            transfer.perform()?;

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        })?;

        Ok(())
    }

    fn get_load_directory_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let emu = EMU_NAME.get().unwrap();
        let mut config_file_path = dirs::config_dir().unwrap();
        config_file_path.push(emu);
        config_file_path.push("qt-config.ini");

        for line in read_lines(&config_file_path)? {
            let line = line?;
            if let Some(load_directory) = line.strip_prefix("load_directory=") {
                return Ok(if load_directory.is_empty() {
                    let mut path_buf = dirs::data_dir().unwrap();
                    path_buf.push(emu);
                    path_buf.push("nand");
                    path_buf
                } else {
                    load_directory.into()
                });
            }
        }
        Err("Could not find 'load_directory' in config file".into())
    }

    fn get_title_version(&self, title_id: &str) -> std::io::Result<Option<String>> {
        let mut pv_path = dirs::cache_dir().unwrap();
        pv_path.push(EMU_NAME.get().unwrap());
        pv_path.push("game_list");
        pv_path.push(format!("{}.pv.txt", title_id));

        if !pv_path.exists() {
            return Ok(None);
        }

        for line in read_lines(pv_path)? {
            let line = line?;
            if line.starts_with("Update (") {
                let from = line.find("(").map(|i| i + 1).unwrap();
                let to = line.rfind(")").unwrap();
                return Ok(Some(line[from..to].to_string()));
            }
        }
        Ok(None)
    }

    fn get_mod_directories(&self) -> Result<Vec<std::ffi::OsString>, Box<dyn std::error::Error>> {
        let load_directory = self.get_load_directory_path()?;
        Ok(load_directory
            .read_dir()?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|e| e.file_name())
            .collect())
    }

    fn parse_mod_path(&self, path: &str) -> ModPathInfo {
        let parts: Vec<&str> = path.splitn(5, "/").collect();

        ModPathInfo {
            title_name: parts[1].to_string(),
            title_id: parts[2].replace(['[', ']'], ""),
            title_version: parts[3].to_string(),
            relative_path: parts[4..].join("/"),
        }
    }
}
