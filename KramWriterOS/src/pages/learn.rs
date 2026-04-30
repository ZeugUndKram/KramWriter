use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use regex::Regex;

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
    answer_revealed: bool, // Track if the answer has been seen for the current card
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
            answer_revealed: false,
            renderer,
            ui_renderer,
        };

        page.load_apkg();
        page
    }

    /// Enhanced HTML cleaner that handles lists, breaks, and non-breaking spaces
    fn clean_html(input: &str) -> String {
        let re_tags = Regex::new(r"<[^>]*>").unwrap();
        
        input
            .replace("<li>", "\n• ") // Convert list items to bullet points
            .replace("</li>", "")
            .replace("<ul>", "\n")
            .replace("</ul>", "\n")
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("&nbsp;", " ")
            .split('\n')
            .map(|line| re_tags.replace_all(line, "").trim().to_string())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn load_apkg(&mut self) {
        let temp_db_path = "/tmp/kram_anki.db";
        
        if !self.path.exists() {
            println!("File does not exist at: {:?}", self.path);
            return;
        }

        let file = File::open(&self.path).expect("Could not open apkg file");
        let mut archive = zip::ZipArchive::new(file).expect("Invalid zip archive");

        let mut db_content = Vec::new();
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            if file.name() == "collection.anki21" || file.name() == "collection.anki2" {
                file.read_to_end(&mut db_content).unwrap();
                break;
            }
        }

        if db_content.is_empty() {
            println!("No Anki database found in zip.");
            return;
        }

        std::fs::write(temp_db_path, db_content).unwrap();

        if let Ok(conn) = rusqlite::Connection::open(temp_db_path) {
            let mut stmt = conn.prepare("SELECT flds FROM notes").unwrap();
            
            let card_iter = stmt.query_map([], |row| {
                let flds: String = row.get(0)?;
                
                let parts: Vec<String> = flds
                    .split('\x1f')
                    .map(|s| Self::clean_html(s))
                    .collect();

                Ok(Flashcard {
                    question: parts.get(0).cloned().unwrap_or_else(|| "Empty Q".to_string()),
                    answer: parts.get(1).cloned().unwrap_or_else(|| "Empty A".to_string()),
                })
            }).unwrap();

            self.deck = card_iter
                .filter_map(|r| r.ok())
                .filter(|card| {
                    let c = card.question.to_lowercase();
                    !c.contains("update to the latest anki") && !c.is_empty()
                })
                .collect();

            println!("Final deck count after filtering: {} cards.", self.deck.len());
        }

        let _ = std::fs::remove_file(temp_db_path);
    }

    fn next_card(&mut self) {
        if !self.deck.is_empty() {
            self.current_index = (self.current_index + 1) % self.deck.len();
            self.state = LearnState::Question;
            self.answer_revealed = false; // Reset flag for new card
        }
    }

    /// Newline-aware drawing logic that respects formatting and paragraphs
    fn draw_centered_wrapped_text(&self, display: &mut SharpDisplay, text: &str, ctx: &Context) {
        let font_size = 24.0;
        let margin = 20;
        let max_width = 400 - (margin * 2);
        let mut lines = Vec::new();

        // Process text paragraph by paragraph to respect explicit newlines
        for paragraph in text.split('\n') {
            let mut current_line = String::new();
            for word in paragraph.split_whitespace() {
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
            if !current_line.is_empty() {
                lines.push(current_line);
            }
        }

        let line_height = 30;
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
                // Unlimited toggling: Space flips back and forth
                if self.state == LearnState::Question {
                    self.state = LearnState::Answer;
                    self.answer_revealed = true; // The answer is now "unlocked"
                } else {
                    self.state = LearnState::Question;
                }
                Action::None
            }
            Key::Char('1') | Key::Char('2') | Key::Char('3') | Key::Char('4') => {
                // 1-4 works on both screens only after the first flip
                if self.answer_revealed {
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

        let footer = if !self.answer_revealed {
            "SPACE: FLIP"
        } else {
            "SPACE: TOGGLE | 1-4: RATE"
        };
        let f_width = self.ui_renderer.calculate_width(footer, 18.0);
        self.ui_renderer.draw_text(display, footer, 200 - (f_width / 2), 230, 18.0, ctx);
    }
}