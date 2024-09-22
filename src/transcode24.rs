//if flac file is 24bit:
//44.1 / 88.2 / 176.4 kHz is converted to 16-bit @ 44.1 kHz
//48 / 96 / 192 kHz is converted to 16-bit @ 48 kHz
use std::process::Command;
use std::fs;

//public function to identify flac sample rate and bit depth
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