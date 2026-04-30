use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;

// --- Updated imports to match your file names ---
use crate::pages::zeugtris::ZeugtrisPage;
use crate::pages::zeugtris_highscores::ZeugtrisHighscoresPage;

pub struct ZeugtrisMenuPage {
    current_index: usize, 
    title: Option<Bitmap>,
    options: [Option<Bitmap>; 2], // Only 2 options: Play and Highscores
}

impl ZeugtrisMenuPage {
    pub fn new() -> Self {
        // Path to your Zeugtris assets
        let asset_path = "/home/kramwriter/KramWriter/assets/zeugtris/menu";
        
        Self {
            current_index: 0,
            title: Bitmap::load(&format!("{}/Title.bmp", asset_path)).ok(),
            options: [
                Bitmap::load(&format!("{}/Options_0.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/Options_1.bmp", asset_path)).ok(),
            ],
        }
    }

    fn draw_full_screen(&self, display: &mut SharpDisplay, bmp: &Bitmap, ctx: &Context) {
        for y in 0..bmp.height.min(240) {
            for x in 0..bmp.width.min(400) {
                let pixel = bmp.pixels[y * bmp.width + x];
                if pixel == Pixel::Black {
                    display.draw_pixel(x, y, Pixel::Black, ctx);
                }
            }
        }
    }
}

impl Page for ZeugtrisMenuPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Up | Key::Down => {
                // Toggles between index 0 (Play) and index 1 (Highscores)
                self.current_index = if self.current_index == 0 { 1 } else { 0 };
                Action::None
            }
            Key::Char('\n') => {
                match self.current_index {
                    0 => Action::Push(Box::new(ZeugtrisPage::new())),           // PLAY
                    1 => Action::Push(Box::new(ZeugtrisHighscoresPage::new())), // HIGHSCORES
                    _ => Action::None,
                }
            }
            Key::Esc => Action::Pop, 
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        display.clear(ctx);

        // Draw the background option highlight
        if let Some(bmp) = &self.options[self.current_index] {
            self.draw_full_screen(display, bmp, ctx);
        }

        // Draw the static Title over it
        if let Some(bmp) = &self.title {
            self.draw_full_screen(display, bmp, ctx);
        }
    }
}