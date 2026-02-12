use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;

const SETTINGS_OPTIONS: [&str; 5] = ["timezone", "location", "darkmode", "drive", "keyboard"];

pub struct SettingsPage {
    current_index: usize,
    images: Vec<[Option<Bitmap>; 2]>,
}

impl SettingsPage {
    pub fn new() -> Self {
        let mut images = Vec::new();

        for option in SETTINGS_OPTIONS.iter() {
            let path_0 = format!("/home/kramwriter/KramWriter/assets/Settings/{}_0.bmp", option);
            let path_1 = format!("/home/kramwriter/KramWriter/assets/Settings/{}_1.bmp", option);
            
            let img_0 = Bitmap::load(&path_0).ok();
            let img_1 = Bitmap::load(&path_1).ok();

            // Terminal Debug: Check the size of loaded images
            if let Some(ref b) = img_0 {
                println!("✅ Loaded {}: {}x{}", option, b.width, b.height);
            } else {
                println!("❌ Failed to load: {}", path_0);
            }

            images.push([img_0, img_1]);
        }

        Self { current_index: 0, images }
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
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        let mut current_y = 5; // Start very close to the top

        for (i, variants) in self.images.iter().enumerate() {
            let selection_index = if i == self.current_index { 1 } else { 0 };

            if let Some(bmp) = &variants[selection_index] {
                let start_x = (400usize.saturating_sub(bmp.width)) / 2;
                
                for y in 0..bmp.height {
                    let screen_y = current_y + y as i32;
                    // Only draw if the pixel is actually on the screen
                    if screen_y >= 0 && screen_y < 240 {
                        for x in 0..bmp.width {
                            let pixel = bmp.pixels[y * bmp.width + x];
                            display.draw_pixel(start_x + x, screen_y as usize, pixel, ctx);
                        }
                    }
                }
                // SPACING SET TO 0: Only move down by the height of the image
                current_y += bmp.height as i32;
            } else {
                // If a file is missing, we draw a small label so the list doesn't break
                display.draw_text(160, current_y as usize, &format!("<{}>", SETTINGS_OPTIONS[i]), ctx);
                current_y += 20; 
            }
        }
    }
}