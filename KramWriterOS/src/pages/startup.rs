use crate::pages::{Page, Action, menu::MenuPage};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;

pub struct LogoPage {
    logo: Option<Bitmap>,
}

impl LogoPage {
    pub fn new() -> Self {
        let path = "/home/kramwriter/KramWriter/assets/logo/logo.bmp";
        println!("Loading logo from: {}", path);
        
        let logo = Bitmap::load(path).ok();
        
        Self { logo }
    }
}

impl Page for LogoPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            // When Enter is pressed, swap the LogoPage for the MenuPage
            Key::Char('\n') => Action::Replace(Box::new(crate::pages::menu::MenuPage::new())),
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // 1. CRITICAL: Clear the display first so we have a fresh canvas
        display.clear(ctx);

        if let Some(bmp) = &self.logo {
            // Center the logo
            let start_x = (400usize.saturating_sub(bmp.width)) / 2;
            let start_y = (240usize.saturating_sub(bmp.height)) / 2;
            
            for y in 0..bmp.height {
                let screen_y = start_y + y;
                if screen_y >= 240 { continue; } // Safety check

                for x in 0..bmp.width {
                    let screen_x = start_x + x;
                    if screen_x >= 400 { continue; } // Safety check

                    let pixel = bmp.pixels[y * bmp.width + x];
                    display.draw_pixel(screen_x, screen_y, pixel, ctx);
                }
            }
        } else {
            // Fallback if file missing
            display.draw_text(150, 100, "LOGO MISSING", ctx);
        }

        // 2. Note: If your SharpDisplay requires a flush/update call to show 
        // pixels on the physical screen, ensure the main loop or the 
        // display driver is handling that.
    }
}