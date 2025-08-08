use crate::{EMU_NAME, utils::read_lines};

pub trait TitleVersionExt {
    fn get_title_version(&self, title_id: &str) -> Result<String, Box<dyn std::error::Error>>;
}

impl<M: super::mod_downloader::ModDownloader> TitleVersionExt for M {
    fn get_title_version(&self, title_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let default_version = "1.0.0";
        let mut pv_path = dirs::cache_dir().unwrap();
        pv_path.push(&EMU_NAME.get().unwrap());
        pv_path.push("game_list");
        pv_path.push(format!("{}.pv.txt", title_id));

        if !pv_path.exists() {
            return Ok(default_version.into());
        }

        for line in read_lines(pv_path)? {
            let line = line?;
            if line.starts_with("Update (") {
                let from = line.find("(").map(|i| i + 1).unwrap();
                let to = line.rfind(")").unwrap();
                return Ok(line[from..to].into());
            }
        }
        Ok(default_version.into())
    }
}
