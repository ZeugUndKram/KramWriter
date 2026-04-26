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
    title_folder_bmp: Option<Bitmap>,
    title_file_bmp: Option<Bitmap>,
    footer_variants: [Option<Bitmap>; 3],
    renderer: FontRenderer,
    parent_path: PathBuf,
    is_folder: bool,
    input_text: String,
    cursor_pos: usize,
    focus: EntryFocus,
    footer_index: usize,
    error_msg: Option<String>,
}

impl NameEntryPage {
    pub fn new(parent_path: PathBuf, is_folder: bool) -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        let asset_path = "/home/kramwriter/KramWriter/assets/NameEntry";
        
        Self {
            title_folder_bmp: Bitmap::load(&format!("{}/title.bmp", asset_path)).ok(),
            title_file_bmp: Bitmap::load(&format!("{}/title_file.bmp", asset_path)).ok(),
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
            footer_index: 1, 
            error_msg: None,
        }
    }

    fn try_save(&mut self) -> Action {
        let mut final_name = self.input_text.trim().to_string();
        if final_name.is_empty() {
            self.error_msg = Some("NAME CANNOT BE EMPTY".to_string());
            return Action::None;
        }

        // Add extension if it's a file
        if !self.is_folder {
            final_name.push_str(".txt");
        }

        let new_path = self.parent_path.join(&final_name);
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
                if !self.is_folder {
                    // Placeholder for when we build the EditorPage
                    // Action::Push(Box::new(EditorPage::new(new_path)))
                    Action::Pop
                } else {
                    Action::Pop
                }
            } else {
                self.error_msg = Some("SYSTEM ERROR".to_string());
                Action::None
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
                Key::Down | Key::Char('\n') => { self.focus = EntryFocus::BottomBar; Action::None }
                Key::Char(c) => {
                    // Limit length so it doesn't run off screen
                    if self.input_text.len() < 20 {
                        self.input_text.insert(self.cursor_pos, c.to_ascii_uppercase());
                        self.cursor_pos += 1;
                        self.error_msg = None;
                    }
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
        // 1. Draw correct Title
        let title = if self.is_folder { &self.title_folder_bmp } else { &self.title_file_bmp };
        if let Some(bmp) = title {
            let x_off = (400 - bmp.width as i32) / 2;
            for y in 0..bmp.height {
                for x in 0..bmp.width {
                    if bmp.pixels[y * bmp.width + x] == Pixel::Black {
                        display.draw_pixel((x as i32 + x_off) as usize, (y + 40) as usize, Pixel::Black, ctx);
                    }
                }
            }
        }

        // 2. Prepare Display String
        let font_size = 32.0;
        let display_text = if self.is_folder { 
            self.input_text.clone() 
        } else { 
            format!("{}.TXT", self.input_text) 
        };

        let full_width = self.renderer.calculate_width(&display_text, font_size);
        let start_x = 200 - (full_width / 2);
        let text_y = 120;
        
        self.renderer.draw_text(display, &display_text, start_x, text_y, font_size, ctx);

        // 3. Draw Cursor (Positioned only within input_text part)
        if self.focus == EntryFocus::TextInput {
            let substring = &self.input_text[0..self.cursor_pos];
            let sub_width = self.renderer.calculate_width(substring, font_size);
            let cursor_x = start_x + sub_width; 

            for cy in (text_y - 28)..(text_y + 2) {
                if cy > 0 && cy < 240 {
                    display.draw_pixel(cursor_x as usize, cy as usize, Pixel::Black, ctx);
                    display.draw_pixel((cursor_x + 1) as usize, cy as usize, Pixel::Black, ctx);
                }
            }
        }

        // 4. Error Message
        if let Some(err) = &self.error_msg {
            let err_w = self.renderer.calculate_width(err, 20.0);
            self.renderer.draw_text(display, err, 200 - (err_w / 2), 165, 20.0, ctx);
        }

        // 5. Footer
        let footer_idx = if self.focus == EntryFocus::TextInput { 0 } else { self.footer_index + 1 };
        if let Some(bmp) = &self.footer_variants[footer_idx] {
            for y in 0..bmp.height {
                for x in 0..bmp.width {
                    if bmp.pixels[y * bmp.width + x] == Pixel::Black {
                        display.draw_pixel(x, (y + 216) as usize, Pixel::Black, ctx);
                    }
                }
            }
        }
    }
}