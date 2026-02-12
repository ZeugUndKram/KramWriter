use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;

// Order as requested: Timezone, Location, Darkmode, Drive, Keyboard
const SETTINGS_OPTIONS: [&str; 5] = ["Timezone", "Location", "Darkmode", "Drive", "Keyboard"];
const VERTICAL_SPACING: i32 = 10; // Change this to adjust gaps between images

pub struct SettingsPage {
    current_index: usize,
    images: Vec<[Option<Bitmap>; 2]>, // Index 0: unselected (_0), Index 1: selected (_1)
}

impl SettingsPage {
    pub fn new() -> Self {
        let mut images = Vec::new();
        for option in SETTINGS_OPTIONS.iter() {
            let path_0 = format!("/home/kramwriter/KramWriter/assets/Settings/{}_0.bmp", option);
            let path_1 = format!("/home/kramwriter/KramWriter/assets/Settings/{}_1.bmp", option);
            
            images.push([
                Bitmap::load(&path_0).ok(),
                Bitmap::load(&path_1).ok(),
            ]);
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
            Key::Esc => Action::Pop, // Return to Main Menu
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        let mut current_y = 20; // Starting Y offset from top

        for (i, variants) in self.images.iter().enumerate() {
            // Pick _1 if selected, otherwise _0
            let version = if i == self.current_index { 1 } else { 0 };

            if let Some(bmp) = &variants[version] {
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
                // Move Y down by image height + spacing for the next item
                current_y += bmp.height as i32 + VERTICAL_SPACING;
            }
        }
    }
}