use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use regex::Regex; // Add regex = "1" to your Cargo.toml

use rusqlite::Connection;
use zip::ZipArchive;

#[derive(PartialEq)]
enum LearnState {
    Question,
    Answer,
}

struct Flashcard {
    question: String,
    answer: String,
}

pub struct LearnPage {
    path: PathBuf,
    deck: Vec<Flashcard>,
    current_index: usize,
    state: LearnState,
    renderer: FontRenderer,
    ui_renderer: FontRenderer,
}

impl LearnPage {
    pub fn new(path: PathBuf) -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/Inter_28pt-Medium.ttf");
        let ui_renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");

        let mut page = Self {
            path,
            deck: Vec::new(),
            current_index: 0,
            state: LearnState::Question,
            renderer,
            ui_renderer,
        };

        page.load_apkg();
        page
    }

    /// Cleans HTML tags out of Anki strings
    fn clean_html(input: &str) -> String {
        let re = Regex::new(r"<[^>]*>").unwrap();
        re.replace_all(input, " ")
            .replace("&nbsp;", " ")
            .replace("&gt;", ">")
            .replace("&lt;", "<")
            .trim()
            .to_string()
    }

    fn load_apkg(&mut self) {
        let temp_db_path = "/tmp/kram_anki.db";
        println!("Attempting to load: {:?}", self.path);

        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) => {
                println!("Failed to open file: {}", e);
                return;
            }
        };

        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(e) => {
                println!("Failed to read Zip: {}", e);
                return;
            }
        };

        // 1. Find the SQLite database inside the Zip
        let mut target_entry_name = None;
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                let name = file.name();
                if name == "collection.anki21" || name == "collection.anki2" {
                    target_entry_name = Some(name.to_string());
                    break;
                }
            }
        }

        let entry_name = match target_entry_name {
            Some(name) => name,
            None => {
                println!("No collection.anki2 or .anki21 found in package.");
                return;
            }
        };

        // 2. Extract DB to /tmp
        let mut archive_file = archive.by_name(&entry_name).unwrap();
        let mut buffer = Vec::new();
        archive_file.read_to_end(&mut buffer).unwrap();
        std::fs::write(temp_db_path, buffer).expect("Failed to write temp DB");

        // 3. Query the Database
        if let Ok(conn) = rusqlite::Connection::open(temp_db_path) {
            // We query NOTES directly. This ignores if they are in a "deck" yet.
            // In Anki, 'flds' contains all fields separated by 0x1f
            let mut stmt = conn.prepare("SELECT flds FROM notes").unwrap();
            
            let flashcard_iter = stmt.query_map([], |row| {
                let flds: String = row.get(0)?;
                let parts: Vec<String> = flds
                    .split('\x1f')
                    .map(|s| Self::clean_html(s))
                    .collect();

                Ok(Flashcard {
                    question: parts.get(0).cloned().unwrap_or_else(|| "No Question".to_string()),
                    answer: parts.get(1).cloned().unwrap_or_else(|| "No Answer".to_string()),
                })
            }).unwrap();

            self.deck = flashcard_iter
                .filter_map(|r| r.ok())
                .filter(|card| !card.question.is_empty())
                .collect();

            println!("Successfully loaded {} cards from notes table.", self.deck.len());
        }

        let _ = std::fs::remove_file(temp_db_path);
    }

    fn next_card(&mut self) {
        if !self.deck.is_empty() {
            self.current_index = (self.current_index + 1) % self.deck.len();
            self.state = LearnState::Question;
        }
    }

    fn draw_centered_wrapped_text(&self, display: &mut SharpDisplay, text: &str, ctx: &Context) {
        let font_size = 24.0;
        let margin = 20;
        let max_width = 400 - (margin * 2);
        
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            let test_str = if current_line.is_empty() { 
                word.to_string() 
            } else { 
                format!("{} {}", current_line, word) 
            };

            if self.renderer.calculate_width(&test_str, font_size) as i32 > max_width {
                lines.push(current_line);
                current_line = word.to_string();
            } else {
                current_line = test_str;
            }
        }
        lines.push(current_line);

        let line_height = 32;
        let total_height = lines.len() as i32 * line_height;
        let mut start_y = (240 - total_height) / 2;

        for line in lines {
            let w = self.renderer.calculate_width(&line, font_size);
            let x = 200 - (w / 2);
            self.renderer.draw_text_colored(display, &line, x, start_y + 24, font_size, Pixel::Black, ctx);
            start_y += line_height;
        }
    }
}

impl Page for LearnPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Char(' ') => {
                if self.state == LearnState::Question {
                    self.state = LearnState::Answer;
                }
                Action::None
            }
            Key::Char('1') | Key::Char('2') | Key::Char('3') | Key::Char('4') => {
                if self.state == LearnState::Answer {
                    self.next_card();
                }
                Action::None
            }
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        display.clear(ctx);

        if self.deck.is_empty() {
            let msg = "NO CARDS FOUND";
            let w = self.ui_renderer.calculate_width(msg, 30.0);
            self.ui_renderer.draw_text(display, msg, 200 - (w/2), 120, 30.0, ctx);
            return;
        }

        let card = &self.deck[self.current_index];

        let header = if self.state == LearnState::Question { "QUESTION" } else { "ANSWER" };
        self.ui_renderer.draw_text(display, header, 10, 25, 20.0, ctx);
        
        let progress = format!("{}/{}", self.current_index + 1, self.deck.len());
        let p_width = self.ui_renderer.calculate_width(&progress, 20.0);
        self.ui_renderer.draw_text(display, &progress, 390 - p_width, 25, 20.0, ctx);

        let content = if self.state == LearnState::Question { &card.question } else { &card.answer };
        self.draw_centered_wrapped_text(display, content, ctx);

        let footer = if self.state == LearnState::Question {
            "SPACE: FLIP"
        } else {
            "1-4: RATE CARD"
        };
        let f_width = self.ui_renderer.calculate_width(footer, 18.0);
        self.ui_renderer.draw_text(display, footer, 200 - (f_width / 2), 230, 18.0, ctx);
    }
}