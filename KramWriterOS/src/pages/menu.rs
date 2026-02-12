use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;

// Ensure these match your actual filenames (lowercase)
const MENU_OPTIONS: [&str; 5] = ["write", "learn", "zeugtris", "settings", "credits"];

const SPACING_TOP_TO_MAIN: i32 = -10;
const SPACING_MAIN_TO_BOTTOM: i32 = 10;
const SPACING_TOP_TO_FARTOP: i32 = 30;
const SPACING_BOTTOM_TO_FARBOTTOM: i32 = 45;

pub struct MenuPage {
    current_index: usize,
    images: Vec<Vec<Option<Bitmap>>>,
}

impl MenuPage {
    pub fn new() -> Self {
        let mut images = Vec::new();
        
        for option in MENU_OPTIONS.iter() {
            let mut variants = Vec::new();
            for suffix in 0..3 {
                let path = format!("/home/kramwriter/KramWriter/assets/menu/{}_{}.bmp", option.to_lowercase(), suffix);
                
                match Bitmap::load(&path) {
                    Ok(bmp) => variants.push(Some(bmp)),
                    Err(_) => {
                        println!("⚠️ Failed to load menu asset: {}", path);
                        variants.push(None);
                    }
                }
            }
            images.push(variants);
        }
        
        Self {
            current_index: 0,
            images,
        }
    }

    fn draw_bitmap_at(&self, display: &mut SharpDisplay, bmp: &Bitmap, y_pos: i32, ctx: &Context) {
        let start_x = (400usize.saturating_sub(bmp.width)) / 2;
        
        for y in 0..bmp.height {
            let screen_y = y_pos + y as i32;
            // Only draw if within screen vertical bounds
            if screen_y >= 0 && screen_y < 240 {
                for x in 0..bmp.width {
                    if start_x + x < 400 {
                        let pixel = bmp.pixels[y * bmp.width + x];
                        display.draw_pixel(start_x + x, screen_y as usize, pixel, ctx);
                    }
                }
            }
        }
    }
}

impl Page for MenuPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Char('\n') => Action::None, // We will add WritingPage later
            Key::Esc => Action::Replace(Box::new(crate::pages::startup::LogoPage::new())),
            Key::Up => {
                if self.current_index > 0 { self.current_index -= 1; }
                Action::None
            },
            Key::Down => {
                if self.current_index < MENU_OPTIONS.len() - 1 { self.current_index += 1; }
                Action::None
            },
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // Variant 1 is the "Selected" larger image
        if let Some(main_bmp) = self.images[self.current_index][1].as_ref() {
            let cy = (240i32 - main_bmp.height as i32) / 2;
            self.draw_bitmap_at(display, main_bmp, cy, ctx);

            let h = main_bmp.height as i32;

            // Previous (Top) - Variant 0
            if self.current_index > 0 {
                if let Some(img) = self.images[self.current_index - 1][0].as_ref() {
                    let y = cy - img.height as i32 + SPACING_TOP_TO_MAIN;
                    self.draw_bitmap_at(display, img, y, ctx);

                    // Far Top - Variant 2
                    if self.current_index > 1 {
                        if let Some(far_img) = self.images[self.current_index - 2][2].as_ref() {
                            let far_y = y - far_img.height as i32 - SPACING_TOP_TO_FARTOP;
                            self.draw_bitmap_at(display, far_img, far_y, ctx);
                        }
                    }
                }
            }

            // Next (Bottom) - Variant 0
            if self.current_index < MENU_OPTIONS.len() - 1 {
                if let Some(img) = self.images[self.current_index + 1][0].as_ref() {
                    let y = cy + h + SPACING_MAIN_TO_BOTTOM;
                    self.draw_bitmap_at(display, img, y, ctx);

                    // Far Bottom - Variant 2
                    if self.current_index < MENU_OPTIONS.len() - 2 {
                        if let Some(far_img) = self.images[self.current_index + 2][2].as_ref() {
                            let far_y = y + img.height as i32 + SPACING_BOTTOM_TO_FARBOTTOM;
                            self.draw_bitmap_at(display, far_img, far_y, ctx);
                        }
                    }
                }
            }
        } else {
            display.draw_text(130, 110, "--- MENU ASSETS MISSING ---", ctx);
        }
    }
}