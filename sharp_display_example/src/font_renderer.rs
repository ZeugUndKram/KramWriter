use ab_glyph::{FontRef, PxScale, point};
use anyhow::{Result, anyhow};

pub struct FontRenderer {
    font: FontRef<'static>,
    scale: PxScale,
}

impl FontRenderer {
    pub fn from_file(path: &str, pixel_height: f32) -> Result<Self> {
        println!("  Reading font file: {}", path);
        let font_data = std::fs::read(path)
            .map_err(|e| anyhow!("Failed to read font file {}: {}", path, e))?;
        
        println!("  Font file size: {} bytes", font_data.len());
        
        let font = FontRef::try_from_slice(&font_data)
            .ok_or_else(|| anyhow!("Failed to parse font file {}", path))?;
        
        let scale = PxScale::from(pixel_height);
        
        println!("  Font created with scale: {:?}", scale);
        Ok(Self { font, scale })
    }
    
    pub fn render_char(&self, ch: char) -> Option<CharBitmap> {
        println!("  Rendering character: '{}'", ch);
        
        let glyph = self.font.glyph_id(ch).with_scale(self.scale);
        
        if let Some(outlined) = self.font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            let width = bounds.width().ceil() as usize;
            let height = bounds.height().ceil() as usize;
            
            println!("  Bounds: {:?} -> {}x{}", bounds, width, height);
            
            if width == 0 || height == 0 {
                println!("  Zero dimensions, using fallback");
                return self.create_fallback(ch);
            }
            
            let mut bitmap = vec![vec![false; width]; height];
            let mut pixels_set = 0;
            
            outlined.draw(|x, y, c| {
                let x = x as usize;
                let y = y as usize;
                
                if x < width && y < height {
                    if c > 0.1 {  // Lower threshold
                        bitmap[y][x] = true;
                        pixels_set += 1;
                    }
                }
            });
            
            println!("  Pixels set: {}/{} ({:.1}%)", 
                pixels_set, width * height,
                (pixels_set as f32 * 100.0) / (width * height) as f32);
            
            if pixels_set > 0 {
                return Some(CharBitmap {
                    width,
                    height,
                    bitmap,
                    advance: glyph.scaled(self.scale).h_advance(),
                });
            }
        }
        
        println!("  No outline or no pixels, using fallback");
        self.create_fallback(ch)
    }
    
    fn create_fallback(&self, ch: char) -> Option<CharBitmap> {
        println!("  Creating fallback for '{}'", ch);
        
        // Simple 8x8 bitmap
        let width = 8;
        let height = 8;
        let mut bitmap = vec![vec![false; width]; height];
        
        // Draw the character as a pattern
        match ch {
            'A' | 'a' => {
                // A shape
                for y in 1..7 {
                    bitmap[y][3] = true;
                }
                bitmap[2][2] = true; bitmap[2][4] = true;
                bitmap[1][3] = true;
            }
            'B' | 'b' => {
                // B shape
                for y in 1..7 { bitmap[y][1] = true; }
                bitmap[1][2] = true; bitmap[1][3] = true;
                bitmap[2][4] = true;
                bitmap[3][2] = true; bitmap[3][3] = true;
                bitmap[4][4] = true;
                bitmap[5][2] = true; bitmap[5][3] = true;
                bitmap[6][2] = true; bitmap[6][3] = true;
            }
            'E' | 'e' => {
                // E shape
                for y in 1..7 { bitmap[y][1] = true; }
                bitmap[1][2] = true; bitmap[1][3] = true; bitmap[1][4] = true;
                bitmap[3][2] = true; bitmap[3][3] = true;
                bitmap[6][2] = true; bitmap[6][3] = true; bitmap[6][4] = true;
            }
            'S' | 's' => {
                // S shape
                bitmap[1][2] = true; bitmap[1][3] = true; bitmap[1][4] = true;
                bitmap[2][1] = true;
                bitmap[3][2] = true; bitmap[3][3] = true; bitmap[3][4] = true;
                bitmap[4][4] = true;
                bitmap[5][1] = true; bitmap[5][2] = true; bitmap[5][3] = true;
            }
            '1' => {
                // 1
                for y in 1..7 { bitmap[y][3] = true; }
                bitmap[1][2] = true; bitmap[1][3] = true; bitmap[1][4] = true;
            }
            '2' => {
                // 2
                bitmap[1][2] = true; bitmap[1][3] = true; bitmap[1][4] = true;
                bitmap[2][1] = true; bitmap[2][4] = true;
                bitmap[3][4] = true;
                bitmap[4][3] = true;
                bitmap[5][2] = true;
                bitmap[6][1] = true; bitmap[6][2] = true; bitmap[6][3] = true; bitmap[6][4] = true;
            }
            '3' => {
                // 3
                bitmap[1][2] = true; bitmap[1][3] = true; bitmap[1][4] = true;
                bitmap[2][1] = true; bitmap[2][4] = true;
                bitmap[3][3] = true; bitmap[3][4] = true;
                bitmap[4][4] = true;
                bitmap[5][1] = true; bitmap[5][4] = true;
                bitmap[6][2] = true; bitmap[6][3] = true; bitmap[6][4] = true;
            }
            '4' => {
                // 4
                for y in 1..4 { bitmap[y][1] = true; }
                bitmap[3][2] = true; bitmap[3][3] = true; bitmap[3][4] = true;
                for y in 4..7 { bitmap[y][4] = true; }
            }
            '5' => {
                // 5
                bitmap[1][1] = true; bitmap[1][2] = true; bitmap[1][3] = true; bitmap[1][4] = true;
                bitmap[2][1] = true;
                bitmap[3][1] = true; bitmap[3][2] = true; bitmap[3][3] = true; bitmap[3][4] = true;
                bitmap[4][4] = true;
                bitmap[5][4] = true;
                bitmap[6][1] = true; bitmap[6][2] = true; bitmap[6][3] = true;
            }
            _ => {
                // Draw border for unknown
                for x in 0..width {
                    bitmap[0][x] = true;
                    bitmap[height-1][x] = true;
                }
                for y in 0..height {
                    bitmap[y][0] = true;
                    bitmap[y][width-1] = true;
                }
                // Diagonal
                for i in 0..width.min(height) {
                    bitmap[i][i] = true;
                }
            }
        }
        
        Some(CharBitmap {
            width,
            height,
            bitmap,
            advance: 8.0,
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