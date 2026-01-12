// src/display.rs
use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Bus, SlaveSelect};
use crate::font_renderer::{FontRenderer, CharBitmap};
use anyhow::{Result, anyhow};

pub struct Display {
    driver: MemoryDisplay,
    buffer: MemoryDisplayBuffer,
    width: usize,
    height: usize,
    font_renderer: Option<FontRenderer>,
    font_size: f32,
}

impl Display {
    pub fn new(width: usize, height: usize) -> Result<Self> {
        let driver = MemoryDisplay::new(
            Bus::Spi0,
            SlaveSelect::Ss0,
            6,  // CS pin (adjust based on your wiring)
            width,
            height as u8,
        )?;
        
        let buffer = MemoryDisplayBuffer::new(width, height as u8);
        
        Ok(Self {
            driver,
            buffer,
            width,
            height,
            font_renderer: None,
            font_size: 16.0,  // Default font size
        })
    }
    
    /// Load a font from a file
    pub fn load_font(&mut self, font_path: &str, size: f32) -> Result<()> {
        let renderer = FontRenderer::from_file(font_path, size)?;
        self.font_renderer = Some(renderer);
        self.font_size = size;
        Ok(())
    }
    
    /// Clear the display (fill with white)
    pub fn clear(&mut self) -> Result<()> {
        self.buffer.fill(Pixel::White);
        self.driver.update(&self.buffer)?;
        Ok(())
    }
    
    /// Update the display with current buffer
    pub fn update(&mut self) -> Result<()> {
        self.driver.update(&self.buffer)?;
        Ok(())
    }
    
    /// Draw a character at position (x, y) in pixels
    pub fn draw_char(&mut self, x: usize, y: usize, ch: char) -> Result<()> {
        if let Some(renderer) = &self.font_renderer {
            if let Some(char_bitmap) = renderer.render_char(ch) {
                // Get the thresholded bitmap
                let bitmap = char_bitmap.to_bitmap(128); // 50% threshold
                
                // Draw each pixel
                for (row_idx, row) in bitmap.iter().enumerate() {
                    for (col_idx, &pixel) in row.iter().enumerate() {
                        if pixel {
                            let px = x + col_idx;
                            let py = y + row_idx;
                            
                            // Check bounds
                            if px < self.width && py < self.height {
                                self.buffer.set_pixel(px, py as u8, Pixel::Black);
                            }
                        }
                    }
                }
                
                // Return the character width for cursor movement
                return Ok(());
            }
        }
        
        // Fallback: draw a placeholder if no font loaded
        self.draw_placeholder_char(x, y)?;
        Ok(())
    }
    
    /// Draw text starting at position (x, y)
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) -> Result<()> {
        let mut cursor_x = x;
        let mut cursor_y = y;
        
        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = x;
                cursor_y += self.font_size as usize + 2; // Line spacing
                continue;
            }
            
            // Check if character would go off screen
            if cursor_x + self.font_size as usize > self.width {
                cursor_x = x;
                cursor_y += self.font_size as usize + 2;
            }
            
            // Check if we've run out of vertical space
            if cursor_y + self.font_size as usize > self.height {
                break;
            }
            
            self.draw_char(cursor_x, cursor_y, ch)?;
            
            // Move cursor for next character
            cursor_x += self.font_size as usize / 2 + 1; // Approximate character width
        }
        
        Ok(())
    }
    
    /// Draw a simple placeholder character (6x8 pixels)
    fn draw_placeholder_char(&mut self, x: usize, y: usize) -> Result<()> {
        // Simple 6x8 box as placeholder
        for dy in 0..8 {
            for dx in 0..6 {
                let px = x + dx;
                let py = y + dy;
                
                if px < self.width && py < self.height {
                    // Draw border
                    if dx == 0 || dx == 5 || dy == 0 || dy == 7 {
                        self.buffer.set_pixel(px, py as u8, Pixel::Black);
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Draw a cursor (vertical line)
    pub fn draw_cursor(&mut self, x: usize, y: usize, height: usize) -> Result<()> {
        for dy in 0..height {
            let px = x;
            let py = y + dy;
            
            if px < self.width && py < self.height {
                self.buffer.set_pixel(px, py as u8, Pixel::Black);
                if px + 1 < self.width {
                    self.buffer.set_pixel(px + 1, py as u8, Pixel::Black);
                }
            }
        }
        Ok(())
    }
    
    /// Get display dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    
    /// Get current font size
    pub fn font_size(&self) -> f32 {
        self.font_size
    }
}