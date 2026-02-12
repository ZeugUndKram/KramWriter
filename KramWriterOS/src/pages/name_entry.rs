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
    footer_variants: [Option<Bitmap>; 3], // _0 (none), _1 (cancel), _2 (save)
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
            footer_index: 1, // Default to "Save" highlight
            error_msg: None,
        }
    }

    fn try_save(&mut self) -> Action {
        if self.input_text.trim().is_empty() {
            self.error_msg = Some("NAME CANNOT BE EMPTY".to_string());
            return Action::None;
        }

        let new_path = self.parent_path.join(&self.input_text);
        if new_path.exists() {
            self.error_msg = Some("NAME ALREADY EXISTS".to_string());
            Action::None
        } else {
            if self.is_folder {
                if fs::create_dir(&new_path).is_ok() {
                    Action::Pop // Return to browser
                } else {
                    self.error_msg = Some("SYSTEM ERROR".to_string());
                    Action::None
                }
            } else {
                Action::None // Handle File creation later
            }
        }
    }
}

impl Page for NameEntryPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match self.focus {
            EntryFocus::TextInput => match key {
                Key::Left => { if self.cursor_pos > 0 { self.cursor_pos -= 1; } Action::None }
                Key::Right => { if self.cursor_pos < self.input_text.len() { self.cursor_pos += 1; } Action::None }
                Key::Backspace => {
                    if self.cursor_pos > 0 {
                        self.input_text.remove(self.cursor_pos - 1);
                        self.cursor_pos -= 1;
                        self.error_msg = None;
                    }
                    Action::None
                }
                Key::Char('\n') | Key::Down => {
                    self.focus = EntryFocus::BottomBar;
                    Action::None
                }
                Key::Char(c) => {
                    self.input_text.insert(self.cursor_pos, c.to_ascii_uppercase());
                    self.cursor_pos += 1;
                    self.error_msg = None;
                    Action::None
                }
                _ => Action::None,
            },
            EntryFocus::BottomBar => match key {
                Key::Up => { self.focus = EntryFocus::TextInput; Action::None }
                Key::Left => { self.footer_index = 0; Action::None }
                Key::Right => { self.footer_index = 1; Action::None }
                Key::Char('\n') => {
                    if self.footer_index == 0 { Action::Pop } else { self.try_save() }
                }
                _ => Action::None,
            }
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // 1. Draw Title
        if let Some(bmp) = &self.title_bmp {
            // Adjust coordinates to center visually
            self.draw_layer(display, bmp, 40, ctx);
        }

        // 2. Draw Input Text (Centered)
        let text_y = 120;
        let text_x = 200 - (self.input_text.len() as i32 * 6); // Rough centering
        self.renderer.draw_text(display, &self.input_text, text_x, text_y, 32.0, ctx);

        // 3. Draw Cursor (if in TextInput mode)
        if self.focus == EntryFocus::TextInput {
            let cursor_offset = (self.cursor_pos as i32 * 12); // Depends on BebasNeue width
            let cx = text_x + cursor_offset;
            for cy in (text_y - 25)..(text_y + 5) {
                display.draw_pixel(cx as usize, cy as usize, Pixel::Black, ctx);
            }
        }

        // 4. Draw Error Message if exists
        if let Some(err) = &self.error_msg {
            self.renderer.draw_text_colored(display, err, 100, 160, 18.0, Pixel::Black, ctx);
        }

        // 5. Draw Footer
        let footer_idx = if self.focus == EntryFocus::TextInput { 0 } else { self.footer_index + 1 };
        if let Some(bmp) = &self.footer_variants[footer_idx] {
            self.draw_layer(display, bmp, 216, ctx);
        }
    }
}

// Helper for drawing full-width bitmaps
impl NameEntryPage {
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