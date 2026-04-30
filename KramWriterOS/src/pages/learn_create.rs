use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

#[derive(PartialEq)]
enum EditSide {
    Front,
    Back,
}

struct CardEditor {
    front: String,
    back: String,
    front_cursor: usize,
    back_cursor: usize,
}

pub struct LearnCreatePage {
    path: PathBuf,
    cards: Vec<CardEditor>,
    current_index: usize,
    side: EditSide,
    renderer: FontRenderer,
    ui_renderer: FontRenderer,
    message: Option<(String, std::time::Instant)>,
}

impl LearnCreatePage {
    pub fn new(path: PathBuf) -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/Inter_28pt-Medium.ttf");
        let ui_renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");

        let first_card = CardEditor {
            front: String::new(),
            back: String::new(),
            front_cursor: 0,
            back_cursor: 0,
        };

        Self {
            path,
            cards: vec![first_card],
            current_index: 0,
            side: EditSide::Front,
            renderer,
            ui_renderer,
            message: None,
        }
    }

    fn current_card_mut(&mut self) -> &mut CardEditor {
        &mut self.cards[self.current_index]
    }

    fn save_to_file(&mut self) {
        let mut content = String::new();
        for card in &self.cards {
            // Only save cards that aren't completely empty
            if !card.front.trim().is_empty() || !card.back.trim().is_empty() {
                content.push_str(&format!("Q: {}\nA: {}\n---\n", card.front.trim(), card.back.trim()));
            }
        }

        if let Ok(mut file) = File::create(&self.path) {
            if file.write_all(content.as_bytes()).is_ok() {
                self.message = Some(("SAVED SUCCESSFULLY!".to_string(), std::time::Instant::now()));
            }
        }
    }

    fn delete_current_card(&mut self) {
        if self.cards.len() > 1 {
            self.cards.remove(self.current_index);
            if self.current_index >= self.cards.len() {
                self.current_index = self.cards.len() - 1;
            }
        } else {
            let card = self.current_card_mut();
            card.front.clear();
            card.back.clear();
            card.front_cursor = 0;
            card.back_cursor = 0;
        }
    }

    fn add_new_card(&mut self) {
        let new_card = CardEditor {
            front: String::new(),
            back: String::new(),
            front_cursor: 0,
            back_cursor: 0,
        };
        self.cards.push(new_card);
        self.current_index = self.cards.len() - 1;
        self.side = EditSide::Front;
    }

    fn draw_editor_text(&self, display: &mut SharpDisplay, text: &str, cursor_pos: usize, ctx: &Context) {
        let font_size = 24.0;
        let x = 20;
        let y = 120;
        self.renderer.draw_text_colored(display, text, x, y, font_size, Pixel::Black, ctx);

        let cursor_x = x + self.renderer.calculate_width(&text[0..cursor_pos.min(text.len())], font_size) as i32;
        for cy in (y - 22)..(y + 4) {
            if cy >= 0 && cy < 240 && cursor_x >= 0 && cursor_x < 400 {
                display.draw_pixel(cursor_x as usize, cy as usize, Pixel::Black, ctx);
                display.draw_pixel((cursor_x + 1) as usize, cy as usize, Pixel::Black, ctx);
            }
        }
    }
}

impl Page for LearnCreatePage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        if let Some((_, time)) = &self.message {
            if time.elapsed().as_secs() >= 2 {
                self.message = None;
            }
        }

        match key {
            // SAVE: Ctrl + S
            Key::Ctrl('s') => {
                self.save_to_file();
                Action::None
            }

            // TOGGLE SIDE: Ctrl + T
            Key::Ctrl('t') => {
                self.side = if self.side == EditSide::Front { EditSide::Back } else { EditSide::Front };
                Action::None
            }

            // PREVIOUS CARD: Ctrl + P
            Key::Ctrl('p') => {
                if self.current_index > 0 {
                    self.current_index -= 1;
                }
                Action::None
            }

            // NEXT CARD: Ctrl + N
            Key::Ctrl('n') => {
                if self.current_index < self.cards.len() - 1 {
                    self.current_index += 1;
                } else {
                    self.add_new_card();
                }
                Action::None
            }

            // DELETE CARD: Ctrl + D
            Key::Ctrl('d') => {
                self.delete_current_card();
                Action::None
            }

            // NAVIGATION
            Key::Left => {
                let is_front = self.side == EditSide::Front;
                let card = self.current_card_mut();
                if is_front && card.front_cursor > 0 {
                    card.front_cursor -= 1;
                } else if !is_front && card.back_cursor > 0 {
                    card.back_cursor -= 1;
                }
                Action::None
            }
            Key::Right => {
                let is_front = self.side == EditSide::Front;
                let card = self.current_card_mut();
                if is_front && card.front_cursor < card.front.len() {
                    card.front_cursor += 1;
                } else if !is_front && card.back_cursor < card.back.len() {
                    card.back_cursor += 1;
                }
                Action::None
            }

            // TYPING
            Key::Char(c) => {
                if c.is_control() { return Action::None; }
                
                let is_front = self.side == EditSide::Front;
                let card = self.current_card_mut();
                if is_front {
                    card.front.insert(card.front_cursor, c);
                    card.front_cursor += 1;
                } else {
                    card.back.insert(card.back_cursor, c);
                    card.back_cursor += 1;
                }
                Action::None
            }
            Key::Backspace => {
                let is_front = self.side == EditSide::Front;
                let card = self.current_card_mut();
                if is_front && card.front_cursor > 0 {
                    card.front.remove(card.front_cursor - 1);
                    card.front_cursor -= 1;
                } else if !is_front && card.back_cursor > 0 {
                    card.back.remove(card.back_cursor - 1);
                    card.back_cursor -= 1;
                }
                Action::None
            }

            Key::Esc => Action::Pop, 
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        display.clear(ctx);

        let card = &self.cards[self.current_index];
        
        // Header
        let side_text = if self.side == EditSide::Front { "EDIT FRONT" } else { "EDIT BACK" };
        self.ui_renderer.draw_text(display, side_text, 10, 25, 20.0, ctx);

        let progress = format!("CARD {}/{}", self.current_index + 1, self.cards.len());
        let p_w = self.ui_renderer.calculate_width(&progress, 20.0);
        self.ui_renderer.draw_text(display, &progress, 390 - p_w, 25, 20.0, ctx);

        // Save Confirmation Message
        if let Some((msg, _)) = &self.message {
            let m_w = self.ui_renderer.calculate_width(msg, 20.0);
            self.ui_renderer.draw_text(display, msg, 200 - (m_w / 2), 50, 20.0, ctx);
        }

        // Editor
        if self.side == EditSide::Front {
            self.draw_editor_text(display, &card.front, card.front_cursor, ctx);
        } else {
            self.draw_editor_text(display, &card.back, card.back_cursor, ctx);
        }

        // Footer
        let footer = "CTRL+T: FLIP | CTRL+P/N: NAV | CTRL+S: SAVE | CTRL+D: DEL";
        let f_w = self.ui_renderer.calculate_width(footer, 15.0);
        self.ui_renderer.draw_text(display, footer, 200 - (f_w / 2), 230, 15.0, ctx);
    }
}