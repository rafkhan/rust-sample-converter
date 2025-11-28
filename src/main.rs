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
    let header_size = 44;
    let mut file = match File::open(path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    let mut header_buffer = vec![0u8; header_size];
    match file.read_exact(&mut header_buffer) {
        Ok(_bytes) => {
            // println!("Successfully read {} bytes from {}", x, path.display());
            // dbg!(header_buffer)9
            let sample_rate_buff = &header_buffer[24..28];
            let sample_rate = i32::from_le_bytes([
                sample_rate_buff[0],
                sample_rate_buff[1],
                sample_rate_buff[2],
                sample_rate_buff[3],
            ]);

            let riff = &header_buffer[0..4];
            match str::from_utf8(riff) {
                Ok(riff_str) => {
                    dbg!(riff_str);
                }
                Err(_e) => {}
            }

            println!("{}, {}", sample_rate, path.display());

            // dbg!(num_ne);
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
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
