mod table;

use crate::table::{WavHeader, WavTable};
use color_eyre::Result;
use std::cmp;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};


#[derive(Debug, Default)]
pub struct App<'a> {
    exit: bool,
    wav_list: Vec<WavHeader<'a>>,
    index: i32,
}

impl<'a> App<'a> {
    pub fn new(wav_list: Vec<WavHeader<'a>>) -> Self {
        Self {
            exit: false,
            wav_list,
            index: 0,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.prev(),
            KeyCode::Down => self.next(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn next(&mut self) {
        self.index = cmp::min(self.index + 1, self.wav_list.len() as i32 - 1);
    }

    fn prev(&mut self) {
        self.index = cmp::max(self.index - 1, 0);
    }
}

impl Widget for &App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" ~ OKAY SYNTHESIZER WAVCONVERT ~ ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<q> ".blue().bold()]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        let table = WavTable::new(&self.wav_list, self.index as usize);
        table.render(inner, buf);
    }
}

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

fn empty_wav_header(path: &Path) -> WavHeader<'_> {
    return WavHeader {
        bit_depth: 0,
        sample_rate: 0,
        path,
    };
}

fn get_wav_header(path: &Path) -> WavHeader<'_> {
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
            return empty_wav_header(path);
        }
    };

    // Verify RIFF header
    if bytes_read < 12 || &buffer[0..4] != b"RIFF" || &buffer[8..12] != b"WAVE" {
        eprintln!("Not a valid WAV file: {}", path.display());
        return empty_wav_header(path);
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
            let bit_depth_offset = sample_rate_offset + 10;

            if bit_depth_offset + 2 <= bytes_read {
                let sample_rate = u32::from_le_bytes([
                    buffer[sample_rate_offset],
                    buffer[sample_rate_offset + 1],
                    buffer[sample_rate_offset + 2],
                    buffer[sample_rate_offset + 3],
                ]);
                let bit_depth =
                    u16::from_le_bytes([buffer[bit_depth_offset], buffer[bit_depth_offset + 1]]);
                return WavHeader {
                    sample_rate: sample_rate,
                    bit_depth: bit_depth,
                    path: path,
                };
            }

            return empty_wav_header(path);
        }

        // Move to next chunk (chunk header is 8 bytes + chunk data)
        pos += 8 + chunk_size;
    }

    eprintln!("No fmt chunk found: {}", path.display());
    return empty_wav_header(path);
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let dir = &args[1];

    let mut wav_list: Vec<PathBuf> = Vec::new();
    let updated_wav_list = tree(dir, &mut wav_list);

    let filtered_wav_list: Vec<WavHeader> = updated_wav_list
        .iter()
        .filter_map(|wav_path| {
            let wav_header = get_wav_header(&wav_path);
            if wav_header.sample_rate != 44100 || wav_header.bit_depth != 16 {
                return Some(wav_header);
            } else {
                return None;
            }
        })
        .collect();

    for w in &filtered_wav_list {
        println!("{},\t{},\t{}", w.path.display(), w.sample_rate, w.bit_depth);
    }

    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let app_result = App::new(filtered_wav_list).run(&mut terminal);
    ratatui::restore();
    app_result?;
    Ok(())
}
