use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(PartialEq, Clone, Copy)] 
enum SetupStep {
    Email,
    Password,
    ReadyToSync,
    Syncing,
}

#[derive(PartialEq, Clone, Copy)]
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
    status_msg: Option<String>, // Added for informational feedback
}

impl SimpleNoteSetupPage {
    pub fn new() -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        let asset_path = "/home/kramwriter/KramWriter/assets/NameEntry"; 
        
        let creds_path = "/home/kramwriter/.simplenote_creds";
        let initial_step = if Path::new(creds_path).exists() {
            SetupStep::ReadyToSync
        } else {
            SetupStep::Email
        };

        Self {
            renderer,
            footer_variants: [
                Bitmap::load(&format!("{}/bottom_bar_0.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/bottom_bar_1.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/bottom_bar_2.bmp", asset_path)).ok(),
            ],
            step: initial_step,
            email: String::new(),
            password: String::new(),
            cursor_pos: 0,
            focus: if initial_step == SetupStep::ReadyToSync { EntryFocus::BottomBar } else { EntryFocus::TextInput },
            footer_index: 1,
            error_msg: None,
            status_msg: None,
        }
    }

    fn handle_submit(&mut self, _ctx: &mut Context) -> Action {
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
                
                let creds = format!("{}\n{}", self.email, self.password);
                if fs::write("/home/kramwriter/.simplenote_creds", creds).is_ok() {
                    self.step = SetupStep::ReadyToSync;
                    self.focus = EntryFocus::BottomBar;
                    self.footer_index = 1;
                    self.error_msg = None;
                    self.status_msg = Some("LOGIN SAVED".to_string());
                } else {
                    self.error_msg = Some("SYS ERROR: COULD NOT SAVE CREDS".to_string());
                }
                Action::None
            }
            SetupStep::ReadyToSync => {
                self.step = SetupStep::Syncing;
                self.status_msg = None;
                self.error_msg = None;

                // NOTE: output() is blocking. The screen will stay on "SYNCING..." 
                // until the Python script completes.
                let output = Command::new("python3")
                    .arg("/home/kramwriter/KramWriter/KramWriterOS/scripts/sync_notes.py")
                    .output();

                match output {
                    Ok(out) => {
                        if out.status.success() {
                            let stdout = String::from_utf8_lossy(&out.stdout);
                            
                            // Parse the output to count actions
                            let downloads = stdout.lines().filter(|l| l.contains("Downloaded")).count();
                            let uploads = stdout.lines().filter(|l| l.contains("Uploaded")).count();
                            
                            self.step = SetupStep::ReadyToSync;
                            if downloads == 0 && uploads == 0 {
                                self.status_msg = Some("SYNC COMPLETE: NO CHANGES".to_string());
                            } else {
                                self.status_msg = Some(format!("SUCCESS: {} DOWN, {} UP", downloads, uploads));
                            }
                            Action::None 
                        } else {
                            self.step = SetupStep::ReadyToSync;
                            let err_text = String::from_utf8_lossy(&out.stderr);
                            eprintln!("Python Sync Error: {}", err_text);
                            self.error_msg = Some("SYNC FAILED. CHECK WIFI.".to_string());
                            Action::None
                        }
                    }
                    Err(e) => {
                        self.step = SetupStep::ReadyToSync;
                        eprintln!("Failed to execute process: {}", e);
                        self.error_msg = Some("PYTHON SCRIPT MISSING".to_string());
                        Action::None
                    }
                }
            }
            SetupStep::Syncing => Action::None,
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
                        self.status_msg = None;
                    }
                    Action::None
                }
                Key::Down | Key::Char('\n') => { self.focus = EntryFocus::BottomBar; Action::None }
                Key::Char(c) => {
                    if self.step == SetupStep::ReadyToSync || self.step == SetupStep::Syncing {
                        return Action::None;
                    }
                    let target = if self.step == SetupStep::Email { &mut self.email } else { &mut self.password };
                    if target.len() < 40 {
                        target.insert(self.cursor_pos, c); 
                        self.cursor_pos += 1;
                        self.error_msg = None;
                        self.status_msg = None;
                    }
                    Action::None
                }
                _ => Action::None,
            },
            EntryFocus::BottomBar => match key {
                Key::Up => { 
                    if self.step != SetupStep::ReadyToSync && self.step != SetupStep::Syncing {
                        self.focus = EntryFocus::TextInput; 
                    }
                    Action::None 
                }
                Key::Left => { self.footer_index = 0; Action::None }
                Key::Right => { self.footer_index = 1; Action::None }
                Key::Char('\n') => {
                    if self.footer_index == 0 { 
                        Action::Pop 
                    } else { 
                        self.handle_submit(ctx) 
                    }
                }
                _ => Action::None,
            }
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // 1. Title
        let title_text = match self.step {
            SetupStep::Email => "ENTER SIMPLENOTE EMAIL",
            SetupStep::Password => "ENTER PASSWORD",
            SetupStep::ReadyToSync => "SIMPLENOTE HUB",
            SetupStep::Syncing => "SYNCING NOTES...",
        };
        let title_w = self.renderer.calculate_width(title_text, 24.0);
        self.renderer.draw_text(display, title_text, 200 - (title_w / 2), 60, 24.0, ctx);

        // 2. Main Content
        let font_size = 28.0;
        let display_text = match self.step {
            SetupStep::Email => self.email.to_uppercase(),
            SetupStep::Password => "*".repeat(self.password.len()),
            SetupStep::ReadyToSync => {
                if self.status_msg.is_some() { "READY TO SYNC AGAIN".to_string() }
                else { "PRESS ENTER TO SYNC".to_string() }
            },
            SetupStep::Syncing => "PLEASE WAIT...".to_string(),
        };

        let full_width = self.renderer.calculate_width(&display_text, font_size);
        let start_x = 200 - (full_width / 2);
        let text_y = 120;
        self.renderer.draw_text(display, &display_text, start_x, text_y, font_size, ctx);

        // 3. Cursor
        if self.focus == EntryFocus::TextInput && (self.step == SetupStep::Email || self.step == SetupStep::Password) {
            let substring = if self.step == SetupStep::Email { &self.email[0..self.cursor_pos] } else { &display_text[0..self.cursor_pos] };
            let sub_width = self.renderer.calculate_width(substring, font_size);
            let cursor_x = start_x + sub_width; 

            for cy in (text_y - 24)..(text_y + 2) {
                display.draw_pixel(cursor_x as usize, cy as usize, Pixel::Black, ctx);
                display.draw_pixel((cursor_x + 1) as usize, cy as usize, Pixel::Black, ctx);
            }
        }

        // 4. Messages (Error in Black, Status in Black/Centered)
        if let Some(err) = &self.error_msg {
            let err_w = self.renderer.calculate_width(err, 20.0);
            self.renderer.draw_text(display, err, 200 - (err_w / 2), 165, 20.0, ctx);
        } else if let Some(status) = &self.status_msg {
            let status_w = self.renderer.calculate_width(status, 20.0);
            self.renderer.draw_text(display, status, 200 - (status_w / 2), 165, 20.0, ctx);
        }

        // 5. Footer
        if self.step != SetupStep::Syncing {
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
}