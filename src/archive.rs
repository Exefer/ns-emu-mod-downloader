use std::path::Path;

pub fn extract_archive<P: AsRef<Path>>(
    archive_path: P,
    output_dir: P,
) -> Result<(), Box<dyn std::error::Error>> {
    sevenz_rust::decompress_file(archive_path, output_dir)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
