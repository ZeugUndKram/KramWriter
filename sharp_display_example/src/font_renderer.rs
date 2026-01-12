use rusttype::{Font, Scale, point};
use anyhow::{Result, anyhow};

pub struct FontRenderer {
    font: Font<'static>,
    scale: Scale,
}

impl FontRenderer {
    pub fn from_file(path: &str, pixel_height: f32) -> Result<Self> {
        println!("  Reading font file: {}", path);
        let font_data = std::fs::read(path)
            .map_err(|e| anyhow!("Failed to read font file {}: {}", path, e))?;
        
        println!("  Font file size: {} bytes", font_data.len());
        
        let font = Font::try_from_vec(font_data)
            .ok_or_else(|| anyhow!("Failed to parse font file {}", path))?;
        
        let scale = Scale::uniform(pixel_height);
        
        println!("  Font created with scale: {:?}", scale);
        Ok(Self { font, scale })
    }
    
    pub fn render_char(&self, ch: char) -> Option<CharBitmap> {
        println!("  Rendering character: '{}'", ch);
        
        let glyph = self.font.glyph(ch).scaled(self.scale);
        let advance = glyph.h_metrics().advance_width;
        
        // Position at baseline
        let positioned = glyph.positioned(point(0.0, 0.0));
        
        let bb = positioned.pixel_bounding_box();
        println!("  Bounding box: {:?}", bb);
        
        let bb = bb?;
        
        // Adjust for negative y coordinates
        let x_offset = -bb.min.x;
        let y_offset = -bb.min.y;
        
        let width = bb.width() as usize;
        let height = bb.height() as usize;
        
        println!("  Character dimensions: {}x{}", width, height);
        println!("  Offsets: x={}, y={}", x_offset, y_offset);
        
        if width == 0 || height == 0 {
            println!("  Warning: Character has zero dimensions!");
            return None;
        }
        
        let mut bitmap = vec![vec![false; width]; height];
        let mut pixels_drawn = 0;
        
        positioned.draw(|x, y, v| {
            // Convert to bitmap coordinates
            let bitmap_x = (x as i32 + x_offset) as usize;
            let bitmap_y = (y as i32 + y_offset) as usize;
            
            if bitmap_x < width && bitmap_y < height {
                if v > 0.1 {  // Lower threshold
                    bitmap[bitmap_y][bitmap_x] = true;
                    pixels_drawn += 1;
                }
            }
        });
        
        println!("  Pixels set: {}/{} ({:.1}%)", 
            pixels_drawn, width * height,
            (pixels_drawn as f32 * 100.0) / (width * height) as f32);
        
        if pixels_drawn == 0 {
            println!("  WARNING: No pixels were drawn!");
            // Try a test pattern instead
            return self.create_test_pattern(ch, width, height);
        }
        
        Some(CharBitmap {
            width,
            height,
            bitmap,
            advance,
        })
    }
    
    fn create_test_pattern(&self, ch: char, width: usize, height: usize) -> Option<CharBitmap> {
        println!("  Creating test pattern for '{}'", ch);
        
        let mut bitmap = vec![vec![false; width]; height];
        
        // Draw a simple pattern to verify rendering works
        for y in 0..height {
            for x in 0..width {
                // Draw border
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    bitmap[y][x] = true;
                }
                // Draw diagonal
                if x == y {
                    bitmap[y][x] = true;
                }
            }
        }
        
        Some(CharBitmap {
            width,
            height,
            bitmap,
            advance: width as f32,
        })
    }
    
    pub fn line_height(&self) -> f32 {
        let v_metrics = self.font.v_metrics(self.scale);
        v_metrics.ascent - v_metrics.descent + v_metrics.line_gap
    }
    
    pub fn ascent(&self) -> f32 {
        self.font.v_metrics(self.scale).ascent
    }
}

pub struct CharBitmap {
    pub width: usize,
    pub height: usize,
    pub bitmap: Vec<Vec<bool>>,
    pub advance: f32,
}