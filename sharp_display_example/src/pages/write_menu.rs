use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;

pub struct WriteMenuPage {
    font_bitmap: Option<(Vec<Pixel>, usize, usize)>,
    font_char_width: usize,
    font_char_height: usize,
    chars_per_row: usize,
    current_text: String,
}

impl WriteMenuPage {
    pub fn new() -> Result<Self> {
        let font_path = "/home/kramwriter/KramWriter/fonts/bebas24.bmp";
        println!("Loading font from: {}", font_path);
        
        let font_bitmap = match std::fs::read(font_path) {
            Ok(data) => {
                println!("Font loaded: {} bytes", data.len());
                let result = Self::parse_font_bmp(&data);
                if let Some((_, width, height)) = &result {
                    println!("Font dimensions: {}x{}", width, height);
                }
                result
            }
            Err(e) => {
                println!("Failed to load font: {}", e);
                None
            }
        };
        
        Ok(Self {
            font_bitmap,
            font_char_width: 30,
            font_char_height: 30,
            chars_per_row: 19,
            current_text: String::from("TEST"),
        })
    }
    
    fn parse_font_bmp(data: &[u8]) -> Option<(Vec<Pixel>, usize, usize)> {
        if data.len() < 54 { return None; }
        if data[0] != 0x42 || data[1] != 0x4D { return None; }
        
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]) as usize;
        let data_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        
        println!("BMP font: {}x{}, {} bpp, offset: {}", width, height, bits_per_pixel, data_offset);
        
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
                        
                        // INVERTED: Black text on white background in BMP -> White text on black screen
                        // So we need to invert: white in BMP becomes black on screen
                        let pixel = if alpha < 128 {
                            Pixel::White  // Transparent = white background
                        } else if luminance > 128 {
                            Pixel::Black  // White in BMP = black on screen (text)
                        } else {
                            Pixel::White  // Black in BMP = white on screen (background)
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
                        
                        // INVERTED for 24-bit
                        let pixel = if luminance > 128 {
                            Pixel::Black  // White in BMP = black on screen
                        } else {
                            Pixel::White  // Black in BMP = white on screen
                        };
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
                        // INVERTED for 1-bit: 1=black in BMP should be black on screen
                        // Actually 1-bit BMP: 1 = foreground (text), 0 = background
                        // We want text to be black on white screen
                        let pixel = if (byte >> bit) & 1 == 1 { 
                            Pixel::Black  // Text in BMP = black on screen
                        } else { 
                            Pixel::White  // Background in BMP = white on screen
                        };
                        pixels.push(pixel);
                    }
                }
            }
            _ => return None,
        }
        
        println!("Parsed {} pixels", pixels.len());
        Some((pixels, width, height))
    }
    
    fn get_char_index(c: char) -> usize {
        let printable_chars = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        match printable_chars.find(c) {
            Some(idx) => idx,
            None => {
                println!("Character '{}' not found in font, using space", c);
                0
            }
        }
    }
    
    fn draw_char(&self, display: &mut SharpDisplay, x: usize, y: usize, c: char) {
        if let Some((pixels, font_width, font_height)) = &self.font_bitmap {
            let char_index = Self::get_char_index(c);
            let chars_per_row = self.chars_per_row;
            let char_width = self.font_char_width;
            let char_height = self.font_char_height;
            
            let grid_x = char_index % chars_per_row;
            let grid_y = char_index / chars_per_row;
            
            let src_x = grid_x * char_width;
            let src_y = grid_y * char_height;
            
            if src_y + char_height > *font_height || src_x + char_width > *font_width {
                return;
            }
            
            for dy in 0..char_height {
                for dx in 0..char_width {
                    let src_pixel_x = src_x + dx;
                    let src_pixel_y = src_y + dy;
                    let pixel_index = src_pixel_y * font_width + src_pixel_x;
                    
                    if pixel_index < pixels.len() {
                        let pixel = pixels[pixel_index];
                        let screen_x = x + dx;
                        let screen_y = y + dy;
                        
                        if screen_x < 400 && screen_y < 240 {
                            display.draw_pixel(screen_x, screen_y, pixel);
                        }
                    }
                }
            }
        }
    }
    
    fn draw_text(&self, display: &mut SharpDisplay, x: usize, y: usize, text: &str) {
        let mut current_x = x;
        for c in text.chars() {
            self.draw_char(display, current_x, y, c);
            current_x += self.font_char_width;
        }
    }
}

impl Page for WriteMenuPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        if self.font_bitmap.is_some() {
            let text_width = self.current_text.len() * self.font_char_width;
            let x = (400 - text_width) / 2;
            let y = (240 - self.font_char_height) / 2;
            
            self.draw_text(display, x, y, &self.current_text);
            
            // Draw instruction with simple font
            let instruction = "Press ESC to return";
            let instr_width = instruction.len() * 6;
            let instr_x = (400 - instr_width) / 2;
            
            // Simple text drawing
            for (i, c) in instruction.chars().enumerate() {
                match c {
                    'A'..='Z' | 'a'..='z' | ' ' | 'E' | 'S' | 'C' | 't' | 'o' | 'r' | 'u' | 'n' => {
                        for dy in 2..6 {
                            for dx in 1..5 {
                                display.draw_pixel(instr_x + i * 6 + dx, 200 + dy, Pixel::Black);
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            display.draw_text(150, 100, "NO FONT LOADED");
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
            Key::Char('\n') => Ok(None),
            Key::Char(c) => {
                self.current_text.push(c);
                Ok(None)
            }
            Key::Backspace => {
                self.current_text.pop();
                Ok(None)
            }
            Key::Esc => Ok(Some(PageId::Menu)),
            _ => Ok(None),
        }
    }
}