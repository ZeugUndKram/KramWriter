use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;

const LETTER_SPACING: usize = 2;
const LINE_SPACING: usize = 5;  // Space between lines
const MAX_LINES: usize = 8;     // Maximum number of lines visible
const CHARS_PER_LINE: usize = 20; // Approximate characters per line

pub struct WriteMenuPage {
    font_bitmap: Option<(Vec<Pixel>, usize, usize)>,
    font_char_width: usize,
    font_char_height: usize,
    chars_per_row: usize,
    char_widths: Vec<usize>,
    lines: Vec<String>,
    cursor_line: usize,
    cursor_pos: usize,
    scroll_offset: usize,  // Which line is at the top
}

impl WriteMenuPage {
    pub fn new() -> Result<Self> {
        let font_path = "/home/kramwriter/KramWriter/fonts/bebas24.bmp";
        println!("Loading font from: {}", font_path);
        
        let (font_bitmap, char_widths) = match std::fs::read(font_path) {
            Ok(data) => {
                println!("Font loaded: {} bytes", data.len());
                match Self::parse_font_bmp(&data) {
                    Some((bitmap, width, height)) => {
                        println!("Font dimensions: {}x{}", width, height);
                        let widths = Self::measure_char_widths(&bitmap, width, height, 30, 30, 19);
                        (Some((bitmap, width, height)), widths)
                    }
                    None => (None, Vec::new()),
                }
            }
            Err(e) => {
                println!("Failed to load font: {}", e);
                (None, Vec::new())
            }
        };
        
        Ok(Self {
            font_bitmap,
            font_char_width: 30,
            font_char_height: 30,
            chars_per_row: 19,
            char_widths,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_pos: 0,
            scroll_offset: 0,
        })
    }
    
    fn parse_font_bmp(data: &[u8]) -> Option<(Vec<Pixel>, usize, usize)> {
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
                            Pixel::Black
                        } else {
                            Pixel::White
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
                        let pixel = if luminance > 128 { Pixel::Black } else { Pixel::White };
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
    
    fn measure_char_widths(pixels: &[Pixel], font_width: usize, font_height: usize, 
                          char_width: usize, char_height: usize, chars_per_row: usize) -> Vec<usize> {
        let printable_chars = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        let mut widths = Vec::new();
        
        for char_index in 0..printable_chars.len() {
            let grid_x = char_index % chars_per_row;
            let grid_y = char_index / chars_per_row;
            
            let src_x = grid_x * char_width;
            let src_y = grid_y * char_height;
            
            let mut leftmost = char_width;
            let mut rightmost = 0;
            
            for dx in 0..char_width {
                for dy in 0..char_height {
                    let src_pixel_x = src_x + dx;
                    let src_pixel_y = src_y + dy;
                    let pixel_index = src_pixel_y * font_width + src_pixel_x;
                    
                    if pixel_index < pixels.len() && pixels[pixel_index] == Pixel::Black {
                        if dx < leftmost { leftmost = dx; }
                        if dx > rightmost { rightmost = dx; }
                    }
                }
            }
            
            let actual_width = if rightmost >= leftmost { 
                (rightmost - leftmost + 1).min(char_width) 
            } else { 
                8
            };
            
            widths.push(actual_width);
        }
        
        widths
    }
    
    fn get_char_index(c: char) -> usize {
        let printable_chars = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        printable_chars.find(c).unwrap_or(0)
    }
    
    fn draw_char_cropped(&self, display: &mut SharpDisplay, x: usize, y: usize, c: char) {
        if let Some((pixels, font_width, font_height)) = &self.font_bitmap {
            let char_index = Self::get_char_index(c);
            let chars_per_row = self.chars_per_row;
            let char_width = self.font_char_width;
            let char_height = self.font_char_height;
            
            let grid_x = char_index % chars_per_row;
            let grid_y = char_index / chars_per_row;
            
            let src_x = grid_x * char_width;
            let src_y = grid_y * char_height;
            
            let mut leftmost = char_width;
            let mut rightmost = 0;
            
            for dx in 0..char_width {
                for dy in 0..char_height {
                    let src_pixel_x = src_x + dx;
                    let src_pixel_y = src_y + dy;
                    let pixel_index = src_pixel_y * font_width + src_pixel_x;
                    
                    if pixel_index < pixels.len() && pixels[pixel_index] == Pixel::Black {
                        if dx < leftmost { leftmost = dx; }
                        if dx > rightmost { rightmost = dx; }
                    }
                }
            }
            
            if rightmost >= leftmost {
                for dy in 0..char_height {
                    for dx in leftmost..=rightmost {
                        let src_pixel_x = src_x + dx;
                        let src_pixel_y = src_y + dy;
                        let pixel_index = src_pixel_y * font_width + src_pixel_x;
                        
                        if pixel_index < pixels.len() {
                            let pixel = pixels[pixel_index];
                            if pixel == Pixel::Black {
                                let screen_x = x + dx - leftmost;
                                let screen_y = y + dy;
                                
                                if screen_x < 400 && screen_y < 240 {
                                    display.draw_pixel(screen_x, screen_y, pixel);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn draw_text_line(&self, display: &mut SharpDisplay, x: usize, y: usize, text: &str) {
        let mut current_x = x;
        for c in text.chars() {
            let char_index = Self::get_char_index(c);
            let char_width = if char_index < self.char_widths.len() { 
                self.char_widths[char_index] 
            } else { 
                8
            };
            
            self.draw_char_cropped(display, current_x, y, c);
            current_x += char_width + LETTER_SPACING;
        }
    }
    
    fn calculate_line_width(&self, text: &str) -> usize {
        let mut width = 0;
        for c in text.chars() {
            let char_index = Self::get_char_index(c);
            let char_width = if char_index < self.char_widths.len() { 
                self.char_widths[char_index] 
            } else { 
                8
            };
            width += char_width + LETTER_SPACING;
        }
        if width > 0 { width - LETTER_SPACING } else { 0 }
    }
    
    fn word_wrap(&self, text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;
        
        for word in text.split_whitespace() {
            let mut word_width = 0;
            for c in word.chars() {
                let char_index = Self::get_char_index(c);
                let char_width = if char_index < self.char_widths.len() { 
                    self.char_widths[char_index] 
                } else { 
                    8
                };
                word_width += char_width + LETTER_SPACING;
            }
            if word_width > 0 {
                word_width -= LETTER_SPACING; // Remove last spacing
            }
            
            // Add space width if not first word
            let space_width = if !current_line.is_empty() { 
                self.char_widths[0] + LETTER_SPACING 
            } else { 
                0 
            };
            
            if current_width + space_width + word_width <= max_width {
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += space_width;
                }
                current_line.push_str(word);
                current_width += word_width;
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                current_line = word.to_string();
                current_width = word_width;
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        lines
    }
}

impl Page for WriteMenuPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        if self.font_bitmap.is_some() && !self.char_widths.is_empty() {
            // Draw visible lines
            let start_y = 20;
            let max_line_width = 380;
            
            for i in 0..MAX_LINES {
                let line_idx = i + self.scroll_offset;
                if line_idx < self.lines.len() {
                    let line = &self.lines[line_idx];
                    let wrapped_lines = self.word_wrap(line, max_line_width);
                    
                    for (wrap_idx, wrapped_line) in wrapped_lines.iter().enumerate() {
                        let line_y = start_y + (i + wrap_idx) * (self.font_char_height + LINE_SPACING);
                        if line_y + self.font_char_height >= 240 { break; }
                        
                        let line_width = self.calculate_line_width(wrapped_line);
                        let x = (400 - line_width) / 2;
                        self.draw_text_line(display, x, line_y, wrapped_line);
                        
                        // Draw cursor if on this line
                        if line_idx == self.cursor_line && wrap_idx == 0 {
                            // Calculate cursor position
                            let before_cursor = &line[..self.cursor_pos.min(line.len())];
                            let cursor_x = x + self.calculate_line_width(before_cursor);
                            for dy in 0..self.font_char_height {
                                display.draw_pixel(cursor_x, line_y + dy, Pixel::Black);
                            }
                        }
                    }
                }
            }
            
            // Draw instruction
            let instruction = "ESC: Menu  Ctrl+S: Save  Ctrl+X: Exit";
            let instr_width = instruction.len() * 6;
            let instr_x = (400 - instr_width) / 2;
            
            for (i, c) in instruction.chars().enumerate() {
                match c {
                    'A'..='Z' | 'a'..='z' | ' ' | 'E' | 'S' | 'C' | 't' | 'r' | 'l' | 'x' | 'i' | 'u' | 'v' | ':' => {
                        for dy in 2..6 {
                            for dx in 1..5 {
                                display.draw_pixel(instr_x + i * 6 + dx, 220 + dy, Pixel::Black);
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
            Key::Char('\n') => {
                // Insert new line
                let current_line = self.lines.remove(self.cursor_line);
                let (left, right) = current_line.split_at(self.cursor_pos.min(current_line.len()));
                
                self.lines.insert(self.cursor_line, left.to_string());
                self.lines.insert(self.cursor_line + 1, right.to_string());
                self.cursor_line += 1;
                self.cursor_pos = 0;
                
                // Adjust scroll
                if self.cursor_line >= self.scroll_offset + MAX_LINES {
                    self.scroll_offset = self.cursor_line - MAX_LINES + 1;
                }
                Ok(None)
            }
            Key::Char(c) => {
                if self.cursor_line >= self.lines.len() {
                    self.lines.push(String::new());
                }
                let line = &mut self.lines[self.cursor_line];
                if self.cursor_pos <= line.len() {
                    line.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                }
                Ok(None)
            }
            Key::Backspace => {
                if self.cursor_pos > 0 {
                    let line = &mut self.lines[self.cursor_line];
                    line.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                } else if self.cursor_line > 0 {
                    // Merge with previous line
                    let current_line = self.lines.remove(self.cursor_line);
                    self.cursor_line -= 1;
                    let prev_line = &mut self.lines[self.cursor_line];
                    self.cursor_pos = prev_line.len();
                    prev_line.push_str(&current_line);
                    
                    // Adjust scroll
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                }
                Ok(None)
            }
            Key::Delete => {
                let line = &mut self.lines[self.cursor_line];
                if self.cursor_pos < line.len() {
                    line.remove(self.cursor_pos);
                } else if self.cursor_line < self.lines.len() - 1 {
                    // Merge with next line
                    let next_line = self.lines.remove(self.cursor_line + 1);
                    line.push_str(&next_line);
                }
                Ok(None)
            }
            Key::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_pos = self.lines[self.cursor_line].len();
                    
                    // Adjust scroll
                    if self.cursor_line < self.scroll_offset {
                        self.scroll_offset = self.cursor_line;
                    }
                }
                Ok(None)
            }
            Key::Right => {
                let line = &self.lines[self.cursor_line];
                if self.cursor_pos < line.len() {
                    self.cursor_pos += 1;
                } else if self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    self.cursor_pos = 0;
                    
                    // Adjust scroll
                    if self.cursor_line >= self.scroll_offset + MAX_LINES {
                        self.scroll_offset = self.cursor_line - MAX_LINES + 1;
                    }
                }
                Ok(None)
            }
            Key::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_pos = self.cursor_pos.min(self.lines[self.cursor_line].len());
                    
                    // Adjust scroll
                    if self.cursor_line < self.scroll_offset {
                        self.scroll_offset = self.cursor_line;
                    }
                }
                Ok(None)
            }
            Key::Down => {
                if self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    self.cursor_pos = self.cursor_pos.min(self.lines[self.cursor_line].len());
                    
                    // Adjust scroll
                    if self.cursor_line >= self.scroll_offset + MAX_LINES {
                        self.scroll_offset = self.cursor_line - MAX_LINES + 1;
                    }
                }
                Ok(None)
            }
            Key::Esc => Ok(Some(PageId::Menu)),
            Key::Ctrl('s') => {
                // Save functionality
                println!("Save not implemented yet");
                Ok(None)
            }
            Key::Ctrl('x') => Ok(Some(PageId::Menu)),
            _ => Ok(None),
        }
    }
}