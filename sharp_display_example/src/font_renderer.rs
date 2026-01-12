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
        let positioned = glyph.positioned(point(0.0, 0.0));
        
        let bb = positioned.pixel_bounding_box();
        println!("  Bounding box: {:?}", bb);
        
        let bb = bb?;
        let width = bb.width() as usize;
        let height = bb.height() as usize;
        
        println!("  Character dimensions: {}x{}", width, height);
        
        if width == 0 || height == 0 {
            println!("  Warning: Character has zero dimensions!");
            return None;
        }
        
        let mut bitmap = vec![vec![false; width]; height];
        
        positioned.draw(|x, y, v| {
            let x = x as i32 - bb.min.x;
            let y = y as i32 - bb.min.y;
            
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                if v > 0.3 {
                    bitmap[y as usize][x as usize] = true;
                }
            }
        });
        
        // Count how many pixels are set
        let pixel_count = bitmap.iter().flatten().filter(|&&p| p).count();
        println!("  Pixels set: {}/{} ({:.1}%)", 
            pixel_count, width * height,
            (pixel_count as f32 * 100.0) / (width * height) as f32);
        
        Some(CharBitmap {
            width,
            height,
            bitmap,
            advance,
        })
    }
    
    pub fn line_height(&self) -> f32 {
        let v_metrics = self.font.v_metrics(self.scale);
        v_metrics.ascent - v_metrics.descent + v_metrics.line_gap
    }
}

pub struct CharBitmap {
    pub width: usize,
    pub height: usize,
    pub bitmap: Vec<Vec<bool>>,
    pub advance: f32,
}