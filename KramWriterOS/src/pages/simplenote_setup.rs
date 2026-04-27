use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;

#[derive(PartialEq)]
enum SetupStep {
    Email,
    Password,
}

#[derive(PartialEq)]
enum EntryFocus {
    TextInput,
    BottomBar,
}

pub struct SimpleNoteSetupPage {
    renderer: FontRenderer,
    footer_variants: [Option<Bitmap>; 3],
    step: SetupStep,
    email: String,
    password: String,
    cursor_pos: usize,
    focus: EntryFocus,
    footer_index: usize,
    error_msg: Option<String>,
}

impl SimpleNoteSetupPage {
    pub fn new() -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        let asset_path = "/home/kramwriter/KramWriter/assets/NameEntry"; // Reusing your existing assets
        
        Self {
            renderer,
            footer_variants: [
                Bitmap::load(&format!("{}/bottom_bar_0.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/bottom_bar_1.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/bottom_bar_2.bmp", asset_path)).ok(),
            ],
            step: SetupStep::Email,
            email: String::new(),
            password: String::new(),
            cursor_pos: 0,
            focus: EntryFocus::TextInput,
            footer_index: 1,
            error_msg: None,
        }
    }

    fn handle_submit(&mut self, ctx: &mut Context) -> Action {
        match self.step {
            SetupStep::Email => {
                if self.email.contains('@') && self.email.contains('.') {
                    self.step = SetupStep::Password;
                    self.cursor_pos = 0;
                    self.focus = EntryFocus::TextInput;
                    Action::None
                } else {
                    self.error_msg = Some("INVALID EMAIL".to_string());
                    Action::None
                }
            }
            SetupStep::Password => {
                if self.password.is_empty() {
                    self.error_msg = Some("PASSWORD REQUIRED".to_string());
                    return Action::None;
                }
                
                // Here we would call the crate::simplenote::login logic
                // For now, let's update context and pop
                ctx.simplenote_email = Some(self.email.clone());
                ctx.simplenote_token = Some("TEMP_TOKEN".to_string()); // Replace with actual API call result
                
                Action::Pop
            }
        }
    }
}

impl Page for SimpleNoteSetupPage {
    fn update(&mut self, key: Key, ctx: &mut Context) -> Action {
        match self.focus {
            EntryFocus::TextInput => match key {
                Key::Left => { if self.cursor_pos > 0 { self.cursor_pos -= 1; } Action::None }
                Key::Right => { 
                    let len = if self.step == SetupStep::Email { self.email.len() } else { self.password.len() };
                    if self.cursor_pos < len { self.cursor_pos += 1; } 
                    Action::None 
                }
                Key::Backspace => {
                    let target = if self.step == SetupStep::Email { &mut self.email } else { &mut self.password };
                    if self.cursor_pos > 0 {
                        target.remove(self.cursor_pos - 1);
                        self.cursor_pos -= 1;
                        self.error_msg = None;
                    }
                    Action::None
                }
                Key::Down | Key::Char('\n') => { self.focus = EntryFocus::BottomBar; Action::None }
                Key::Char(c) => {
                    let target = if self.step == SetupStep::Email { &mut self.email } else { &mut self.password };
                    if target.len() < 30 {
                        // Email stays uppercase for your UI style, or use c.to_lowercase() if Simplenote is picky
                        target.insert(self.cursor_pos, c); 
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
                    if self.footer_index == 0 { Action::Pop } else { self.handle_submit(ctx) }
                }
                _ => Action::None,
            }
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // 1. Draw Title Text
        let title_text = match self.step {
            SetupStep::Email => "ENTER SIMPLENOTE EMAIL",
            SetupStep::Password => "ENTER PASSWORD",
        };
        let title_w = self.renderer.calculate_width(title_text, 24.0);
        self.renderer.draw_text(display, title_text, 200 - (title_w / 2), 60, 24.0, ctx);

        // 2. Prepare Display String
        let font_size = 28.0;
        let display_text = if self.step == SetupStep::Email {
            self.email.to_uppercase()
        } else {
            "*".repeat(self.password.len()) // Mask password
        };

        let full_width = self.renderer.calculate_width(&display_text, font_size);
        let start_x = 200 - (full_width / 2);
        let text_y = 120;
        
        self.renderer.draw_text(display, &display_text, start_x, text_y, font_size, ctx);

        // 3. Draw Cursor
        if self.focus == EntryFocus::TextInput {
            let substring = if self.step == SetupStep::Email {
                &self.email[0..self.cursor_pos]
            } else {
                &display_text[0..self.cursor_pos]
            };
            let sub_width = self.renderer.calculate_width(substring, font_size);
            let cursor_x = start_x + sub_width; 

            for cy in (text_y - 24)..(text_y + 2) {
                display.draw_pixel(cursor_x as usize, cy as usize, Pixel::Black, ctx);
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