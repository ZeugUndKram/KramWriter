use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;

// --- Added the import for the Learn file browser ---
use crate::pages::file_browser_learn::{FileBrowserLearnPage, BrowserMode};

pub struct LearnMenuPage {
    current_index: usize, 
    title: Option<Bitmap>,
    options: [Option<Bitmap>; 3],
}

impl LearnMenuPage {
    pub fn new() -> Self {
        // Ensure this path matches the exact case of your folder on the Pi
        let asset_path = "/home/kramwriter/Kramwriter/assets/Learn/Menu";
        
        Self {
            current_index: 0,
            title: Bitmap::load(&format!("{}/Title.bmp", asset_path)).ok(),
            options: [
                Bitmap::load(&format!("{}/Options_0.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/Options_1.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/Options_2.bmp", asset_path)).ok(),
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

impl Page for LearnMenuPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Up => {
                if self.current_index > 0 {
                    self.current_index -= 1;
                } else {
                    self.current_index = 2; 
                }
                Action::None
            }
            Key::Down => {
                if self.current_index < 2 {
                    self.current_index += 1;
                } else {
                    self.current_index = 0; 
                }
                Action::None
            }
            Key::Char('\n') => {
                match self.current_index {
                    // --- Connected the actions here ---
                    0 => Action::Push(Box::new(FileBrowserLearnPage::new(BrowserMode::OpenFile))), // OPEN DECK
                    1 => Action::Push(Box::new(FileBrowserLearnPage::new(BrowserMode::Full))),     // CREATE DECK
                    2 => Action::None, // SETTINGS (To be implemented later)
                    _ => Action::None,
                }
            }
            Key::Esc => Action::Pop, 
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        display.clear(ctx);

        if let Some(bmp) = &self.options[self.current_index] {
            self.draw_full_screen(display, bmp, ctx);
        }

        if let Some(bmp) = &self.title {
            self.draw_full_screen(display, bmp, ctx);
        }
    }
}