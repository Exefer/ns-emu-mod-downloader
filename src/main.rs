mod archive;
mod curl_helper;
mod downloaders;
mod entities;
mod utils;

use crate::downloaders::{
    mod_downloader::ModDownloader, theboy181_downloader::TheBoy181Downloader,
    yuzu_mod_archive_downloader::YuzuModArchiveDownloader,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Write},
    ops::Deref,
    sync::OnceLock,
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GameTitle {
    title_name: String,
    title_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GameDataset(Vec<GameTitle>);

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ModDownload {
    title_id: String,
    mod_url_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ModDownloadDataset(Vec<ModDownload>);

impl Deref for ModDownloadDataset {
    type Target = Vec<ModDownload>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) const ARCHIVE_EXTENSIONS: &[&str] = &[".zip", ".rar", ".7z"];

pub(crate) static EMU_NAME: OnceLock<String> = OnceLock::new();

fn get_input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn display_options<T: std::fmt::Display>(title: &str, items: &[T]) {
    println!("\n{}:", title);
    for (i, item) in items.iter().enumerate() {
        println!("  {}) {}", i + 1, item);
    }
}

fn get_emu() -> Result<String, Box<dyn std::error::Error>> {
    let emus: &[&[u8]] = &[
        &[121, 117, 122, 117],
        &[115, 117, 121, 117],
        &[101, 100, 101, 110],
        &[99, 105, 116, 114, 111, 110],
        &[116, 111, 114, 122, 117],
        &[115, 117, 100, 97, 99, 104, 105],
    ];
    let emus: Vec<String> = emus
        .iter()
        .map(|slice| String::from_utf8(slice.to_vec()).unwrap())
        .collect();

    display_options("Select an emulator to download mods for", &emus);
    let input = get_input(&format!("\nEnter your choice [1-{}]: ", emus.len()))?;

    let choice = input.parse::<usize>().unwrap_or(0);

    if choice == 0 || choice > emus.len() {
        return Err(format!(
            "Invalid option '{}'. Please choose a value from 1 to {}.",
            input,
            emus.len()
        )
        .into());
    }

    let emu = &emus[choice - 1][..];
    let emu_data_dir = dirs::data_dir().unwrap().join(emu);
    let emu_config_dir = dirs::config_dir().unwrap().join(emu);

    if !emu_data_dir.exists() || !emu_config_dir.exists() {
        println!(
            "\nPlease install {} first or verify it's properly configured.\nExpected directories:\n  Data: {}\n  Config: {}\n",
            emu,
            emu_data_dir.display(),
            emu_config_dir.display()
        );

        return Err(format!("Emulator '{}' is not installed on the system.", emu,).into());
    }

    Ok(emu.into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Mod Downloader ===");

    EMU_NAME.set(get_emu()?.to_string())?;

    display_options(
        "Select a source to download mods from",
        &["TheBoy181", "Yuzu Mod Archive"],
    );
    let input = get_input("\nEnter your choice [1-2]: ")?;

    let mut downloader: Box<dyn ModDownloader> = match input.as_str() {
        "1" => Box::new(TheBoy181Downloader::new()),
        "2" => Box::new(YuzuModArchiveDownloader::new()),
        _ => {
            return Err(format!(
                "\nInvalid option '{}'. Please choose a value from 1 to 2.",
                input
            )
            .into());
        }
    };

    let games = downloader.read_game_titles()?;

    println!("\nFound mods for the following games:");
    for (index, game) in games.iter().enumerate() {
        println!(
            "  {}) {}: {} mods",
            index,
            game.title_name,
            game.mod_download_urls.len()
        );
    }

    let proceed = get_input("\nDo you want to proceed to the download [Y/n]: ")?;
    match proceed.as_str() {
        "Y" => {
            downloader.download_mods(&games)?;
            println!("Operation successfull.");
        }
        _ => println!("Operation canceled."),
    }

    Ok(())
}
