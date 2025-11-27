use std::env;
use std::fs;

fn tree(directory: &String) {
    let entries = fs::read_dir(directory).unwrap();
    for e in entries {
        match e {
            Ok(entry) => {
                let path = entry.path();
                if path.is_dir() {
                    dbg!(path);
                }
            }
            Err(_e) => {
                panic!("error reading directory: {directory}")
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = &args[1];
    tree(dir);
}
