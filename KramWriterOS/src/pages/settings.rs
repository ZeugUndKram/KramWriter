use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;

// Match the lowercase filenames from your 'ls' output
const SETTINGS_OPTIONS: [&str; 5] = ["timezone", "location", "darkmode", "drive", "keyboard"];

pub struct SettingsPage {
    current_index: usize,
    images: Vec<[Option<Bitmap>; 2]>,
    vertical_spacing: i32, // Variable to control spacing
}

impl SettingsPage {
    pub fn new() -> Self {
        let mut images = Vec::new();
        let vertical_spacing = 15; // Set your desired spacing here

        for option in SETTINGS_OPTIONS.iter() {
            // Updated paths to match the lowercase filenames found in /assets/Settings/
            let path_0 = format!("/home/kramwriter/KramWriter/assets/Settings/{}_0.bmp", option);
            let path_1 = format!("/home/kramwriter/KramWriter/assets/Settings/{}_1.bmp", option);
            
            let img_0 = Bitmap::load(&path_0).ok();
            let img_1 = Bitmap::load(&path_1).ok();

            if img_0.is_none() {
                println!("⚠️ Settings Page: Failed to load {}", path_0);
            }

            images.push([img_0, img_1]);
        }

        Self { 
            current_index: 0, 
            images,
            vertical_spacing 
        }
    }
}

impl Page for SettingsPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Up => {
                if self.current_index > 0 { self.current_index -= 1; }
                Action::None
            }
            Key::Down => {
                if self.current_index < SETTINGS_OPTIONS.len() - 1 { self.current_index += 1; }
                Action::None
            }
            Key::Esc => Action::Pop, // Return to menu
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        let mut current_y = 20; // Starting top margin

        for (i, variants) in self.images.iter().enumerate() {
            // variant 1 is selected (_1.bmp), variant 0 is unselected (_0.bmp)
            let selection_index = if i == self.current_index { 1 } else { 0 };

            if let Some(bmp) = &variants[selection_index] {
                let start_x = (400usize.saturating_sub(bmp.width)) / 2;
                
                for y in 0..bmp.height {
                    let screen_y = current_y + y as i32;
                    if screen_y >= 0 && screen_y < 240 {
                        for x in 0..bmp.width {
                            let pixel = bmp.pixels[y * bmp.width + x];
                            display.draw_pixel(start_x + x, screen_y as usize, pixel, ctx);
                        }
                    }
                }
                // Add the image height plus our custom spacing variable
                current_y += bmp.height as i32 + self.vertical_spacing;
            }
        }
    }
}