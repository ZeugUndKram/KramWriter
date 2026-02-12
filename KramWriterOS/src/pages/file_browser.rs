use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;

pub struct FileBrowserPage {
    home_icon: Option<Bitmap>,
    renderer: FontRenderer,
    current_path: String,
}

impl FileBrowserPage {
    pub fn new() -> Self {
        let home_icon = Bitmap::load("/home/kramwriter/KramWriter/assets/FileBrowser/icon_home.bmp").ok();
        
        // Initialize the font renderer with your BebasNeue TTF
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        
        Self {
            home_icon,
            renderer,
            // Hardcoded for now to match your reference image
            current_path: String::from("/MAIN/NOTIZEN/"),
        }
    }
}

impl Page for FileBrowserPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            // Esc returns to the Write Menu
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // 1. Draw the Top Divider Line FIRST
        // This ensures the icon and text sit "on top" of it visually
        for x in 0..400 {
            display.draw_pixel(x, 22, Pixel::Black, ctx);
        }

        // 2. Draw Home Icon 
        if let Some(bmp) = &self.home_icon {
            // We'll move it slightly to (4, 2) to ensure it's not touching the divider
            // unless it's intended to.
            self.draw_icon(display, bmp, 4, 2, ctx);
        } else {
            // Debug: If the icon fails to load, draw a small square so you know it's missing
            for y in 2..15 {
                for x in 4..15 {
                    display.draw_pixel(x, y, Pixel::Black, ctx);
                }
            }
        }

        // 3. Draw Path Text
        // If the icon is 20px wide, 35 is a good X offset.
        self.renderer.draw_text(
            display, 
            &self.current_path, 
            35, 
            19, // Adjusted slightly for the font's baseline
            20.0, 
            ctx
        );
    }
}

impl FileBrowserPage {
    /// Helper to draw small icons with transparency (skipping white pixels)
    fn draw_icon(&self, display: &mut SharpDisplay, bmp: &Bitmap, x_offset: usize, y_offset: usize, ctx: &Context) {
        for y in 0..bmp.height {
            for x in 0..bmp.width {
                let pixel = bmp.pixels[y * bmp.width + x];
                // Only draw the black pixels of the icon
                if pixel == Pixel::Black {
                    let screen_x = x + x_offset;
                    let screen_y = y + y_offset;
                    if screen_x < 400 && screen_y < 240 {
                        display.draw_pixel(screen_x, screen_y, pixel, ctx);
                    }
                }
            }
        }
    }
}