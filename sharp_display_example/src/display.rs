// src/display.rs
use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Bus, SlaveSelect};
use crate::font_renderer::FontRenderer;
use anyhow::Result;

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
        println!("Initializing Sharp Memory Display {}x{}...", width, height);
        
        // Initialize the display FIRST
        let driver = MemoryDisplay::new(
            Bus::Spi0,
            SlaveSelect::Ss0,
            6,  // GPIO 6 as you instructed
            width,
            height as u8,
        )?;
        
        println!("Display initialized. Creating buffer...");
        
        // Create buffer with correct size
        let buffer = MemoryDisplayBuffer::new(width, height as u8);
        
        // Clear the buffer immediately
        let mut buffer_copy = buffer.clone();
        buffer_copy.fill(Pixel::White);
        
        println!("Buffer created. Updating display...");
        
        // Update display with cleared buffer
        driver.update(&buffer_copy)?;
        
        println!("Display ready!");
        
        Ok(Self {
            driver,
            buffer: buffer_copy,
            width,
            height,
            font_renderer: None,
            font_size: 16.0,
        })
    }
    
    /// Load a font from a file
    pub fn load_font(&mut self, font_path: &str, size: f32) -> Result<()> {
        println!("Loading font from: {}", font_path);
        let renderer = FontRenderer::from_file(font_path, size)?;
        self.font_renderer = Some(renderer);
        self.font_size = size;
        println!("Font loaded successfully (size: {}px)", size);
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
                return Ok(());
            }
        }
        
        // Fallback: draw a placeholder if no font loaded
        self.draw_placeholder_char(x, y)?;
        Ok(())
    }
    
    /// Draw text starting at position (x, y)
    pub fn draw_text(&mut self, mut x: usize, mut y: usize, text: &str) -> Result<()> {
        println!("Drawing text at ({}, {}): '{}'", x, y, text);
        
        let start_x = x;
        
        for ch in text.chars() {
            if ch == '\n' {
                x = start_x;
                y += self.font_size as usize + 2; // Line spacing
                continue;
            }
            
            // Check if character would go off screen
            if x + self.font_size as usize > self.width {
                x = start_x;
                y += self.font_size as usize + 2;
            }
            
            // Check if we've run out of vertical space
            if y + self.font_size as usize > self.height {
                println!("Out of vertical space!");
                break;
            }
            
            self.draw_char(x, y, ch)?;
            
            // Move cursor for next character (approximate based on font size)
            x += (self.font_size * 0.6) as usize; // Rough estimate for character width
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