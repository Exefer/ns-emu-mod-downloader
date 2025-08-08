use crate::{EMU_NAME, entities::game::Game, utils::read_lines};
use std::path::PathBuf;

pub trait ModDownloader {
    fn download_mods(&mut self, games: &Vec<Game>) -> Result<(), Box<dyn std::error::Error>>;
    fn read_game_titles(&mut self) -> Result<Vec<Game>, Box<dyn std::error::Error>>;

    fn get_load_directory_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let emu = EMU_NAME.get().unwrap();
        let mut config_file_path = dirs::config_dir().unwrap();
        config_file_path.push(emu);
        config_file_path.push("qt-config.ini");

        for line in read_lines(&config_file_path)? {
            let line = line?;
            if line.starts_with("load_directory=") {
                let load_directory = line.replace("load_directory=", "");
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
}
