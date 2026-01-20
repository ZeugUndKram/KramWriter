use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;

const MENU_OPTIONS: [&str; 5] = [
    "Write",
    "Learn", 
    "Zeugtris",  // Updated to match the new Zeugtris option
    "Settings",
    "Credits",
];

const SPACING_TOP_TO_MAIN: i32 = -10;           // Space between top image and main image (can be negative)
const SPACING_MAIN_TO_BOTTOM: i32 = 10;        // Space between main image and bottom image (can be negative)
const SPACING_TOP_TO_FARTOP: i32 = 30;        // Space between far top image and top image (can be negative)
const SPACING_BOTTOM_TO_FARBOTTOM: i32 = 45;  // Space between bottom image and far bottom image (can be negative)

pub struct MenuPage {
    current_index: usize,
    images_cache: Vec<Vec<Option<(Vec<Pixel>, usize, usize)>>>,
}

impl MenuPage {
    pub fn new() -> Result<Self> {
        let mut images_cache = Vec::new();
        
        for option in MENU_OPTIONS.iter() {
            let mut option_images = Vec::new();
            
            for suffix in 0..3 {
                let path = format!("/home/kramwriter/KramWriter/assets/menu/{}_{}.bmp", option, suffix);
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
    
    fn draw_image_at(&self, display: &mut SharpDisplay, image_data: Option<&(Vec<Pixel>, usize, usize)>, y_pos: i32) {
        if let Some((pixels, width, height)) = image_data {
            let start_x = (400 - width) / 2;
            
            for y in 0..*height {
                let screen_y = y_pos + y as i32;
                if screen_y >= 240 || screen_y < 0 { continue; }
                
                for x in 0..*width {
                    let screen_x = start_x + x;
                    if screen_x < 400 {
                        let pixel = pixels[y * width + x];
                        display.draw_pixel(screen_x, screen_y as usize, pixel);
                    }
                }
            }
        }
    }
}

impl Page for MenuPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        let main_image_data = self.images_cache[self.current_index].get(0).and_then(|x| x.as_ref());
        
        if let Some((_, _, height)) = main_image_data {
            let center_y = (240 - height) / 2;
            let h = *height as i32;
            let cy = center_y as i32;
            
            // Draw images above the main one (if any)
            if self.current_index >= 2 {
                // Second previous option (2 above) with suffix 2
                let second_prev_image = self.images_cache[self.current_index - 2].get(2).and_then(|x| x.as_ref());
                let second_prev_y = cy - h - SPACING_TOP_TO_MAIN - SPACING_TOP_TO_FARTOP;
                self.draw_image_at(display, second_prev_image, second_prev_y);
            }
            
            if self.current_index >= 1 {
                // Previous option (1 above) with suffix 1
                let prev_image = self.images_cache[self.current_index - 1].get(1).and_then(|x| x.as_ref());
                let prev_y = cy - h - SPACING_TOP_TO_MAIN;
                self.draw_image_at(display, prev_image, prev_y);
            }
            
            // Draw main image
            self.draw_image_at(display, main_image_data, cy);
            
            // Draw images below the main one (if any)
            if self.current_index + 1 < MENU_OPTIONS.len() {
                let next_image = self.images_cache[self.current_index + 1].get(1).and_then(|x| x.as_ref());
                let next_y = cy + h + SPACING_MAIN_TO_BOTTOM;
                self.draw_image_at(display, next_image, next_y);
            }
            
            if self.current_index + 2 < MENU_OPTIONS.len() {
                let second_next_image = self.images_cache[self.current_index + 2].get(2).and_then(|x| x.as_ref());
                let second_next_y = cy + h + SPACING_MAIN_TO_BOTTOM + SPACING_BOTTOM_TO_FARBOTTOM;
                self.draw_image_at(display, second_next_image, second_next_y);
            }
        } else {
            display.draw_text(150, 100, "NO IMAGE");
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
            Key::Char('\n') => {
                match self.current_index {
                    0 => Ok(Some(PageId::WriteMenu)),
                    2 => Ok(Some(PageId::ZeugtrisMenu)),  // Zeugtris option - index 2
                    _ => Ok(Some(PageId::Logo)),
                }
            }
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