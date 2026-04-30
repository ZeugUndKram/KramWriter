use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;

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
}

impl LearnCreatePage {
    pub fn new(path: PathBuf) -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/Inter_28pt-Medium.ttf");
        let ui_renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");

        // Start with one empty card
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
        }
    }

    fn current_card_mut(&mut self) -> &mut CardEditor {
        &mut self.cards[self.current_index]
    }

    fn delete_current_card(&mut self) {
        if self.cards.len() > 1 {
            self.cards.remove(self.current_index);
            if self.current_index >= self.cards.len() {
                self.current_index = self.cards.len() - 1;
            }
        } else {
            // If it's the last card, just clear it
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

        // Draw the text
        self.renderer.draw_text_colored(display, text, x, y, font_size, Pixel::Black, ctx);

        // Draw cursor
        let cursor_x = x + self.renderer.calculate_width(&text[0..cursor_pos], font_size) as i32;
        for cy in (y - 22)..(y + 4) {
            display.draw_pixel(cursor_x as usize, cy as usize, Pixel::Black, ctx);
            display.draw_pixel((cursor_x + 1) as usize, cy as usize, Pixel::Black, ctx);
        }
    }
}

impl Page for LearnCreatePage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            // NAVIGATION: CTRL + ARROWS / SPACE / BACKSPACE
            Key::Ctrl('f') | Key::Ctrl('r') => { // Termion represents Ctrl+Right as 'f' often, but we handle explicit arrow logic
                Action::None
            }
            
            // Note: Many terminals send specific codes for Ctrl+Arrows. 
            // If your keyboard driver maps them to specific chars, adjust here.
            // Using standard key combos for deck management:
            
            // TOGGLE SIDE: Ctrl + Space
            Key::Ctrl(' ') => {
                self.side = if self.side == EditSide::Front { EditSide::Back } else { EditSide::Front };
                Action::None
            }

            // PREVIOUS CARD: Ctrl + Left
            Key::Ctrl('b') => { // Standard terminal 'Backward'
                if self.current_index > 0 {
                    self.current_index -= 1;
                }
                Action::None
            }

            // NEXT CARD / NEW CARD: Ctrl + Right
            Key::Ctrl('f') => { // Standard terminal 'Forward'
                if self.current_index < self.cards.len() - 1 {
                    self.current_index += 1;
                } else {
                    self.add_new_card();
                }
                Action::None
            }

            // DELETE CARD: Ctrl + Backspace (often sent as Ctrl+h or Ctrl+w)
            Key::Ctrl('h') | Key::Ctrl('\x7f') => {
                self.delete_current_card();
                Action::None
            }

            // TEXT EDITING: Left / Right Arrows
            Key::Left => {
                let card = self.current_card_mut();
                if self.side == EditSide::Front && card.front_cursor > 0 {
                    card.front_cursor -= 1;
                } else if self.side == EditSide::Back && card.back_cursor > 0 {
                    card.back_cursor -= 1;
                }
                Action::None
            }
            Key::Right => {
                let card = self.current_card_mut();
                if self.side == EditSide::Front && card.front_cursor < card.front.len() {
                    card.front_cursor += 1;
                } else if self.side == EditSide::Back && card.back_cursor < card.back.len() {
                    card.back_cursor += 1;
                }
                Action::None
            }

            // TYPING
            Key::Char(c) => {
                let side = self.side == EditSide::Front;
                let card = self.current_card_mut();
                if side {
                    card.front.insert(card.front_cursor, c);
                    card.front_cursor += 1;
                } else {
                    card.back.insert(card.back_cursor, c);
                    card.back_cursor += 1;
                }
                Action::None
            }
            Key::Backspace => {
                let side = self.side == EditSide::Front;
                let card = self.current_card_mut();
                if side && card.front_cursor > 0 {
                    card.front.remove(card.front_cursor - 1);
                    card.front_cursor -= 1;
                } else if !side && card.back_cursor > 0 {
                    card.back.remove(card.back_cursor - 1);
                    card.back_cursor -= 1;
                }
                Action::None
            }

            Key::Esc => Action::Pop, // We should implement a "Save" action here eventually
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

        // Editor
        if self.side == EditSide::Front {
            self.draw_editor_text(display, &card.front, card.front_cursor, ctx);
        } else {
            self.draw_editor_text(display, &card.back, card.back_cursor, ctx);
        }

        // Footer Instructions
        let footer = "CTRL+SPACE: FLIP | CTRL+L/R: NAV | CTRL+BKSP: DEL";
        let f_w = self.ui_renderer.calculate_width(footer, 16.0);
        self.ui_renderer.draw_text(display, footer, 200 - (f_w / 2), 230, 16.0, ctx);
    }
}