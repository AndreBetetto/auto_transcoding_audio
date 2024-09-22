use std::process::Command;
use std::fs::{create_dir_all, OpenOptions};
use std::path::Path;
use std::io::Write;

pub fn generate_spectrograms(flac_files: Vec<String>, folder: &str) {
    // Create the spectrogram folder inside the given folder
    let spectrogram_dir = format!("{}/spectrograms", folder);
    if !Path::new(&spectrogram_dir).exists() {
        create_dir_all(&spectrogram_dir).expect("Failed to create spectrogram directory");
    }

    // Create or open the log file
    let log_file_path = format!("{}/spectrograms/spectrogram_log.txt", folder);
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .expect("Failed to open log file");

    // Select files (at least 1 and at most 3: first, middle, and last track)
    let flac_count = flac_files.len();
    let selected_files: Vec<String> = match flac_count {
        0 => {
            eprintln!("No FLAC files found.");
            return;
        }
        1 => vec![flac_files[0].clone()],
        2 => vec![flac_files[0].clone(), flac_files[flac_count - 1].clone()],
        _ => vec![
            flac_files[0].clone(),
            flac_files[flac_count / 2].clone(),
            flac_files[flac_count - 1].clone(),
        ],
    };

    for flac_file in selected_files {
        let file_name = Path::new(&flac_file).file_stem().unwrap().to_str().unwrap();

        // Use SoX to get file info
        let sox_info_output = Command::new("sox")
            .args(&["--i", &flac_file])
            .output()
            .expect("Failed to retrieve FLAC info");

        // Log the FLAC file info
        writeln!(log_file, "FLAC File: ./flac/{}.flac", file_name).unwrap();
        writeln!(log_file, "FLAC Info:\n{}", String::from_utf8_lossy(&sox_info_output.stdout)).unwrap();

        // Generate full spectrogram
        let full_spectrogram = format!("{}/{}-full.png", spectrogram_dir, file_name);
        let full_spectrogram_command = format!("sox ./flac/{}.flac -n remix 1 spectrogram -x 3000 -y 513 -z 120 -w Kaiser -o ./spectro/{}-full.png", file_name, file_name);
        let full_spectrogram_output = Command::new("sox")
            .args(&[&flac_file, "-n", "remix", "1", "spectrogram", "-x", "3000", "-y", "513", "-z", "120", "-w", "Kaiser", "-o", &full_spectrogram])
            .output()
            .expect("Failed to generate full spectrogram");

        if !full_spectrogram_output.status.success() {
            eprintln!("Error generating full spectrogram: {}", String::from_utf8_lossy(&full_spectrogram_output.stderr));
        }

        // Log the command used for full spectrogram
        writeln!(log_file, "Command for full spectrogram: {}", full_spectrogram_command).unwrap();

        // Generate zoomed spectrogram
        let zoom_spectrogram = format!("{}/{}-zoom.png", spectrogram_dir, file_name);
        let zoom_spectrogram_command = format!("sox ./flac/{}.flac -n remix 1 spectrogram -X 500 -y 1025 -z 120 -w Kaiser -S 1:00 -d 0:02 -o ./spectro/{}-zoom.png", file_name, file_name);
        let zoom_spectrogram_output = Command::new("sox")
            .args(&[&flac_file, "-n", "remix", "1", "spectrogram", "-X", "500", "-y", "1025", "-z", "120", "-w", "Kaiser", "-S", "1:00", "-d", "0:02", "-o", &zoom_spectrogram])
            .output()
            .expect("Failed to generate zoom spectrogram");

        if !zoom_spectrogram_output.status.success() {
            eprintln!("Error generating zoom spectrogram: {}", String::from_utf8_lossy(&zoom_spectrogram_output.stderr));
        }

        // Log the command used for zoom spectrogram
        writeln!(log_file, "Command for zoom spectrogram: {}", zoom_spectrogram_command).unwrap();

        // Add separator
        writeln!(log_file, "\n=========\n").unwrap();
    }

    println!("Spectrogram generation completed. Log file saved to: {}", log_file_path);
}
            