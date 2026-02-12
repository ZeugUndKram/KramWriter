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
            Key::Char('\n') => Action::Replace(Box::new(MenuPage::new())),
            Key::Esc => Action::Exit,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        if let Some(bmp) = &self.logo {
            // Center the logo
            let start_x = (400usize.saturating_sub(bmp.width)) / 2;
            let start_y = (240usize.saturating_sub(bmp.height)) / 2;
            
            for y in 0..bmp.height {
                for x in 0..bmp.width {
                    if y < 240 && x < 400 {
                        let pixel = bmp.pixels[y * bmp.width + x];
                        display.draw_pixel(start_x + x, start_y + y, pixel, ctx);
                    }
                }
            }
        } else {
            // Fallback if file missing
            display.draw_text(150, 100, "LOGO MISSING", ctx);
        }
    }
}