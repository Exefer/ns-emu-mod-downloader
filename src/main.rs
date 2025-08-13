mod curl_helper;
mod entities;
mod mod_downloader;
mod utils;
use mod_downloader::ModDownloader;
use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
    sync::OnceLock,
};

// TODO: Create a config struct
pub(crate) static EMU_NAME: OnceLock<String> = OnceLock::new();

fn get_input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn display_options<T: std::fmt::Display>(title: &str, items: &[T]) {
    println!("{}:", title);
    for (index, item) in items.iter().enumerate() {
        println!("  {}) {}", index + 1, item);
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

    display_options("\nSelect an emulator to download mods for", &emus);
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

        return Err(format!("Emulator '{}' is not installed on this system.", emu,).into());
    }

    Ok(emu.into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Mod Downloader ===");

    let emu = get_emu()?;

    EMU_NAME.set(emu).unwrap();

    let repos: HashMap<&str, &str> = [
        ("1", "Bellerof/switch-pchtxt-mods"),
        ("2", "Bellerof/Switch-Ultrawide-Mods"),
        ("3", "Bellerof/ue4-emuswitch-60fps"),
        ("4", "Bellerof/switch-port-mods"),
    ]
    .into();

    display_options(
        "\nSelect a repository to download mods from",
        &repos.values().collect::<Vec<_>>(),
    );

    let input = get_input(&format!("\nEnter your choice [1-{}]: ", repos.keys().len()))?;

    let repo = repos.get(input.as_str()).ok_or_else(|| {
        format!(
            "Invalid option '{}'. Please choose 1 to {}.",
            input,
            repos.keys().len()
        )
    })?;

    let mut downloader: ModDownloader = ModDownloader::new(repo.to_string());

    let games = downloader.read_game_titles()?;

    if games.is_empty() {
        return Err("No mod installation folders found on this system.".into());
    }

    let games: Vec<_> = games
        .into_iter()
        .filter(|game| !game.mod_download_entries.is_empty())
        .collect();

    if games.is_empty() {
        println!("No mods available for any installed game.");
        return Ok(());
    }

    println!("\nFound mods for the following games:");
    for (index, game) in games.iter().enumerate() {
        let mods = game
            .mod_download_entries
            .iter()
            .filter_map(|entry| {
                entry
                    .mod_relative_path
                    .split_once("/")
                    .map(|(first, _)| first)
            })
            .collect::<HashSet<&str>>();
        println!("  {}) {}: {} mods", index + 1, game.title_name, mods.len());
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
