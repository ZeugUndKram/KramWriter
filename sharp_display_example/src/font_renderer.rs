use rusttype::{Font, Scale, point};
use anyhow::{Result, anyhow};
use image::{GrayImage, Luma};
use std::convert::TryFrom;

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
        let h_metrics = glyph.h_metrics();
        
        // Position at origin
        let glyph = glyph.positioned(point(0.0, 0.0));
        
        // Get bounding box
        let bb = glyph.pixel_bounding_box()?;
        let width = bb.width() as u32;
        let height = bb.height() as u32;
        
        println!("  Bounding box: {:?} -> {}x{}", bb, width, height);
        
        if width == 0 || height == 0 {
            println!("  Zero dimensions, using fallback");
            return self.create_fallback(ch);
        }
        
        // Create image buffer
        let mut image = GrayImage::new(width, height);
        
        // Draw glyph to image
        glyph.draw(|x, y, v| {
            let x = x as i32 - bb.min.x;
            let y = y as i32 - bb.min.y;
            
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                // Convert alpha to grayscale (0-255)
                let value = (v * 255.0) as u8;
                image.put_pixel(x as u32, y as u32, Luma([value]));
            }
        });
        
        // Convert to boolean bitmap with threshold
        let threshold = 128u8;
        let mut bitmap = vec![vec![false; width as usize]; height as usize];
        let mut pixels_set = 0;
        
        for y in 0..height {
            for x in 0..width {
                let pixel_value = image.get_pixel(x, y).0[0];
                if pixel_value > threshold {
                    bitmap[y as usize][x as usize] = true;
                    pixels_set += 1;
                }
            }
        }
        
        println!("  Pixels set: {}/{} ({:.1}%)", 
            pixels_set, width * height,
            (pixels_set as f32 * 100.0) / (width * height) as f32);
        
        // If no pixels, show what we got
        if pixels_set == 0 {
            println!("  DEBUG: First few pixel values:");
            for y in 0..height.min(5) {
                for x in 0..width.min(5) {
                    let val = image.get_pixel(x, y).0[0];
                    print!("{:3} ", val);
                }
                println!();
            }
            return self.create_fallback(ch);
        }
        
        Some(CharBitmap {
            width: width as usize,
            height: height as usize,
            bitmap,
            advance: h_metrics.advance_width,
        })
    }
    
    fn create_fallback(&self, ch: char) -> Option<CharBitmap> {
        println!("  Using fallback for '{}'", ch);
        
        // Create a simple 8x8 bitmap showing the character
        let width = 8;
        let height = 8;
        let mut bitmap = vec![vec![false; width]; height];
        
        // Draw border
        for x in 0..width {
            bitmap[0][x] = true;
            bitmap[height-1][x] = true;
        }
        for y in 0..height {
            bitmap[y][0] = true;
            bitmap[y][width-1] = true;
        }
        
        // Try to represent the character crudely
        match ch {
            'A' | 'a' => {
                // Draw an A shape
                bitmap[1][3] = true;
                bitmap[2][2] = true; bitmap[2][4] = true;
                bitmap[3][1] = true; bitmap[3][5] = true;
                bitmap[4][1] = true; bitmap[4][2] = true; bitmap[4][3] = true; 
                bitmap[4][4] = true; bitmap[4][5] = true;
            }
            'B' | 'b' => {
                // Draw a B shape
                for y in 1..7 { bitmap[y][1] = true; }
                bitmap[1][2] = true; bitmap[1][3] = true;
                bitmap[2][4] = true;
                bitmap[3][2] = true; bitmap[3][3] = true;
                bitmap[4][4] = true;
                bitmap[5][2] = true; bitmap[5][3] = true;
                bitmap[6][2] = true; bitmap[6][3] = true;
            }
            'C' | 'c' => {
                // Draw a C shape
                bitmap[1][2] = true; bitmap[1][3] = true; bitmap[1][4] = true;
                bitmap[2][1] = true;
                bitmap[3][1] = true;
                bitmap[4][1] = true;
                bitmap[5][1] = true;
                bitmap[6][2] = true; bitmap[6][3] = true; bitmap[6][4] = true;
            }
            '1' => {
                // Draw 1
                for y in 1..7 { bitmap[y][3] = true; }
                bitmap[1][2] = true; bitmap[1][3] = true; bitmap[1][4] = true;
                bitmap[6][2] = true; bitmap[6][3] = true; bitmap[6][4] = true;
            }
            '2' => {
                // Draw 2
                bitmap[1][2] = true; bitmap[1][3] = true; bitmap[1][4] = true;
                bitmap[2][1] = true; bitmap[2][5] = true;
                bitmap[3][5] = true;
                bitmap[4][4] = true;
                bitmap[5][3] = true;
                bitmap[6][1] = true; bitmap[6][2] = true; bitmap[6][3] = true;
                bitmap[6][4] = true; bitmap[6][5] = true;
            }
            '3' => {
                // Draw 3
                bitmap[1][2] = true; bitmap[1][3] = true; bitmap[1][4] = true;
                bitmap[2][1] = true; bitmap[2][5] = true;
                bitmap[3][5] = true;
                bitmap[4][3] = true; bitmap[4][4] = true;
                bitmap[5][5] = true;
                bitmap[6][1] = true; bitmap[6][2] = true; bitmap[6][3] = true;
                bitmap[6][4] = true;
            }
            _ => {
                // Just draw diagonal for unknown chars
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