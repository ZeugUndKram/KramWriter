use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;

pub struct CreditsPage {
    image: Option<Bitmap>,
}

impl CreditsPage {
    pub fn new() -> Self {
        let path = "/home/kramwriter/KramWriter/assets/Credits/credits.bmp";
        let image = Bitmap::load(path).ok();
        if image.is_none() {
            println!("⚠️ Failed to load credits asset: {}", path);
        }
        Self { image }
    }
}

impl Page for CreditsPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            // Only Esc triggers the return (Pop)
            Key::Esc => Action::Pop,
            
            // Ignore everything else so the user stays on the page
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        if let Some(bmp) = &self.image {
            let start_x = (400usize.saturating_sub(bmp.width)) / 2;
            let start_y = (240usize.saturating_sub(bmp.height)) / 2;
            
            for y in 0..bmp.height {
                let screen_y = start_y as i32 + y as i32;
                if screen_y >= 0 && screen_y < 240 {
                    for x in 0..bmp.width {
                        let pixel = bmp.pixels[y * bmp.width + x];
                        display.draw_pixel(start_x + x, screen_y as usize, pixel, ctx);
                    }
                }
            }
        } else {
            // Fallback text if image is missing
            display.draw_text(140, 110, "Credits File Missing", ctx);
        }
    }
}