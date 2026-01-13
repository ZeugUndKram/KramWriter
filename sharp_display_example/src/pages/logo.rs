// src/pages/logo.rs
use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;

pub struct LogoPage {
    logo_data: Option<Vec<Pixel>>,
    logo_width: usize,
    logo_height: usize,
}

impl LogoPage {
    pub fn new() -> Result<Self> {
        // Try to load logo.bmp
        let path = "/home/kramwriter/KramWriter/assets/logo.bmp";
        let (pixels, width, height) = match std::fs::read(path) {
            Ok(data) => Self::parse_bmp(&data).unwrap_or_else(|| (vec![], 0, 0)),
            Err(_) => (vec![], 0, 0),
        };
        
        Ok(Self {
            logo_data: if !pixels.is_empty() { Some(pixels) } else { None },
            logo_width: width,
            logo_height: height,
        })
    }
    
    fn parse_bmp(data: &[u8]) -> Option<(Vec<Pixel>, usize, usize)> {
        // Simple BMP parser for 1-bit monochrome BMP
        if data.len() < 54 { return None; }
        
        // Check BMP signature
        if data[0] != 0x42 || data[1] != 0x4D { return None; }
        
        // Get width and height from header (little-endian)
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]) as usize;
        
        // Only handle 1-bit (monochrome) BMP for now
        if bits_per_pixel != 1 { return None; }
        
        let data_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        
        let mut pixels = Vec::with_capacity(width * height);
        
        // Calculate row size in bytes (padded to 4-byte boundary)
        let row_bytes = ((width + 31) / 32) * 4;
        
        for y in 0..height {
            let row_start = data_offset + (height - 1 - y) * row_bytes; // BMP is bottom-up
            
            for x in 0..width {
                let byte_index = row_start + (x / 8);
                if byte_index >= data.len() { 
                    pixels.push(Pixel::White);
                    continue;
                }
                
                let bit_position = 7 - (x % 8);
                let bit = (data[byte_index] >> bit_position) & 1;
                
                pixels.push(if bit == 1 { Pixel::Black } else { Pixel::White });
            }
        }
        
        Some((pixels, width, height))
    }
}

impl Page for LogoPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        if let Some(logo_pixels) = &self.logo_data {
            // Center the logo on screen
            let start_x = (400usize.saturating_sub(self.logo_width)) / 2;
            let start_y = (240usize.saturating_sub(self.logo_height)) / 2;
            
            for y in 0..self.logo_height.min(240) {
                for x in 0..self.logo_width.min(400) {
                    let pixel = logo_pixels[y * self.logo_width + x];
                    display.draw_pixel(start_x + x, start_y + y, pixel);
                }
            }
        } else {
            // Fallback if no logo
            display.draw_text(150, 100, "NO LOGO");
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
            Key::Char('\n') => Ok(Some(PageId::Menu)),
            _ => Ok(None),
        }
    }
}