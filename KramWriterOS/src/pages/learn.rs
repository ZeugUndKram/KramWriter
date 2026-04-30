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
        let renderer = FontRenderer::new("fonts/Inter-Regular.ttf");
        let ui_renderer = FontRenderer::new("fonts/BebasNeue-Regular.ttf");

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
        if let Ok(file) = File::open(&self.path) {
            // Explicitly type the archive
            if let Ok(mut archive) = ZipArchive::new(file) {
                for i in 0..archive.len() {
                    let mut archive_file = archive.by_index(i).unwrap();
                    
                    // Anki 2.1+ uses collection.anki21, older uses collection.anki2
                    if archive_file.name().contains("collection.anki2") {
                        let mut buffer = Vec::new();
                        let _ = archive_file.read_to_end(&mut buffer);
                        
                        // Open connection from the buffer
                        if let Ok(conn) = Connection::open_in_memory() {
                            // Extract notes - Anki notes are usually HTML/Text in the 'flds' column
                            let mut stmt = conn.prepare("SELECT flds FROM notes").unwrap();
                            let rows = stmt.query_map([], |row| {
                                let flds: String = row.get(0)?;
                                let parts: Vec<&str> = flds.split('\x1f').collect();
                                Ok(Flashcard {
                                    question: parts.get(0).unwrap_or(&"Empty").to_string(),
                                    answer: parts.get(1).unwrap_or(&"Empty").to_string(),
                                })
                            }).unwrap();

                            // Explicitly collect into the deck
                            self.deck = rows.filter_map(|r| r.ok()).collect::<Vec<Flashcard>>();
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