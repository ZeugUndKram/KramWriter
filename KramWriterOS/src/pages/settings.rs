use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;

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
            
            // Handle the unique naming for the keyboard selected state
            let suffix_selected = if *option == "keyboard" { "3" } else { "1" };
            let path_selected = format!("/home/kramwriter/KramWriter/assets/Settings/{}_{}.bmp", option, suffix_selected);
            
            let img_0 = Bitmap::load(&path_0).ok();
            let img_selected = Bitmap::load(&path_selected).ok();

            if img_selected.is_none() {
                println!("❌ Failed to load selected state: {}", path_selected);
            }

            images.push([img_0, img_selected]);
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
        // Draw every image at (0,0) so their internal transparency 
        // allows them to stack on top of each other.
        for (i, variants) in self.images.iter().enumerate() {
            let selection_index = if i == self.current_index { 1 } else { 0 };

            if let Some(bmp) = &variants[selection_index] {
                for y in 0..bmp.height {
                    if y < 240 {
                        for x in 0..bmp.width {
                            if x < 400 {
                                let pixel = bmp.pixels[y * bmp.width + x];
                                
                                // Only draw Black pixels to treat White as transparent
                                if pixel == Pixel::Black {
                                    display.draw_pixel(x, y, pixel, ctx);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}