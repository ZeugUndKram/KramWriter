use fontdue::{Font, FontSettings};
use std::fs;
use crate::display::SharpDisplay;
use crate::context::Context;
use rpi_memory_display::Pixel;

pub struct FontRenderer {
    pub font: Font,
}

impl FontRenderer {
    pub fn new(path: &str) -> Self {
        let font_data = fs::read(path).expect("Failed to read font file");
        let font = Font::from_bytes(font_data, FontSettings::default()).expect("Failed to parse font");
        Self { font }
    }

    /// Standard black text rendering
    pub fn draw_text(&self, display: &mut SharpDisplay, text: &str, x: i32, y: i32, size: f32, ctx: &Context) {
        self.draw_text_colored(display, text, x, y, size, Pixel::Black, ctx);
    }

    /// Colored text rendering (allows White text for selected rows)
    pub fn draw_text_colored(&self, display: &mut SharpDisplay, text: &str, x: i32, y: i32, size: f32, color: Pixel, ctx: &Context) {
        let mut x_cursor = x as f32;

        for char in text.chars() {
            let (metrics, bitmap) = self.font.rasterize(char, size);
            
            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let coverage = bitmap[row * metrics.width + col];
                    
                    if coverage > 128 {
                        let px = (x_cursor + col as f32 + metrics.xmin as f32) as i32;
                        let py = (y as f32 + row as f32 - metrics.ymin as f32 - metrics.height as f32) as i32;
                        
                        // Bounds check to ensure we don't crash if text goes off-screen
                        if px >= 0 && px < 400 && py >= 0 && py < 240 {
                            display.draw_pixel(px as usize, py as usize, color, ctx);
                        }
                    }
                }
            }
            x_cursor += metrics.advance_width;
        }
    }
    pub fn calculate_width(&self, text: &str, size: f32) -> i32 {
        let mut total_width = 0.0;
        
        for c in text.chars() {
            // fontdue uses metrics() to get character dimensions
            let metrics = self.font.metrics(c, size);
            total_width += metrics.advance_width;
        }

        total_width as i32
    }
}