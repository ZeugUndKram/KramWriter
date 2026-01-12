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
    println!("Initializing display {}x{}...", width, height);
    
    let mut driver = MemoryDisplay::new(  // Add 'mut' here
        Bus::Spi0,
        SlaveSelect::Ss0,
        6,  // GPIO 6
        width,
        height as u8,
    )?;
    
    let buffer = MemoryDisplayBuffer::new(width, height as u8);
    
    // Clear display immediately
    let mut temp_buffer = buffer.clone();
    temp_buffer.fill(Pixel::White);
    driver.update(&temp_buffer)?;
    
    Ok(Self {
        driver,
        buffer: temp_buffer,
        width,
        height,
        font_renderer: None,
        font_size: 24.0,
    })
    }
    
    pub fn load_font(&mut self, font_path: &str, size: f32) -> Result<()> {
        println!("Loading font: {}", font_path);
        let renderer = FontRenderer::from_file(font_path, size)?;
        self.font_renderer = Some(renderer);
        self.font_size = size;
        println!("Font loaded (size: {}px)", size);
        Ok(())
    }
    
    pub fn clear(&mut self) -> Result<()> {
        self.buffer.fill(Pixel::White);
        self.driver.update(&self.buffer)?;
        Ok(())
    }
    
    pub fn update(&mut self) -> Result<()> {
        self.driver.update(&self.buffer)?;
        Ok(())
    }
    
    pub fn draw_char(&mut self, x: usize, y: usize, ch: char) -> Result<usize> {
        if let Some(renderer) = &self.font_renderer {
            if let Some(char_bitmap) = renderer.render_char(ch) {
                // Draw the character
                for (row_idx, row) in char_bitmap.bitmap.iter().enumerate() {
                    for (col_idx, &pixel) in row.iter().enumerate() {
                        if pixel {
                            let px = x + col_idx;
                            let py = y + row_idx;
                            
                            if px < self.width && py < self.height {
                                self.buffer.set_pixel(px, py as u8, Pixel::Black);
                            }
                        }
                    }
                }
                return Ok(char_bitmap.width + 1); // Return width + spacing
            }
        }
        
        // Fallback: draw a box for missing characters
        self.draw_fallback_char(x, y)?;
        Ok(8) // Return fallback width
    }
    
    fn draw_fallback_char(&mut self, x: usize, y: usize) -> Result<()> {
        // Draw a simple 6x8 box
        for dy in 0..8.min(self.height - y) {
            for dx in 0..6.min(self.width - x) {
                if dx == 0 || dx == 5 || dy == 0 || dy == 7 {
                    self.buffer.set_pixel(x + dx, (y + dy) as u8, Pixel::Black);
                }
            }
        }
        Ok(())
    }
    
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) -> Result<()> {
        let mut cursor_x = x;
        let mut cursor_y = y;
        
        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = x;
                cursor_y += self.font_size as usize + 2;
                continue;
            }
            
            // Check bounds
            if cursor_y >= self.height {
                break; // No more vertical space
            }
            
            if cursor_x >= self.width {
                cursor_x = x;
                cursor_y += self.font_size as usize + 2;
                if cursor_y >= self.height {
                    break;
                }
            }
            
            let char_width = self.draw_char(cursor_x, cursor_y, ch)?;
            cursor_x += char_width;
        }
        
        Ok(())
    }
    
    pub fn buffer_mut(&mut self) -> &mut MemoryDisplayBuffer {
        &mut self.buffer
    }
}