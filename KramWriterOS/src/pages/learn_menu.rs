use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;

// Vertical offsets for the menu items
const OPTION_1_Y: i32 = 40; 
const OPTION_2_Y: i32 = 90;

pub struct LearnMenuPage {
    current_index: usize, 
    title: Option<Bitmap>,
    option_1_variants: [Option<Bitmap>; 2],
    option_2_variants: [Option<Bitmap>; 2],
}

impl LearnMenuPage {
    pub fn new() -> Self {
        // Path adjusted to lowercase 'w' to match your Pi's filesystem
        let asset_path = "/home/kramwriter/Kramwriter/assets/Learn/menu";
        
        Self {
            current_index: 0,
            title: Bitmap::load(&format!("{}/title.bmp", asset_path)).ok(),
            option_1_variants: [
                Bitmap::load(&format!("{}/option_1_0.bmp", asset_path)).ok(), // Selected
                Bitmap::load(&format!("{}/option_1_1.bmp", asset_path)).ok(), // Unselected
            ],
            option_2_variants: [
                Bitmap::load(&format!("{}/option_2_0.bmp", asset_path)).ok(), // Selected
                Bitmap::load(&format!("{}/option_2_1.bmp", asset_path)).ok(), // Unselected
            ],
        }
    }

    fn draw_layer(&self, display: &mut SharpDisplay, bmp: &Bitmap, y_offset: i32, ctx: &Context) {
        for y in 0..bmp.height {
            let screen_y = y as i32 + y_offset;
            if screen_y >= 0 && screen_y < 240 {
                for x in 0..bmp.width.min(400) {
                    // Using the pixel data from the loaded bitmap
                    let pixel = bmp.pixels[y * bmp.width + x];
                    if pixel == Pixel::Black {
                        display.draw_pixel(x, screen_y as usize, Pixel::Black, ctx);
                    }
                }
            }
        }
    }
}

impl Page for LearnMenuPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Up => {
                if self.current_index == 1 { self.current_index = 0; }
                Action::None
            }
            Key::Down => {
                if self.current_index == 0 { self.current_index = 1; }
                Action::None
            }
            Key::Char('\n') => {
                match self.current_index {
                    0 => Action::None, // Placeholder for first Learn option
                    1 => Action::None, // Placeholder for second Learn option
                    _ => Action::None,
                }
            }
            Key::Esc => Action::Pop, // Returns to main MenuPage
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // Clear screen based on dark mode context
        display.clear(ctx);

        // 1. Draw Title
        if let Some(bmp) = &self.title { 
            self.draw_layer(display, bmp, 0, ctx); 
        }

        // 2. Draw Option 1 (Uses variant 0 if selected, variant 1 if not)
        let opt1_idx = if self.current_index == 0 { 0 } else { 1 };
        if let Some(bmp) = &self.option_1_variants[opt1_idx] {
            self.draw_layer(display, bmp, OPTION_1_Y, ctx);
        }

        // 3. Draw Option 2 (Uses variant 0 if selected, variant 1 if not)
        let opt2_idx = if self.current_index == 1 { 0 } else { 1 };
        if let Some(bmp) = &self.option_2_variants[opt2_idx] {
            self.draw_layer(display, bmp, OPTION_2_Y, ctx);
        }
    }
}