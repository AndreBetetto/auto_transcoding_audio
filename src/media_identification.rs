use std::process::Command;
use std::fs;

pub fn identify_flac_files(folder: &str) -> Vec<String> {
    let mut flac_files = Vec::new();

    // Scan the folder for FLAC files
    let paths = fs::read_dir(folder).unwrap();

    for path in paths {
        let path_str = path.unwrap().path().display().to_string();
        if path_str.ends_with(".flac") {    
            flac_files.push(path_str);
        }
    }

    flac_files
}

//get one flac file, and analyse channels, sample rate and bit depth
pub fn get_info_from_flac(flac_file: &str) -> Option<(String, String, String)> {
    let output = Command::new("sox")
        .args(&["--i", flac_file])
        .output()
        .expect("Failed to retrieve FLAC info");

    let info = String::from_utf8_lossy(&output.stdout);
    let mut channels = None;
    let mut sample_rate = None;
    let mut bit_depth = None;

    for line in info.lines() {
        if line.starts_with("Channels") {
            channels = Some(line.split(":").nth(1)?.trim().to_string());
        } else if line.starts_with("Sample Rate") {
            sample_rate = Some(line.split(":").nth(1)?.trim().to_string());
        } else if line.starts_with("Precision") {
            bit_depth = Some(line.split(":").nth(1)?.trim().to_string());
        }
    }

    match (channels, sample_rate, bit_depth) {
        (Some(c), Some(s), Some(b)) => Some((c, s, b)),
        _ => None,  // Return None if any of the values are missing
    }
}

pub fn get_metadata_from_flac(flac_file: &str) -> Option<(String, String, String)> {
    let output = Command::new("metaflac")
        .args(&["--export-tags-to=-", flac_file])
        .output()
        .expect("Failed to retrieve metadata");
    let metadata = String::from_utf8_lossy(&output.stdout);
    let mut artist = String::new();
    let mut album = String::new();
    let mut year = String::new();

    for line in metadata.lines() {
        let lower_line = line.to_lowercase();
        if lower_line.starts_with("artist=") {
            artist = line["ARTIST=".len()..].to_string();
        } else if lower_line.starts_with("album=") {
            album = line["ALBUM=".len()..].to_string();
        } else if lower_line.starts_with("date=") {
            year = line["DATE=".len()..].to_string();
        }
    }
    if !artist.is_empty() && !album.is_empty() && !year.is_empty() {
        Some((artist, album, year))
    } else {    
        None
    }
}