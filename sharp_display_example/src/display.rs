// src/display.rs - COMPLETE FILE
use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Bus, SlaveSelect};
use anyhow::Result;

const WIDTH: usize = 400;
const HEIGHT: usize = 240;

pub struct SharpDisplay {
    inner: MemoryDisplay,
    buffer: MemoryDisplayBuffer,
}

impl SharpDisplay {
    pub fn new(cs_pin: u8) -> Result<Self> {
        let mut inner = MemoryDisplay::new(
            Bus::Spi0,
            SlaveSelect::Ss0,
            cs_pin,
            WIDTH,
            HEIGHT as u8,
        )?;
        
        inner.clear()?;
        
        let buffer = MemoryDisplayBuffer::new(WIDTH, HEIGHT as u8);
        
        Ok(Self { inner, buffer })
    }
    
    pub fn clear(&mut self) -> Result<()> {
        self.buffer.fill(Pixel::White);
        self.inner.clear()?;
        Ok(())
    }
    
    pub fn update(&mut self) -> Result<()> {
        self.inner.update(&self.buffer)?;
        Ok(())
    }
    
    pub fn draw_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        if x < WIDTH && y < HEIGHT {
            self.buffer.set_pixel(x, y as u8, pixel);
        }
    }
    
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        for (i, c) in text.chars().enumerate() {
            if x + i * 6 < WIDTH {
                self.draw_char(x + i * 6, y, c);
            }
        }
    }
    
    fn draw_char(&mut self, x: usize, y: usize, c: char) {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | ' ' | ':' | '.' | '-' => {
                for dy in 2..6 {
                    for dx in 1..5 {
                        self.draw_pixel(x + dx, y + dy, Pixel::Black);
                    }
                }
            }
            _ => {}
        }
    }
}