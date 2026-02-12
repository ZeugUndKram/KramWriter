use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;

const MENU_OPTIONS: [&str; 5] = ["Write", "Learn", "Zeugtris", "Settings", "Credits"];

const SPACING_TOP_TO_MAIN: i32 = -10;
const SPACING_MAIN_TO_BOTTOM: i32 = 10;
const SPACING_TOP_TO_FARTOP: i32 = 10;
const SPACING_BOTTOM_TO_FARBOTTOM: i32 = 10;

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
                let path = format!("/home/kramwriter/KramWriter/assets/Menu/{}_{}.bmp", option, suffix);
                variants.push(Bitmap::load(&path).ok());
            }//test
            images.push(variants);
        }
        Self { current_index: 0, images }
    }

    fn draw_bitmap_at(&self, display: &mut SharpDisplay, bmp: &Bitmap, y_pos: i32, ctx: &Context) {
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
            Key::Up => {
                if self.current_index > 0 {
                    self.current_index -= 1;
                }
                Action::None
            }
            Key::Down => {
                if self.current_index < MENU_OPTIONS.len() - 1 {
                    self.current_index += 1;
                }
                Action::None
            }
            // Handle the selection (Enter key)
            Key::Char('\n') => {
                match self.current_index {
                    0 => Action::None, // Write (Add later)
                    1 => Action::None, // Learn (Add later)
                    2 => Action::None, // Zeugtris (Add later)
                    3 => Action::Push(Box::new(crate::pages::settings::SettingsPage::new())),
                    4 => Action::Push(Box::new(crate::pages::credits::CreditsPage::new())), // Credits
                    _ => Action::None,
                }
            }
            Key::Esc => Action::Replace(Box::new(crate::pages::startup::LogoPage::new())),
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // CENTER: Suffix _0 (Variant index 0)
        if let Some(main_bmp) = self.images[self.current_index][0].as_ref() {
            let cy = (240i32 - main_bmp.height as i32) / 2;
            self.draw_bitmap_at(display, main_bmp, cy, ctx);
            let h = main_bmp.height as i32;

            // TOP: Suffix _1 (Variant index 1)
            if self.current_index > 0 {
                if let Some(img) = self.images[self.current_index - 1][1].as_ref() {
                    let y = cy - img.height as i32 + SPACING_TOP_TO_MAIN;
                    self.draw_bitmap_at(display, img, y, ctx);

                    // FAR TOP: Suffix _2 (Variant index 2)
                    if self.current_index > 1 {
                        if let Some(far_img) = self.images[self.current_index - 2][2].as_ref() {
                            let far_y = y - far_img.height as i32 - SPACING_TOP_TO_FARTOP;
                            self.draw_bitmap_at(display, far_img, far_y, ctx);
                        }
                    }
                }
            }

            // BOTTOM: Suffix _1 (Variant index 1)
            if self.current_index < MENU_OPTIONS.len() - 1 {
                if let Some(img) = self.images[self.current_index + 1][1].as_ref() {
                    let y = cy + h + SPACING_MAIN_TO_BOTTOM;
                    self.draw_bitmap_at(display, img, y, ctx);

                    // FAR BOTTOM: Suffix _2 (Variant index 2)
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
}