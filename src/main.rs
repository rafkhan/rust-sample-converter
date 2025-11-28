use std::env;
use std::fs;
use std::path::PathBuf;

fn tree<'a>(directory: &String, wav_list: &'a mut Vec<PathBuf>) -> &'a mut Vec<PathBuf> {
    let entries = fs::read_dir(directory).unwrap();
    for e in entries {
        match e {
            Ok(entry) => {
                let path_buf = entry.path();
                let path_string = &path_buf.to_str().unwrap().to_string();
                dbg!(path_string);
                if path_buf.is_dir() {
                    tree(path_string, wav_list);
                } else {
                    let extension = path_buf.extension();
                    match extension {
                        Some(ext) => {
                            if ext == "wav" {
                                dbg!(&path_buf);
                                wav_list.push(path_buf);
                            }
                        }
                        None => {
                            dbg!("ignored");
                        }
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = &args[1];

    let mut wav_list: Vec<PathBuf> = Vec::new();
    let updated_wav_list = tree(dir, &mut wav_list);
    dbg!(updated_wav_list);
}
