use std::fs;
use std::io;
use std::path::Path;

use sevenz_rust::SevenZReader;
use unrar::Archive;
use zip::ZipArchive;

#[derive(Debug)]
enum ArchiveFormat {
    Zip,
    Rar,
    SevenZ,
}

fn detect_format<P: AsRef<Path>>(path: P) -> Option<ArchiveFormat> {
    let extension = path.as_ref().extension()?.to_str()?.to_lowercase();

    match extension.as_str() {
        "zip" => Some(ArchiveFormat::Zip),
        "rar" => Some(ArchiveFormat::Rar),
        "7z" => Some(ArchiveFormat::SevenZ),
        _ => None,
    }
}

fn extract_zip<P: AsRef<Path>>(
    archive_path: P,
    output_dir: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => output_dir.as_ref().join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

fn extract_rar<P: AsRef<Path>>(
    archive_path: P,
    output_dir: P,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&output_dir)?;

    let mut archive = Archive::new(archive_path.as_ref()).open_for_processing()?;

    loop {
        match archive.read_header()? {
            Some(header_archive) => {
                let entry = header_archive.entry();
                let output_path = output_dir.as_ref().join(&entry.filename);

                if entry.is_file() {
                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    archive = header_archive.extract_to(&output_path)?;
                } else {
                    archive = header_archive.skip()?;
                }
            }
            None => break,
        }
    }

    Ok(())
}

fn extract_7z<P: AsRef<Path>>(
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

pub fn extract_archive<P: AsRef<Path>>(
    archive_path: P,
    output_dir: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let format = detect_format(&archive_path).ok_or("Unsupported archive format")?;

    match format {
        ArchiveFormat::Zip => extract_zip(archive_path, output_dir)?,
        ArchiveFormat::Rar => extract_rar(archive_path, output_dir)?,
        ArchiveFormat::SevenZ => extract_7z(archive_path, output_dir)?,
    }

    Ok(())
}
