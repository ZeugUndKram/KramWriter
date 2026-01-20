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
        let path = "/home/kramwriter/KramWriter/assets/logo/logo.bmp";
        println!("Loading logo from: {}", path);
        
        match std::fs::read(path) {
            Ok(data) => {
                println!("Loaded {} bytes", data.len());
                match Self::parse_bmp(&data) {
                    Some((pixels, width, height)) => {
                        println!("Parsed BMP: {}x{}, {} pixels", width, height, pixels.len());
                        Ok(Self {
                            logo_data: Some(pixels),
                            logo_width: width,
                            logo_height: height,
                        })
                    }
                    None => {
                        println!("Failed to parse BMP");
                        Ok(Self {
                            logo_data: None,
                            logo_width: 0,
                            logo_height: 0,
                        })
                    }
                }
            }
            Err(e) => {
                println!("Failed to read logo: {}", e);
                Ok(Self {
                    logo_data: None,
                    logo_width: 0,
                    logo_height: 0,
                })
            }
        }
    }
    
    fn parse_bmp(data: &[u8]) -> Option<(Vec<Pixel>, usize, usize)> {
        if data.len() < 54 { return None; }
        if data[0] != 0x42 || data[1] != 0x4D { return None; }
        
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]) as usize;
        let data_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        
        println!("BMP: {}x{}, {} bpp, offset: {}", width, height, bits_per_pixel, data_offset);
        
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
                        let alpha = a;
                        
                        let pixel = if alpha < 128 {
                            Pixel::White
                        } else if luminance > 128 {
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
            _ => {
                println!("Unsupported BMP format: {} bpp", bits_per_pixel);
                return None;
            }
        }
        
        Some((pixels, width, height))
    }
}

impl Page for LogoPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        if let Some(logo_pixels) = &self.logo_data {
            let start_x = (400usize.saturating_sub(self.logo_width)) / 2;
            let start_y = (240usize.saturating_sub(self.logo_height)) / 2;
            
            for y in 0..self.logo_height.min(240) {
                for x in 0..self.logo_width.min(400) {
                    let pixel = logo_pixels[y * self.logo_width + x];
                    display.draw_pixel(start_x + x, start_y + y, pixel);
                }
            }
        } else {
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