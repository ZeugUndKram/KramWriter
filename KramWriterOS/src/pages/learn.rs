use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

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
    renderer: FontRenderer,    // Using Inter for Q&A
    ui_renderer: FontRenderer, // Using Bebas for headers
}

impl LearnPage {
    pub fn new(path: PathBuf) -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/Inter-Regular.ttf");
        let ui_renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");

        let mut page = Self {
            path: path.clone(),
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
        let file = File::open(&self.path).ok();
        if let Some(f) = file {
            if let Ok(mut archive) = zip::ZipArchive::new(f) {
                // Anki 2.1+ uses collection.anki21, older uses collection.anki2
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    if file.name().contains("collection.anki2") {
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).unwrap();
                        
                        // Load SQLite from memory buffer
                        if let Ok(conn) = rusqlite::Connection::open_in_memory() {
                            let _ = conn.execute_batch("ATTACH DATABASE ':memory:' AS temp_db;");
                            // This is a simplified extraction of the 'notes' table
                            if let Ok(mut stmt) = conn.prepare("SELECT flds FROM notes") {
                                let rows = stmt.query_map([], |row| {
                                    let flds: String = row.get(0)?;
                                    // Anki fields are separated by \x1f (Unit Separator)
                                    let parts: Vec<&str> = flds.split('\x1f').collect();
                                    Ok(Flashcard {
                                        question: parts.get(0).unwrap_or(&"Empty").to_string(),
                                        answer: parts.get(1).unwrap_or(&"Empty").to_string(),
                                    })
                                }).ok();

                                if let Some(flashcards) = rows {
                                    self.deck = flashcards.flatten().collect();
                                }
                            }
                        }
                        break;
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
            // Finished deck
        }
    }

    fn draw_centered_wrapped_text(&self, display: &mut SharpDisplay, text: &str, ctx: &Context) {
        let font_size = 24.0;
        let margin = 20;
        let max_width = 400 - (margin * 2);
        
        // Simplified wrapping logic based on your editor.rs
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
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
            // Difficulty selection 1-4
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
            self.ui_renderer.draw_text(display, "NO CARDS FOUND", 100, 120, 30.0, ctx);
            return;
        }

        let card = &self.deck[self.current_index];

        // Draw Header
        let header = if self.state == LearnState::Question { "QUESTION" } else { "ANSWER" };
        self.ui_renderer.draw_text(display, header, 10, 25, 20.0, ctx);
        
        // Draw Progress
        let progress = format!("{}/{}", self.current_index + 1, self.deck.len());
        let p_width = self.ui_renderer.calculate_width(&progress, 20.0);
        self.ui_renderer.draw_text(display, &progress, 390 - p_width, 25, 20.0, ctx);

        // Draw Content
        let content = if self.state == LearnState::Question { &card.question } else { &card.answer };
        self.draw_centered_wrapped_text(display, content, ctx);

        // Footer UI hint
        let footer = if self.state == LearnState::Question {
            "PRESS SPACE TO FLIP"
        } else {
            "1: AGAIN  2: HARD  3: GOOD  4: EASY"
        };
        let f_width = self.ui_renderer.calculate_width(footer, 18.0);
        self.ui_renderer.draw_text(display, footer, 200 - (f_width / 2), 230, 18.0, ctx);
    }
}