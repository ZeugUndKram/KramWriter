use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;

const MENU_OPTIONS: [&str; 5] = ["write", "learn", "zeugtris", "settings", "credits"];

// Your spacing constants
const SPACING_TOP_TO_MAIN: i32 = -10;
const SPACING_MAIN_TO_BOTTOM: i32 = 10;
const SPACING_TOP_TO_FARTOP: i32 = 30;
const SPACING_BOTTOM_TO_FARBOTTOM: i32 = 45;

pub struct MenuPage {
    current_index: usize,
    // Outer Vec: Options (Write, Learn...), Inner Vec: Variants (0, 1, 2)
    images: Vec<Vec<Option<Bitmap>>>,
}

impl MenuPage {
    pub fn new() -> Self {
        let mut images = Vec::new();
        
        for option in MENU_OPTIONS.iter() {
            let mut variants = Vec::new();
            // Load 0 (unselected), 1 (selected), 2 (extra)
            for suffix in 0..3 {
                let path = format!("/home/kramwriter/KramWriter/assets/menu/{}_{}.bmp", option, suffix);
                variants.push(Bitmap::load(&path).ok());
            }
            images.push(variants);
        }
        
        Self {
            current_index: 0,
            images,
        }
    }

    fn draw_bitmap_at(&self, display: &mut SharpDisplay, bmp: &Bitmap, y_pos: i32, ctx: &Context) {
        // Center Horizontally
        let start_x = (400usize.saturating_sub(bmp.width)) / 2;
        
        for y in 0..bmp.height {
            let screen_y = y_pos + y as i32;
            if screen_y >= 0 && screen_y < 240 {
                for x in 0..bmp.width {
                    let pixel = bmp.pixels[y * bmp.width + x];
                    display.draw_pixel(start_x + x, screen_y as usize, pixel, ctx);
                }
            }
        }
    }
}

impl Page for MenuPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Char('\n') => {
                // Return Action based on selection
                match self.current_index {
                    0 => Action::None, // TODO: Action::Push(Box::new(WritingPage::new()))
                    _ => Action::None,
                }
            }
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
        // We need at least the main image to draw
        let main_bmp = match self.images[self.current_index][1].as_ref() {
            Some(b) => b,
            None => {
                display.draw_text(10, 10, "Assets Missing", ctx);
                return;
            }
        };

        // Calculate center position
        let cy = (240i32 - main_bmp.height as i32) / 2;
        
        // 1. Draw Main (Selected) Image
        self.draw_bitmap_at(display, main_bmp, cy, ctx);

        let h = main_bmp.height as i32;

        // 2. Draw Previous (Top) Image
        if self.current_index > 0 {
            if let Some(img) = self.images[self.current_index - 1][0].as_ref() {
                let y = cy - img.height as i32 + SPACING_TOP_TO_MAIN;
                self.draw_bitmap_at(display, img, y, ctx);

                // 3. Draw Far Top
                if self.current_index > 1 {
                    if let Some(far_img) = self.images[self.current_index - 2][2].as_ref() {
                        let far_y = y - far_img.height as i32 - SPACING_TOP_TO_FARTOP;
                        self.draw_bitmap_at(display, far_img, far_y, ctx);
                    }
                }
            }
        }

        // 4. Draw Next (Bottom) Image
        if self.current_index < MENU_OPTIONS.len() - 1 {
            if let Some(img) = self.images[self.current_index + 1][0].as_ref() {
                let y = cy + h + SPACING_MAIN_TO_BOTTOM;
                self.draw_bitmap_at(display, img, y, ctx);

                // 5. Draw Far Bottom
                if self.current_index < MENU_OPTIONS.len() - 2 {
                    if let Some(far_img) = self.images[self.current_index + 2][2].as_ref() {
                        let far_y = y + img.height as i32 + SPACING_BOTTOM_TO_FARBOTTOM;
                        self.draw_bitmap_at(display, far_img, far_y, ctx);
                    }
                }
            }
        }
    }
}