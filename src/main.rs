mod media_identification;
mod transcoding;
mod spectrogram;

use std::env;
//use log::{info, debug, error};
use std::fs::File;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} -T <folder>", args[0]);
        return;
    }

    let folder = &args[2];    
    if args[1] == "-T" {
        // Identify media using FFMPEG or SOX
        let flac_files = media_identification::identify_flac_files(folder);
        //get the first flac file to get the metadata of the folder
        let metadata = media_identification::get_metadata_from_flac(&flac_files[0]).unwrap();
        //create three folders inside the given folder, one for flac, other to mp3 CBR and other to mp3 VBR in format: artist - album (year) [codec]
        let flac_folder = format!("{}/", folder);
        let mp3_cbr_folder = format!("{}/", folder);
        let mp3_vbr_folder = format!("{}/", folder);
        let artist_album_year_cbr = format!("{}/{} - {} ({}) [MP3_CBR]", mp3_cbr_folder, metadata.0, metadata.1, metadata.2);
        let artist_album_year_vbr = format!("{}/{} - {} ({}) [MP3_VBR]", mp3_vbr_folder, metadata.0, metadata.1, metadata.2);
        let artist_album_year_flac = format!("{}/{} - {} ({}) [FLAC]", flac_folder, metadata.0, metadata.1, metadata.2);
        let _ = std::fs::create_dir_all(&artist_album_year_flac);
        let _ = std::fs::create_dir_all(&artist_album_year_cbr);
        let _ = std::fs::create_dir_all(&artist_album_year_vbr);
        //create dirs for flac, mp3_cbr and mp3_vbr 
        let _ = std::fs::create_dir_all(format!("{}/flac", folder));
        let _ = std::fs::create_dir_all(format!("{}/flac24", folder));
        let _ = std::fs::create_dir_all(format!("{}/mp3_cbr", folder));
        let _ = std::fs::create_dir_all(format!("{}/mp3_vbr", folder));
        //let mut log_file = std::fs::File::create(format!("{}/transcoding_log.txt", folder)).expect("Failed to create log file");
        let _ = File::create(format!("{}/transcoding_logcbr.txt", folder)).unwrap();
        let _ = File::create(format!("{}/transcoding_logvbr.txt", folder)).unwrap();
        let _ = File::create(format!("{}/transcoding_log24.txt", folder)).unwrap();

        // Transcode each FLAC to MP3 and get folder name
        for flac_file in flac_files {   
            transcoding::transcode_to_mp3(&flac_file, folder);
        }
        //move all files from flac folder to the artist_album_year_flac folder
        let _ = std::fs::rename(format!("{}/flac", folder), &artist_album_year_flac);
        //move all files from mp3_cbr folder to the artist_album_year_cbr folder
        let _ = std::fs::rename(format!("{}/mp3_cbr", folder), &artist_album_year_cbr);
        //move all files from mp3_vbr folder to the artist_album_year_vbr folder
        let _ = std::fs::rename(format!("{}/mp3_vbr", folder), &artist_album_year_vbr);
        //move log file
        let _ = std::fs::rename(format!("{}/transcoding_logcbr.txt", folder), format!("{}/transcoding_logcbr.txt", &artist_album_year_cbr));
        let _ = std::fs::rename(format!("{}/transcoding_logvbr.txt", folder), format!("{}/transcoding_logvbr.txt", &artist_album_year_vbr));
        let _ = std::fs::rename(format!("{}/transcoding_log24.txt", folder), format!("{}/transcoding_log24.txt", &artist_album_year_flac));
        
        
    } else if args[1] == "-S" {
        // Identify FLAC files
        let flac_files = media_identification::identify_flac_files(folder);
        
        // Generate spectrograms for selected FLAC files
        spectrogram::generate_spectrograms(flac_files, folder);
    } else {
        eprintln!("Unknown option: {}", args[1]);
    }               
}   