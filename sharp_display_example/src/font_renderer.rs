use rusttype::{Font, Scale, point};
use image::{GrayImage, Luma};
use std::path::Path;
use anyhow::{Result, anyhow};

pub struct FontRenderer {
    font: Font<'static>,
    scale: Scale,
    line_height: f32,
}

impl FontRenderer {
    /// Load a font from a file
    pub fn from_file(path: &str, pixel_height: f32) -> Result<Self> {
        // Read font file
        let font_data = std::fs::read(path)
            .map_err(|e| anyhow!("Failed to read font file {}: {}", path, e))?;
        
        // Parse font
        let font = Font::try_from_vec(font_data)
            .ok_or_else(|| anyhow!("Failed to parse font file {}", path))?;
        
        // Calculate scale based on desired pixel height
        let scale = Scale::uniform(pixel_height);
        
        // Measure line height (using 'M' as reference)
        let v_metrics = font.v_metrics(scale);
        let line_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        
        Ok(Self {
            font,
            scale,
            line_height,
        })
    }
    
    /// Render a character to a bitmap
    pub fn render_char(&self, ch: char) -> Option<CharBitmap> {
        let glyph = self.font.glyph(ch).scaled(self.scale);
        let h_metrics = glyph.h_metrics();
        
        // Get pixel-accurate bounding box
        let bbox = glyph.exact_bounding_box()?;
        
        // Position the glyph
        let glyph = glyph.positioned(point(0.0, 0.0));
        
        // Create image buffer for the glyph
        let width = bbox.width().ceil() as u32;
        let height = bbox.height().ceil() as u32;
        
        let mut image = GrayImage::new(width, height);
        
        // Draw the glyph
        glyph.draw(|x, y, v| {
            let x = x as i32;
            let y = y as i32;
            
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                // Convert alpha to grayscale (0 = transparent, 255 = black)
                let value = (v * 255.0) as u8;
                image.put_pixel(x as u32, y as u32, Luma([value]));
            }
        });
        
        Some(CharBitmap {
            width: width as usize,
            height: height as usize,
            data: image,
            left_bearing: h_metrics.left_side_bearing,
            advance_width: h_metrics.advance_width,
        })
    }
    
    /// Get line height in pixels
    pub fn line_height(&self) -> f32 {
        self.line_height
    }
    
    /// Get ascent (distance from baseline to top of tallest character)
    pub fn ascent(&self) -> f32 {
        self.font.v_metrics(self.scale).ascent
    }
    
    /// Get descent (distance from baseline to bottom of lowest character)
    pub fn descent(&self) -> f32 {
        self.font.v_metrics(self.scale).descent
    }
}

pub struct CharBitmap {
    pub width: usize,
    pub height: usize,
    pub data: GrayImage,
    pub left_bearing: f32,
    pub advance_width: f32,
}

impl CharBitmap {
    /// Convert to a simple thresholded bitmap (1-bit per pixel)
    pub fn to_bitmap(&self, threshold: u8) -> Vec<Vec<bool>> {
        let mut bitmap = vec![vec![false; self.width]; self.height];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.data.get_pixel(x as u32, y as u32).0[0];
                bitmap[y][x] = pixel > threshold;
            }
        }
        
        bitmap
    }
}