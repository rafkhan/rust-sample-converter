use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

fn tree<'a>(directory: &String, wav_list: &'a mut Vec<PathBuf>) -> &'a mut Vec<PathBuf> {
    let entries = fs::read_dir(directory).unwrap();
    for e in entries {
        match e {
            Ok(entry) => {
                let path_buf = entry.path();
                let path_string = &path_buf.to_str().unwrap().to_string();
                // dbg!(path_string);
                if path_buf.is_dir() {
                    tree(path_string, wav_list);
                } else {
                    let extension = path_buf.extension();
                    match extension {
                        Some(ext) => {
                            if ext == "wav" {
                                // dbg!(&path_buf);
                                wav_list.push(path_buf);
                            }
                        }
                        None => {}
                    }
                }
            }
            Err(_e) => {
                panic!("error reading directory: {directory}")
            }
        }
    }

    return wav_list;
}

fn check_wav_header(path: &Path) {
    let mut file = match File::open(path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    // Read enough bytes to find fmt chunk (some files have large bext chunks)
    let mut buffer = vec![0u8; 1024];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    // Verify RIFF header
    if bytes_read < 12 || &buffer[0..4] != b"RIFF" || &buffer[8..12] != b"WAVE" {
        eprintln!("Not a valid WAV file: {}", path.display());
        return;
    }

    // Search for "fmt " chunk
    let mut pos = 12; // Start after "RIFF" + size + "WAVE"
    while pos + 8 < bytes_read {
        let chunk_id = &buffer[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            buffer[pos + 4],
            buffer[pos + 5],
            buffer[pos + 6],
            buffer[pos + 7],
        ]) as usize;

        if chunk_id == b"fmt " {
            // fmt chunk found - sample rate is at offset 4 within the chunk data
            let sample_rate_offset = pos + 8 + 4;
            if sample_rate_offset + 4 <= bytes_read {
                let sample_rate = u32::from_le_bytes([
                    buffer[sample_rate_offset],
                    buffer[sample_rate_offset + 1],
                    buffer[sample_rate_offset + 2],
                    buffer[sample_rate_offset + 3],
                ]);
                println!("{}, {}", sample_rate, path.display());
            }
            return;
        }

        // Move to next chunk (chunk header is 8 bytes + chunk data)
        pos += 8 + chunk_size;
    }

    eprintln!("No fmt chunk found: {}", path.display());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = &args[1];

    let mut wav_list: Vec<PathBuf> = Vec::new();
    let updated_wav_list = tree(dir, &mut wav_list);

    for wav_path in updated_wav_list {
        check_wav_header(&wav_path.as_path());
    }
}
