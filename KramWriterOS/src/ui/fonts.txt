use fontdue::{Font, FontSettings};
use std::fs;

pub struct FontRenderer {
    pub font: Font,
}

impl FontRenderer {
    pub fn new(path: &str) -> Self {
        let font_data = fs::read(path).expect("Failed to read font file");
        let font = Font::from_bytes(font_data, FontSettings::default()).expect("Failed to parse font");
        Self { font }
    }

    pub fn draw_text(&self, display: &mut crate::display::SharpDisplay, text: &str, x: i32, y: i32, size: f32, ctx: &crate::context::Context) {
        let mut x_cursor = x as f32;

        for char in text.chars() {
            let (metrics, bitmap) = self.font.rasterize(char, size);
            
            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let coverage = bitmap[row * metrics.width + col];
                    
                    // fontdue returns 0-255 coverage. 
                    // Since your screen is 1-bit, we use a threshold (128).
                    if coverage > 128 {
                        let pixel_x = (x_cursor + col as f32 + metrics.xmin as f32) as usize;
                        let pixel_y = (y as f32 + row as f32 - metrics.ymin as f32 - metrics.height as f32) as usize;
                        
                        display.draw_pixel(pixel_x, pixel_y, rpi_memory_display::Pixel::Black, ctx);
                    }
                }
            }
            // Advance the cursor to the next character position
            x_cursor += metrics.advance_width;
        }
    }
}