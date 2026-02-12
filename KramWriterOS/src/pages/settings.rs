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
        // We no longer use a moving current_y because 
        // each BMP is already a full-screen 400x240 map.
        let start_y = 0;
        let start_x = 0;

        for (i, variants) in self.images.iter().enumerate() {
            let selection_index = if i == self.current_index { 1 } else { 0 };

            if let Some(bmp) = &variants[selection_index] {
                for y in 0..bmp.height {
                    let screen_y = start_y + y as i32;
                    if screen_y >= 0 && screen_y < 240 {
                        for x in 0..bmp.width {
                            let pixel = bmp.pixels[y * bmp.width + x];
                            
                            // TRANSPARENCY LOGIC:
                            // Only draw the pixel if it is Black. 
                            // This prevents the "white" background of the top image
                            // from erasing the "black" text of the image underneath.
                            if pixel == rpi_memory_display::Pixel::Black {
                                display.draw_pixel(start_x + x, screen_y as usize, pixel, ctx);
                            }
                        }
                    }
                }
            }
        }
    }