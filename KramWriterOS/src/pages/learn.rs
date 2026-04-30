use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
// Removed unused Bitmap import to clear the warning
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

// Add these imports at the top
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

    fn load_apkg(&mut self) {
    let temp_db_path = "/tmp/kram_anki.db";

    if let Ok(file) = File::open(&self.path) {
        if let Ok(mut archive) = zip::ZipArchive::new(file) {
            // First, get a list of all files in the zip to find the best database
            let mut target_entry_name = None;
            
            // Prioritize collection.anki21 (Modern Anki) over collection.anki2 (Legacy)
            for i in 0..archive.len() {
                if let Ok(file) = archive.by_index(i) {
                    let name = file.name();
                    if name == "collection.anki21" {
                        target_entry_name = Some(name.to_string());
                        break; 
                    } else if name == "collection.anki2" && target_entry_name.is_none() {
                        target_entry_name = Some(name.to_string());
                    }
                }
            }

            if let Some(entry_name) = target_entry_name {
                if let Ok(mut archive_file) = archive.by_name(&entry_name) {
                    let mut buffer = Vec::new();
                    if archive_file.read_to_end(&mut buffer).is_ok() {
                        // Write to temp file for rusqlite to open
                        if std::fs::write(temp_db_path, buffer).is_ok() {
                            if let Ok(conn) = rusqlite::Connection::open(temp_db_path) {
                                // Try to get notes
                                let mut stmt_result = conn.prepare("SELECT flds FROM notes");
                                
                                if let Ok(mut stmt) = stmt_result {
                                    let rows = stmt.query_map([], |row| {
                                        let flds: String = row.get(0)?;
                                        // Anki uses the unit separator character \x1f to split fields
                                        let parts: Vec<&str> = flds.split('\x1f').collect();
                                        Ok(Flashcard {
                                            question: parts.get(0).unwrap_or(&"Empty").to_string(),
                                            answer: parts.get(1).unwrap_or(&"Empty").to_string(),
                                        })
                                    });

                                    if let Ok(flashcard_iter) = rows {
                                        self.deck = flashcard_iter.filter_map(|r| r.ok()).collect();
                                    }
                                }
                            }
                            let _ = std::fs::remove_file(temp_db_path);
                        }
                    }
                }
            }
        }
    }
}

    fn next_card(&mut self) {
        if self.current_index < self.deck.len() - 1 {
            self.current_index += 1;
            self.state = LearnState::Question;
        } else {
            // Option: Loop back or Action::Pop
            self.current_index = 0;
            self.state = LearnState::Question;
        }
    }

    fn draw_centered_wrapped_text(&self, display: &mut SharpDisplay, text: &str, ctx: &Context) {
        let font_size = 24.0;
        let margin = 20;
        let max_width = 400 - (margin * 2);
        
        let mut lines = Vec::new();
        let mut current_line = String::new();

        // Clean up basic Anki HTML tags (like <br> or <div>) for the terminal/display
        let clean_text = text.replace("<br>", " ").replace("<div>", " ").replace("</div>", "");

        for word in clean_text.split_whitespace() {
            let test_str = if current_line.is_empty() { word.to_string() } else { format!("{} {}", current_line, word) };
            if self.renderer.calculate_width(&test_str, font_size) as i32 > max_width {
                lines.push(current_line);
                current_line = word.to_string();
            } else {
                current_line = test_str;
            }
        }
        lines.push(current_line);

        let total_height = lines.len() as i32 * 28;
        let mut start_y = (240 - total_height) / 2;

        for line in lines {
            let w = self.renderer.calculate_width(&line, font_size);
            let x = 200 - (w / 2);
            self.renderer.draw_text_colored(display, &line, x, start_y + 24, font_size, Pixel::Black, ctx);
            start_y += 28;
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

        // Header
        let header = if self.state == LearnState::Question { "QUESTION" } else { "ANSWER" };
        self.ui_renderer.draw_text(display, header, 10, 25, 20.0, ctx);
        
        let progress = format!("{}/{}", self.current_index + 1, self.deck.len());
        let p_width = self.ui_renderer.calculate_width(&progress, 20.0);
        self.ui_renderer.draw_text(display, &progress, 390 - p_width, 25, 20.0, ctx);

        // Content
        let content = if self.state == LearnState::Question { &card.question } else { &card.answer };
        self.draw_centered_wrapped_text(display, content, ctx);

        // Footer
        let footer = if self.state == LearnState::Question {
            "SPACE: FLIP"
        } else {
            "1: AGAIN  2: HARD  3: GOOD  4: EASY"
        };
        let f_width = self.ui_renderer.calculate_width(footer, 18.0);
        self.ui_renderer.draw_text(display, footer, 200 - (f_width / 2), 230, 18.0, ctx);
    }
}