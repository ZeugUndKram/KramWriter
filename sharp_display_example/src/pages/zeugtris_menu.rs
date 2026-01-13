use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;

pub struct ZeugtrisMenuPage {
    logo_data: Option<(Vec<Pixel>, usize, usize)>,
}

impl ZeugtrisMenuPage {
    pub fn new() -> Result<Self> {
        let path = "/home/kramwriter/KramWriter/assets/zeugtris_logo.bmp";
        
        let logo_data = match std::fs::read(path) {
            Ok(data) => Self::parse_bmp(&data),
            Err(_) => None,
        };
        
        Ok(Self { logo_data })
    }
    
    fn parse_bmp(data: &[u8]) -> Option<(Vec<Pixel>, usize, usize)> {
        if data.len() < 54 { return None; }
        if data[0] != 0x42 || data[1] != 0x4D { return None; }
        
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]) as usize;
        let data_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        
        if data_offset >= data.len() { return None; }
        
        let mut pixels = Vec::with_capacity(width * height);
        
        match bits_per_pixel {
            32 => {
                let row_bytes = width * 4;
                for y in 0..height {
                    let row_start = data_offset + (height - 1 - y) * row_bytes;
                    for x in 0..width {
                        let pixel_start = row_start + x * 4;
                        if pixel_start + 3 >= data.len() {
                            pixels.push(Pixel::White);
                            continue;
                        }
                        let b = data[pixel_start] as u32;
                        let g = data[pixel_start + 1] as u32;
                        let r = data[pixel_start + 2] as u32;
                        let a = data[pixel_start + 3] as u32;
                        
                        let luminance = (r * 299 + g * 587 + b * 114) / 1000;
                        let pixel = if a < 128 || luminance > 128 { 
                            Pixel::White 
                        } else { 
                            Pixel::Black 
                        };
                        pixels.push(pixel);
                    }
                }
            }
            24 => {
                let row_bytes = ((width * 3 + 3) / 4) * 4;
                for y in 0..height {
                    let row_start = data_offset + (height - 1 - y) * row_bytes;
                    for x in 0..width {
                        let pixel_start = row_start + x * 3;
                        if pixel_start + 2 >= data.len() {
                            pixels.push(Pixel::White);
                            continue;
                        }
                        let b = data[pixel_start] as u32;
                        let g = data[pixel_start + 1] as u32;
                        let r = data[pixel_start + 2] as u32;
                        
                        let luminance = (r * 299 + g * 587 + b * 114) / 1000;
                        let pixel = if luminance > 128 { Pixel::White } else { Pixel::Black };
                        pixels.push(pixel);
                    }
                }
            }
            1 => {
                let row_bytes = ((width + 31) / 32) * 4;
                for y in 0..height {
                    let row_start = data_offset + (height - 1 - y) * row_bytes;
                    for x in 0..width {
                        if row_start + (x / 8) >= data.len() {
                            pixels.push(Pixel::White);
                            continue;
                        }
                        let byte = data[row_start + (x / 8)];
                        let bit = 7 - (x % 8);
                        let pixel = if (byte >> bit) & 1 == 1 { Pixel::Black } else { Pixel::White };
                        pixels.push(pixel);
                    }
                }
            }
            _ => return None,
        }
        
        Some((pixels, width, height))
    }
    
    fn draw_image_centered(&self, display: &mut SharpDisplay, y: usize) {
        if let Some((pixels, width, height)) = &self.logo_data {
            let start_x = (400usize.saturating_sub(*width)) / 2;
            let start_y = y;
            
            for py in 0..height.min(240 - y) {
                for px in 0..width.min(400) {
                    let pixel = pixels[py * width + px];
                    if pixel == Pixel::Black {
                        display.draw_pixel(start_x + px, start_y + py, pixel);
                    }
                }
            }
        }
    }
}

impl Page for ZeugtrisMenuPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        if let Some((_, _, height)) = &self.logo_data {
            // Center logo vertically
            let start_y = (240 - height) / 2;
            self.draw_image_centered(display, start_y);
            
            // Draw "Press Enter" text below logo
            let text_y = start_y + height + 20;
            let text = "PRESS ENTER";
            let text_width = text.len() * 6;
            let text_x = (400 - text_width) / 2;
            
            if text_y < 240 {
                display.draw_text(text_x, text_y, text);
            }
        } else {
            // Fallback: draw text if no logo
            display.draw_text(150, 100, "ZEUGTRIS");
            display.draw_text(130, 120, "PRESS ENTER");
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
            Key::Char('\n') => Ok(Some(PageId::Zeugtris)),
            Key::Esc => Ok(Some(PageId::Menu)),
            _ => Ok(None),
        }
    }
}