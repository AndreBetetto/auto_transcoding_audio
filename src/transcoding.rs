use std::process::{Command, Stdio};
use std::fs::{rename, OpenOptions, remove_file,};
use std::io::Write;
use std::path::Path;

//when 24 bits, convert to 16 bits
pub fn transcode_24_to_16 (flac_file: &str, folder: &str){
    //copy flac file and rename to flac_file_16
    let output_flac = format!("{}_16.flac", flac_file.trim_end_matches(".flac"));
    let _ = std::fs::copy(flac_file, output_flac.clone());
    let flac_info = Command::new("sox")
        .args(&["--i", flac_file])
    .output()   
        .expect("Failed to retrieve FLAC info");
    let info = String::from_utf8_lossy(&flac_info.stdout);

    let mut sample_rate = String::new();
    //let mut bit_depth = String::new();
    //let mut channels = String::new(); // For stereo/mono info     

    for line in info.lines() {      
        if line.starts_with("Sample Rate") {
            sample_rate = line["Sample Rate   : ".len()..].to_string();
        } else if line.starts_with("Precision") {
            //bit_depth = line["Precision     : ".len()..].to_string();
        } else if line.starts_with("Channels") {
            //channels = line["Channels      : ".len()..].to_string();
        }
    }

    if sample_rate == " 44100" || sample_rate == " 88200" || sample_rate == " 176400" {
        // Convert to 16-bit @ 44.1 kHz
        Command::new("sox")
            .args(&["-S", flac_file, "-b", "16", &output_flac, "rate", "-v", "-L", "44100", "dither"])
            .output()   
            .expect("Failed to convert FLAC to 16-bit 44.1 kHz");
    } else if sample_rate == " 48000" || sample_rate == " 96000" || sample_rate == " 192000" {
        // Convert to 16-bit @ 48 kHz
        Command::new("sox")
            .args(&["-S", flac_file, "-b", "16", &output_flac, "rate", "-v", "-L", "48000", "dither"])
            .output()   
            .expect("Failed to convert FLAC to 16-bit 48 kHz");
    }
    let flac_file_trimmed = Path::new(flac_file)
            .file_name()
            .unwrap()               
            .to_str()
            .unwrap();      
    let output_flac_trimmed = Path::new(&output_flac)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
    let mut log_file = OpenOptions::new()
        .append(true)
        .open(format!("{}/transcoding_log24.txt", folder))
        .expect("Failed to open log file");
    writeln!(log_file, "Converted '{}' from 24-bit to 16-bit at {} Hz", flac_file_trimmed, sample_rate).unwrap();
    if sample_rate == " 44100" || sample_rate == " 88200" || sample_rate == " 176400" {
        writeln!(log_file, "Command used: sox -S '{}' -b 16 '{}' rate -v -L {} dither", flac_file_trimmed, output_flac_trimmed, sample_rate).unwrap();
    } else if sample_rate == " 48000" || sample_rate == " 96000" || sample_rate == " 192000" {
        writeln!(log_file, "Command used: sox -S '{}' -b 16 '{}' rate -v -L {} dither", flac_file_trimmed, output_flac_trimmed, sample_rate).unwrap();
    }
    writeln!(log_file, "\n====================\n").unwrap();
    //get metadata
    copy_tags(flac_file, &output_flac);
    //transcode the new flac to vbr
    transcode_flac_vbr(&output_flac, folder, true, &sample_rate);
    //transcode the new flac to cbr
    transcode_flac_cbr(&output_flac, folder, true, &sample_rate);
    //move flac file to flac24 folder
    let _ = rename(&output_flac, Path::new(&output_flac).with_file_name("flac").join(Path::new(&output_flac).file_name().unwrap()));  
    let _ = rename(&flac_file, Path::new(&flac_file).with_file_name("flac24").join(Path::new(&flac_file).file_name().unwrap()));

}
    
pub fn transcode_flac_vbr(flac_file: &str, folder: &str, transcode_24_to_16: bool, freq: &str){
    // Retrieve FLAC file information using SoX
    let flac_info = Command::new("sox")
        .args(&["--i", flac_file])
        .output()
        .expect("Failed to retrieve FLAC info");
    let info = String::from_utf8_lossy(&flac_info.stdout);

    let mut sample_rate = String::new();
    let mut bit_depth = String::new();
    let mut channels = String::new();

    for line in info.lines() {
        if line.starts_with("Sample Rate") {
            sample_rate = line["Sample Rate   : ".len()..].trim().to_string();
        } else if line.starts_with("Precision") {
            bit_depth = line["Precision     : ".len()..].trim().to_string();
        } else if line.starts_with("Channels") {
            channels = line["Channels      : ".len()..].trim().to_string();
        }
    }

    // Extract raw PCM from the appropriate FLAC file using FFMPEG
    let pcm_output = Command::new("ffmpeg")
        .args(&["-i", flac_file, "-f", "wav", "-"])
        .output()
        .expect("Failed to extract PCM from FLAC");

    // Use LAME command-line to encode PCM to MP3 VBR 0
    let output_mp3 = format!("{}.mp3", flac_file.trim_end_matches(".flac"));
    // Write PCM output to LAME's stdin, without moving lame_cmd
    let mut lame_cmd = Command::new("lame")
        .args(&["-V0 --vbr-new", "-", &output_mp3])
        .stdin(Stdio::piped())   // Use piped stdin for LAME
        .spawn()
        .expect("Failed to start LAME");

    // Write PCM output to LAME's stdin, without moving lame_cmd
    if let Some(stdin) = lame_cmd.stdin.as_mut() {
        stdin.write_all(&pcm_output.stdout).unwrap();
    }

    // Wait for LAME to finish
    let result = lame_cmd.wait_with_output().expect("Failed to run LAME command");

    if !result.status.success() {
        eprintln!("LAME failed with error: {}", String::from_utf8_lossy(&result.stderr));
    } else {
        // After transcoding, handle tags
        copy_tags(flac_file, &output_mp3);
        //move file to the mp3_vbr folder
        let _ = rename(&output_mp3, Path::new(&output_mp3).with_file_name("mp3_vbr").join(Path::new(&output_mp3).file_name().unwrap()));
        let mut log_file = OpenOptions::new()
            .append(true)
            .open(format!("{}/transcoding_logvbr.txt", folder))
            .expect("Failed to open transcoding log file");
        //trim flac_file to get only the name of the file in flac_file_trimmed
        let flac_file_trimmed = Path::new(flac_file)
            .file_name()
            .unwrap()               
            .to_str()
            .unwrap();      
        let output_mp3_trimmed = Path::new(&output_mp3)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        writeln!(log_file, "File: '{}'", flac_file_trimmed).unwrap();
        writeln!(log_file, "Sample Rate: {}", sample_rate).unwrap();
        writeln!(log_file, "Bit Depth: {}", bit_depth).unwrap();
        writeln!(log_file, "Original Codec: FLAC").unwrap();
        writeln!(log_file, "Transcoded to: MP3 VBR 0").unwrap();
        writeln!(log_file, "Channels: {}", channels).unwrap();
        if transcode_24_to_16{
            //tell the previously conversion
            writeln!(log_file, "\nConverted '{}' from 24-bit to 16-bit at {} Hz", flac_file_trimmed, sample_rate).unwrap();
            if freq == "44100" || freq == "88200" || freq == "176400" {             
                writeln!(log_file, "Command used: sox -S '{}' -b 16 '{}' rate -v -L {} dither", flac_file_trimmed, output_mp3_trimmed, sample_rate).unwrap();
            } else if freq == " 48000" || freq == " 96000" || freq == " 192000" {
                writeln!(log_file, "Command used: sox -S '{}' -b 16 '{}' rate -v -L {} dither", flac_file_trimmed, output_mp3_trimmed, sample_rate).unwrap();
            }
        }
        writeln!(log_file, "Command used: ffmpeg -i '{}' -f wav - | lame -V0 --vbr-new - '{}'", flac_file_trimmed, output_mp3_trimmed).unwrap();
        writeln!(log_file, "\n====================\n").unwrap();
        //move mp3 file to mp3_vbr folder
        let _ = rename(&output_mp3, Path::new(&output_mp3).with_file_name("mp3_vbr").join(Path::new(&output_mp3).file_name().unwrap()));
    }
}

pub fn transcode_flac_cbr(flac_file: &str, folder: &str, transcode_24_to_16: bool, freq: &str){
    // Retrieve FLAC file information using SoX
    let flac_info = Command::new("sox")
        .args(&["--i", flac_file])
        .output()
        .expect("Failed to retrieve FLAC info");
    let info = String::from_utf8_lossy(&flac_info.stdout);

    let mut sample_rate = String::new();
    let mut bit_depth = String::new();
    let mut channels = String::new();

    for line in info.lines() {
        if line.starts_with("Sample Rate") {
            sample_rate = line["Sample Rate   : ".len()..].trim().to_string();
        } else if line.starts_with("Precision") {
            bit_depth = line["Precision     : ".len()..].trim().to_string();
        } else if line.starts_with("Channels") {
            channels = line["Channels      : ".len()..].trim().to_string();
        }
    }

    // Extract raw PCM from the appropriate FLAC file using FFMPEG
    let pcm_output = Command::new("ffmpeg")
        .args(&["-i", flac_file, "-f", "wav", "-"])
        .output()
        .expect("Failed to extract PCM from FLAC");

    // Use LAME command-line to encode PCM to MP3 VBR 0
    let output_mp3 = format!("{}.mp3", flac_file.trim_end_matches(".flac"));
    // Write PCM output to LAME's stdin, without moving lame_cmd
    let mut lame_cmd = Command::new("lame")
        .args(&["-b 320", "-", &output_mp3])
        .stdin(Stdio::piped())   // Use piped stdin for LAME
        .spawn()
        .expect("Failed to start LAME");

    // Write PCM output to LAME's stdin, without moving lame_cmd
    if let Some(stdin) = lame_cmd.stdin.as_mut() {
        stdin.write_all(&pcm_output.stdout).unwrap();
    }

    // Wait for LAME to finish
    let result = lame_cmd.wait_with_output().expect("Failed to run LAME command");

    if !result.status.success() {
        eprintln!("LAME failed with error: {}", String::from_utf8_lossy(&result.stderr));
    } else {
        // After transcoding, handle tags
        copy_tags(flac_file, &output_mp3);
        //move file to the mp3_vbr folder
        let _ = rename(&output_mp3, Path::new(&output_mp3).with_file_name("mp3_cbr").join(Path::new(&output_mp3).file_name().unwrap()));
        let mut log_file = OpenOptions::new()
            .append(true)
            .open(format!("{}/transcoding_logcbr.txt", folder))
            .expect("Failed to open transcoding log file");
        //trim flac_file to get only the name of the file in flac_file_trimmed
        let flac_file_trimmed = Path::new(flac_file)
            .file_name()
            .unwrap()               
            .to_str()
            .unwrap();      
        let output_mp3_trimmed = Path::new(&output_mp3)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        writeln!(log_file, "File: '{}'", flac_file_trimmed).unwrap();
        writeln!(log_file, "Sample Rate: {}", sample_rate).unwrap();
        writeln!(log_file, "Bit Depth: {}", bit_depth).unwrap();
        writeln!(log_file, "Original Codec: FLAC").unwrap();
        writeln!(log_file, "Transcoded to: MP3 CBR 320").unwrap();
        writeln!(log_file, "Channels: {}", channels).unwrap();
        if transcode_24_to_16{
            //tell the previously conversion
            writeln!(log_file, "\nConverted '{}' from 24-bit to 16-bit at {} Hz", flac_file_trimmed, freq).unwrap();
            if freq == "44100" || freq == "88200" || freq == "176400" {             
                writeln!(log_file, "Command used: sox -S '{}' -b 16 '{}' rate -v -L {} dither", flac_file, flac_file_trimmed, sample_rate).unwrap();
            } else if freq == " 48000" || freq == " 96000" || freq == " 192000" {
                writeln!(log_file, "Command used: sox -S '{}' -b 16 '{}' rate -v -L {} dither", flac_file, flac_file_trimmed, sample_rate).unwrap();
            }    
        }
        writeln!(log_file, "Command used: ffmpeg -i '{}' -f wav - | lame -b 320 - '{}'", flac_file_trimmed, output_mp3_trimmed).unwrap();
        writeln!(log_file, "\n====================\n").unwrap();
        //move mp3 file to mp3_cbr folder
        let _ = rename(&output_mp3, Path::new(&output_mp3).with_file_name("mp3_cbr").join(Path::new(&output_mp3).file_name().unwrap()));
    }
}

pub fn transcode_to_mp3(flac_file: &str, folder: &str){
    // Retrieve FLAC file information using SoX
    let flac_info = Command::new("sox")
        .args(&["--i", flac_file])
        .output()
        .expect("Failed to retrieve FLAC info");
    let info = String::from_utf8_lossy(&flac_info.stdout);

    //let mut sample_rate = String::new();
    let mut bit_depth = String::new();
    //let mut channels = String::new();

    for line in info.lines() {
        if line.starts_with("Sample Rate") {
            //sample_rate = line["Sample Rate   : ".len()..].trim().to_string();
        } else if line.starts_with("Precision") {
            bit_depth = line["Precision     : ".len()..].trim().to_string();
        } else if line.starts_with("Channels") {
            //channels = line["Channels      : ".len()..].trim().to_string();
        }
    }

    // Check if the FLAC file is 24-bit
    if bit_depth == "24-bit" {
        transcode_24_to_16(flac_file, folder);
    } else {
        transcode_flac_cbr(flac_file, folder, false, "0");
        transcode_flac_vbr(flac_file, folder, false, "0");
    }
    //move flac file to flac folder
    let _ = rename(flac_file, Path::new(flac_file).with_file_name("flac").join(Path::new(flac_file).file_name().unwrap()));     
    //variable 24bits for false
}

fn copy_tags(flac_file: &str, mp3_file: &str) {
    let temp_metadata = "/tmp/metadata.txt";
    let temp_cover = "/tmp/cover.jpg";

    // Export FLAC metadata to a temporary .txt file
    let tag_output = Command::new("metaflac")
        .args(&["--export-tags-to", temp_metadata, flac_file])
        .output()
        .expect("Failed to extract FLAC tags");

    if !tag_output.status.success() {
        eprintln!("Error extracting tags from FLAC: {}", String::from_utf8_lossy(&tag_output.stderr));
        return;
    }

    // Apply the tags to the MP3 file using id3v2
    apply_id3_tags(temp_metadata, mp3_file);

    // Export album art to a temporary .jpg file (if exists)
    let cover_output = Command::new("metaflac")
        .args(&["--export-picture-to", temp_cover, flac_file])
        .output()
        .expect("Failed to extract cover art");
    //if image is 1:1 and not 500x500, use imagemagick to resize it, else, rescale to width = 500
    if !cover_output.status.success() {
        eprintln!("Error extracting cover art: {}", String::from_utf8_lossy(&cover_output.stderr));
    } else {
        let cover_size = Command::new("identify")
            .args(&["-format", "%w %h", temp_cover])
            .output()   
            .expect("Failed to get cover art size");
        let cover_size = String::from_utf8_lossy(&cover_size.stdout);
        let cover_size = cover_size.trim().split(" ").collect::<Vec<&str>>();
        if cover_size[0] != cover_size[1] {
            let resize_output = Command::new("convert")
                .args(&[temp_cover, "-resize", "500x500", temp_cover])
                .output()
                .expect("Failed to resize cover art");
            if !resize_output.status.success() {
                eprintln!("Error resizing cover art: {}", String::from_utf8_lossy(&resize_output.stderr));
            }
        } else {
            let rescale_output = Command::new("convert")
                .args(&[temp_cover, "-resize", "500", temp_cover])
                .output()
                .expect("Failed to rescale cover art");
            if !rescale_output.status.success() {
                eprintln!("Error rescaling cover art: {}", String::from_utf8_lossy(&rescale_output.stderr));
            }
        }
    }
    
    if Path::new(temp_cover).exists() {
        // Use mid3v2 to add the cover art  
        let apply_cover_output = Command::new("mid3v2")
            .args(&["-p", temp_cover, mp3_file])
            .output()
            .expect("Failed to add cover art to MP3");

        if !apply_cover_output.status.success() {
            eprintln!("Error adding cover art: {}", String::from_utf8_lossy(&apply_cover_output.stderr));
        }
    }

    // Cleanup temporary files
    let _ = remove_file(temp_metadata);
    let _ = remove_file(temp_cover);
}

fn apply_id3_tags(metadata_file: &str, mp3_file: &str) {
    // Read the metadata from the temporary file
    let metadata = std::fs::read_to_string(metadata_file).expect("Failed to read metadata file");

    for line in metadata.lines() {
        let mut parts = line.splitn(2, '=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            // Map FLAC tag names to ID3 tag names if necessary
            let id3_key = match key.to_lowercase().as_str() {
                "artist" => "--artist",
                "composer" => "--composer",
                "album" => "--album",
                "title" => "--song",
                "tracknumber" => "--track",
                "date" => "--year",
                "genre" => "--genre",
                _ => continue, // Skip unsupported tags
            };

            // Apply each tag using id3v2
            let id3_output = Command::new("id3v2")
                .args(&[id3_key, value, mp3_file])      
                .output()
                .expect("Failed to write ID3 tags to MP3");

            if !id3_output.status.success() {
                eprintln!("Error applying tag {}: {}", id3_key, String::from_utf8_lossy(&id3_output.stderr));
            }
        }
    }
}               