use sevenz_rust::SevenZReader;
use std::fs;
use std::path::Path;

pub fn extract_archive<P: AsRef<Path>>(
    archive_path: P,
    output_dir: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut sz = SevenZReader::open(archive_path, sevenz_rust::Password::empty())?;

    fs::create_dir_all(&output_dir)?;

    sz.for_each_entries(|entry, reader| {
        if !entry.is_directory() {
            let output_path = output_dir.as_ref().join(&entry.name);

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut output_file = fs::File::create(&output_path)?;
            std::io::copy(reader, &mut output_file)?;
        } else {
            let dir_path = output_dir.as_ref().join(&entry.name);
            fs::create_dir_all(dir_path)?;
        }
        Ok(true)
    })?;

    Ok(())
}
