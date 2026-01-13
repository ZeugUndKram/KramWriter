use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;

const MENU_OPTIONS: [&str; 5] = [
    "Write",
    "Learn", 
    "Zeugtris",
    "Settings",
    "Credits",
];

pub struct MenuPage {
    current_index: usize,
    images_cache: Vec<Vec<Option<(Vec<Pixel>, usize, usize)>>>,
}

impl MenuPage {
    pub fn new() -> Result<Self> {
        let mut images_cache = Vec::new();
        
        // Preload all images (0, 1, 2 for each option)
        for option in MENU_OPTIONS.iter() {
            let mut option_images = Vec::new();
            
            for suffix in 0..3 {
                let path = format!("/home/kramwriter/KramWriter/assets/{}_{}.bmp", option, suffix);
                match std::fs::read(&path) {
                    Ok(data) => {
                        option_images.push(Self::parse_bmp(&data));
                    }
                    Err(_) => {
                        option_images.push(None);
                    }
                }
            }
            
            images_cache.push(option_images);
        }
        
        Ok(Self {
            current_index: 0,
            images_cache,
        })
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
            _ => return None,
        }
        
        Some((pixels, width, height))
    }
    
    fn draw_image(&self, display: &mut SharpDisplay, image_data: Option<&(Vec<Pixel>, usize, usize)>, x_offset: i32, y_offset: i32) {
        if let Some((pixels, width, height)) = image_data {
            let start_x = ((400 - width) / 2) as i32 + x_offset;
            let start_y = ((240 - height) / 2) as i32 + y_offset;
            
            for y in 0..*height {
                for x in 0..*width {
                    let screen_x = start_x + x as i32;
                    let screen_y = start_y + y as i32;
                    
                    if screen_x >= 0 && screen_x < 400 && screen_y >= 0 && screen_y < 240 {
                        let pixel = pixels[y * width + x];
                        display.draw_pixel(screen_x as usize, screen_y as usize, pixel);
                    }
                }
            }
        }
    }
}

impl Page for MenuPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        // Draw centered image (current_index, suffix 0)
        let center_image = self.images_cache[self.current_index].get(0).and_then(|x| x.as_ref());
        self.draw_image(display, center_image, 0, 0);
        
        // Draw next image below (current_index + 1, suffix 1) if exists
        if self.current_index + 1 < MENU_OPTIONS.len() {
            let next_image = self.images_cache[self.current_index + 1].get(1).and_then(|x| x.as_ref());
            self.draw_image(display, next_image, 0, 120);
        }
        
        // Draw second next image below (current_index + 2, suffix 2) if exists
        if self.current_index + 2 < MENU_OPTIONS.len() {
            let second_next_image = self.images_cache[self.current_index + 2].get(2).and_then(|x| x.as_ref());
            self.draw_image(display, second_next_image, 0, 180);
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
            Key::Char('\n') => Ok(Some(PageId::Logo)),
            Key::Up => {
                if self.current_index > 0 {
                    self.current_index -= 1;
                }
                Ok(None)
            }
            Key::Down => {
                if self.current_index < MENU_OPTIONS.len() - 1 {
                    self.current_index += 1;
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}