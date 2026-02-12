use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::fs;
use std::path::PathBuf;

#[derive(PartialEq)]
enum EntryFocus {
    TextInput,
    BottomBar,
}

pub struct NameEntryPage {
    title_bmp: Option<Bitmap>,
    footer_variants: [Option<Bitmap>; 3], // 0: none, 1: cancel, 2: save
    renderer: FontRenderer,
    parent_path: PathBuf,
    is_folder: bool,
    input_text: String,
    cursor_pos: usize,
    focus: EntryFocus,
    footer_index: usize, // 0: Cancel, 1: Save
    error_msg: Option<String>,
}

impl NameEntryPage {
    pub fn new(parent_path: PathBuf, is_folder: bool) -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        let asset_path = "/home/kramwriter/KramWriter/assets/NameEntry";
        
        Self {
            title_bmp: Bitmap::load(&format!("{}/title.bmp", asset_path)).ok(),
            footer_variants: [
                Bitmap::load(&format!("{}/bottom_bar_0.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/bottom_bar_1.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/bottom_bar_2.bmp", asset_path)).ok(),
            ],
            renderer,
            parent_path,
            is_folder,
            input_text: String::new(),
            cursor_pos: 0,
            focus: EntryFocus::TextInput,
            footer_index: 1, // Default highlight on SAVE
            error_msg: None,
        }
    }

    fn try_save(&mut self) -> Action {
        let name = self.input_text.trim();
        if name.is_empty() {
            self.error_msg = Some("NAME CANNOT BE EMPTY".to_string());
            return Action::None;
        }

        let new_path = self.parent_path.join(name);
        if new_path.exists() {
            self.error_msg = Some("NAME ALREADY EXISTS".to_string());
            Action::None
        } else {
            let success = if self.is_folder {
                fs::create_dir(&new_path).is_ok()
            } else {
                fs::File::create(&new_path).is_ok()
            };

            if success {
                Action::Pop 
            } else {
                self.error_msg = Some("SYSTEM ERROR".to_string());
                Action::None
            }
        }
    }

    fn draw_layer(&self, display: &mut SharpDisplay, bmp: &Bitmap, y_offset: i32, ctx: &Context) {
        for y in 0..bmp.height {
            let sy = y as i32 + y_offset;
            if sy >= 0 && sy < 240 {
                for x in 0..bmp.width.min(400) {
                    if bmp.pixels[y * bmp.width + x] == Pixel::Black {
                        display.draw_pixel(x, sy as usize, Pixel::Black, ctx);
                    }
                }
            }
        }
    }
}

impl Page for NameEntryPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match self.focus {
            EntryFocus::TextInput => match key {
                Key::Left => {
                    if self.cursor_pos > 0 { self.cursor_pos -= 1; }
                    Action::None
                }
                Key::Right => {
                    if self.cursor_pos < self.input_text.len() { self.cursor_pos += 1; }
                    Action::None
                }
                Key::Backspace => {
                    if self.cursor_pos > 0 {
                        self.input_text.remove(self.cursor_pos - 1);
                        self.cursor_pos -= 1;
                        self.error_msg = None;
                    }
                    Action::None
                }
                Key::Down | Key::Char('\n') => {
                    self.focus = EntryFocus::BottomBar;
                    Action::None
                }
                Key::Char(c) => {
                    // Filter for valid filename characters
                    if c.is_alphanumeric() || c == '_' || c == '-' || c == ' ' {
                        self.input_text.insert(self.cursor_pos, c.to_ascii_uppercase());
                        self.cursor_pos += 1;
                        self.error_msg = None;
                    }
                    Action::None
                }
                _ => Action::None,
            },
            EntryFocus::BottomBar => match key {
                Key::Up => {
                    self.focus = EntryFocus::TextInput;
                    Action::None
                }
                Key::Left => {
                    self.footer_index = 0; // Cancel
                    Action::None
                }
                Key::Right => {
                    self.footer_index = 1; // Save
                    Action::None
                }
                Key::Char('\n') => {
                    if self.footer_index == 0 { Action::Pop } else { self.try_save() }
                }
                _ => Action::None,
            }
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // 1. Title (title.bmp)
        if let Some(bmp) = &self.title_bmp {
            self.draw_layer(display, bmp, 40, ctx);
        }

        // 2. Input Field
        let text_y = 110;
        let char_width = 14; // Bebas Neue approximate width at size 32
        let total_w = self.input_text.len() as i32 * char_width;
        let start_x = 200 - (total_w / 2); // Center text
        
        self.renderer.draw_text(display, &self.input_text, start_x, text_y, 32.0, ctx);

        // 3. Cursor
        if self.focus == EntryFocus::TextInput {
            let cursor_x = start_x + (self.cursor_pos as i32 * char_width);
            // Draw a 2px wide cursor
            for cy in (text_y - 26)..(text_y + 4) {
                if cy > 0 && cy < 240 {
                    display.draw_pixel(cursor_x as usize, cy as usize, Pixel::Black, ctx);
                    display.draw_pixel((cursor_x + 1) as usize, cy as usize, Pixel::Black, ctx);
                }
            }
        }

        // 4. Error Message
        if let Some(err) = &self.error_msg {
            let err_w = err.len() as i32 * 8;
            self.renderer.draw_text(display, err, 200 - (err_w / 2), 160, 18.0, ctx);
        }

        // 5. Footer (bottom_bar_0, 1, 2)
        let footer_idx = if self.focus == EntryFocus::TextInput { 0 } else { self.footer_index + 1 };
        if let Some(bmp) = &self.footer_variants[footer_idx] {
            self.draw_layer(display, bmp, 216, ctx);
        }
    }
}