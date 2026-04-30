use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;

pub struct LearnMenuPage {
    current_index: usize, 
    title: Option<Bitmap>,
    // Array to hold the three full-screen option variants
    options: [Option<Bitmap>; 3],
}

impl LearnMenuPage {
    pub fn new() -> Self {
        // Using the lowercase 'w' as found in your Pi's terminal output
        let asset_path = "/home/kramwriter/KramWriter/assets/Learn/Menu";
        
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

    /// Draws a full-screen 400x240 bitmap
    fn draw_full_screen(&self, display: &mut SharpDisplay, bmp: &Bitmap, ctx: &Context) {
        for y in 0..bmp.height.min(240) {
            for x in 0..bmp.width.min(400) {
                let pixel = bmp.pixels[y * bmp.width + x];
                // Only draw Black pixels to allow layering (Title over Options)
                // If your bitmaps have white backgrounds, this acts as transparency
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
                    self.current_index = 2; // Wrap around to bottom
                }
                Action::None
            }
            Key::Down => {
                if self.current_index < 2 {
                    self.current_index += 1;
                } else {
                    self.current_index = 0; // Wrap around to top
                }
                Action::None
            }
            Key::Char('\n') => {
                match self.current_index {
                    0 => Action::None, // Action for Option 0
                    1 => Action::None, // Action for Option 1
                    2 => Action::None, // Action for Option 2
                    _ => Action::None,
                }
            }
            Key::Esc => Action::Pop, // Back to main menu
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // 1. Clear with background color based on Dark Mode
        display.clear(ctx);

        // 2. Draw the selected Option background first
        if let Some(bmp) = &self.options[self.current_index] {
            self.draw_full_screen(display, bmp, ctx);
        }

        // 3. Draw the Title over the top
        if let Some(bmp) = &self.title {
            self.draw_full_screen(display, bmp, ctx);
        }
    }
}