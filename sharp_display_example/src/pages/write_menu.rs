use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;

const LETTER_SPACING: usize = 2;
const LINE_SPACING: usize = 3;
const MAX_VISIBLE_LINES: usize = 6;
const MAX_LINE_WIDTH: usize = 380;
const LEFT_MARGIN: usize = 10;

pub struct WriteMenuPage {
    font_bitmap: Option<(Vec<Pixel>, usize, usize)>,
    small_font_bitmap: Option<(Vec<Pixel>, usize, usize)>,
    font_char_width: usize,
    font_char_height: usize,
    small_font_char_width: usize,
    small_font_char_height: usize,
    chars_per_row: usize,
    small_chars_per_row: usize,
    char_widths: Vec<usize>,
    small_char_widths: Vec<usize>,
    lines: Vec<String>,
    cursor_line: usize,
    cursor_pos: usize,
    scroll_offset: usize,
    word_count: usize,
}

impl WriteMenuPage {
    pub fn new() -> Result<Self> {
        let font_path = "/home/kramwriter/KramWriter/fonts/bebas24.bmp";
        let small_font_path = "/home/kramwriter/KramWriter/fonts/libsans12.bmp";
        
        let (font_bitmap, char_widths) = match std::fs::read(font_path) {
            Ok(data) => {
                match Self::parse_font_bmp(&data, false) {
                    Some((bitmap, width, height)) => {
                        let widths = Self::measure_char_widths(&bitmap, width, 30, 30, 19);
                        (Some((bitmap, width, height)), widths)
                    }
                    None => (None, Vec::new()),
                }
            }
            Err(_) => (None, Vec::new()),
        };
        
        let (small_font_bitmap, small_char_widths) = match std::fs::read(small_font_path) {
            Ok(data) => {
                match Self::parse_font_bmp(&data, true) {
                    Some((bitmap, width, height)) => {
                        let widths = Self::measure_char_widths(&bitmap, width, 12, 12, 32);
                        (Some((bitmap, width, height)), widths)
                    }
                    None => (None, Vec::new()),
                }
            }
            Err(_) => (None, Vec::new()),
        };
        
        let mut page = Self {
            font_bitmap,
            small_font_bitmap,
            font_char_width: 30,
            font_char_height: 30,
            small_font_char_width: 12,
            small_font_char_height: 12,
            chars_per_row: 19,
            small_chars_per_row: 32,
            char_widths,
            small_char_widths,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_pos: 0,
            scroll_offset: 0,
            word_count: 0,
        };
        
        // Calculate initial word count
        page.update_word_count();
        
        Ok(page)
    }
    
    fn parse_font_bmp(data: &[u8], is_small_font: bool) -> Option<(Vec<Pixel>, usize, usize)> {
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
                let row_bytes = ((width * 3 + 3) / 4) * 4; // BMP rows are padded to 4 bytes
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
                        
                        // For 24-bit BMPs, we need to decide threshold based on font
                        let threshold = if is_small_font { 128 } else { 128 };
                        let pixel = if luminance > threshold {
                            Pixel::Black
                        } else {
                            Pixel::White
                        };
                        pixels.push(pixel);
                    }
                }
            }
            _ => return None,
        }
        
        Some((pixels, width, height))
    }
    
    fn measure_char_widths(pixels: &[Pixel], font_width: usize, 
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
                if char_width == 12 { 6 } else { 8 } // Different default for small font
            };
            
            widths.push(actual_width);
        }
        
        widths
    }
    
    fn get_char_index(c: char) -> usize {
        let printable_chars = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        printable_chars.find(c).unwrap_or(0)
    }
    
    fn draw_char_cropped(&self, display: &mut SharpDisplay, x: usize, y: usize, c: char, use_small_font: bool) {
        if use_small_font {
            if let Some((font_bitmap, font_width, _)) = &self.small_font_bitmap {
                let char_index = Self::get_char_index(c);
                let char_widths = &self.small_char_widths;
                let char_width = self.small_font_char_width;
                let char_height = self.small_font_char_height;
                let chars_per_row = self.small_chars_per_row;
                
                let char_width_actual = if char_index < char_widths.len() { 
                    char_widths[char_index] 
                } else { 
                    6 
                };
                
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
                        
                        if pixel_index < font_bitmap.len() && font_bitmap[pixel_index] == Pixel::Black {
                            if dx < leftmost { leftmost = dx; }
                            if dx > rightmost { rightmost = dx; }
                        }
                    }
                }
                
                if rightmost >= leftmost {
                    let actual_rightmost = leftmost + char_width_actual - 1;
                    for dy in 0..char_height {
                        for dx in leftmost..=actual_rightmost.min(rightmost) {
                            let src_pixel_x = src_x + dx;
                            let src_pixel_y = src_y + dy;
                            let pixel_index = src_pixel_y * font_width + src_pixel_x;
                            
                            if pixel_index < font_bitmap.len() {
                                let pixel = font_bitmap[pixel_index];
                                if pixel == Pixel::Black {
                                    let screen_x = x + dx - leftmost;
                                    let screen_y = y + dy;
                                    
                                    if screen_x < 400 && screen_y < 240 {
                                        display.draw_pixel(screen_x, screen_y, Pixel::White);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            if let Some((font_bitmap, font_width, _)) = &self.font_bitmap {
                let char_index = Self::get_char_index(c);
                let char_widths = &self.char_widths;
                let char_width = self.font_char_width;
                let char_height = self.font_char_height;
                let chars_per_row = self.chars_per_row;
                
                let char_width_actual = if char_index < char_widths.len() { 
                    char_widths[char_index] 
                } else { 
                    8 
                };
                
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
                        
                        if pixel_index < font_bitmap.len() && font_bitmap[pixel_index] == Pixel::Black {
                            if dx < leftmost { leftmost = dx; }
                            if dx > rightmost { rightmost = dx; }
                        }
                    }
                }
                
                if rightmost >= leftmost {
                    let actual_rightmost = leftmost + char_width_actual - 1;
                    for dy in 0..char_height {
                        for dx in leftmost..=actual_rightmost.min(rightmost) {
                            let src_pixel_x = src_x + dx;
                            let src_pixel_y = src_y + dy;
                            let pixel_index = src_pixel_y * font_width + src_pixel_x;
                            
                            if pixel_index < font_bitmap.len() {
                                let pixel = font_bitmap[pixel_index];
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
    }
    
    fn draw_text_line(&self, display: &mut SharpDisplay, x: usize, y: usize, text: &str, use_small_font: bool) {
        let mut current_x = x;
        for c in text.chars() {
            let char_index = Self::get_char_index(c);
            let char_widths = if use_small_font { &self.small_char_widths } else { &self.char_widths };
            let char_width = if char_index < char_widths.len() { 
                char_widths[char_index] 
            } else { 
                if use_small_font { 6 } else { 8 }
            };
            
            self.draw_char_cropped(display, current_x, y, c, use_small_font);
            current_x += char_width + LETTER_SPACING;
        }
    }
    
    fn calculate_text_width(&self, text: &str, use_small_font: bool) -> usize {
        let mut width = 0;
        let char_widths = if use_small_font { &self.small_char_widths } else { &self.char_widths };
        
        for c in text.chars() {
            let char_index = Self::get_char_index(c);
            let char_width = if char_index < char_widths.len() { 
                char_widths[char_index] 
            } else { 
                if use_small_font { 6 } else { 8 }
            };
            width += char_width + LETTER_SPACING;
        }
        if width > 0 { width - LETTER_SPACING } else { 0 }
    }
    
    fn update_word_count(&mut self) {
        let mut count = 0;
        let mut in_word = false;
        
        for line in &self.lines {
            for c in line.chars() {
                if c.is_alphanumeric() {
                    if !in_word {
                        count += 1;
                        in_word = true;
                    }
                } else {
                    in_word = false;
                }
            }
            // Reset at end of each line
            in_word = false;
        }
        
        self.word_count = count;
    }
    
    fn wrap_line(&self, line: &str) -> Vec<String> {
        let mut result = Vec::new();
        
        // If line is empty, return empty string
        if line.trim().is_empty() {
            return vec![String::new()];
        }
        
        let mut current_line = String::new();
        let mut current_width = 0;
        let mut last_whitespace_idx = 0;
        
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            let char_index = Self::get_char_index(c);
            let char_width = if char_index < self.char_widths.len() { 
                self.char_widths[char_index] + LETTER_SPACING
            } else { 
                8 + LETTER_SPACING
            };
            
            // Check if adding this character would overflow
            if current_width + char_width > MAX_LINE_WIDTH && !current_line.is_empty() {
                // Try to break at last whitespace if possible
                if last_whitespace_idx > 0 {
                    // Split at the last whitespace
                    let (keep, move_to_next) = current_line.split_at(last_whitespace_idx);
                    result.push(keep.trim_end().to_string());
                    
                    // Start new line with the word that was after whitespace
                    current_line = move_to_next.trim_start().to_string();
                    current_width = self.calculate_text_width(&current_line, false) + LETTER_SPACING;
                    
                    // Add current character to the new line
                    current_line.push(c);
                    current_width += char_width;
                } else {
                    // No whitespace to break at, just break here
                    result.push(current_line);
                    current_line = String::new();
                    current_width = 0;
                    
                    current_line.push(c);
                    current_width += char_width;
                }
                
                last_whitespace_idx = 0;
            } else {
                current_line.push(c);
                current_width += char_width;
                
                // Track last whitespace position for word wrapping
                if c.is_whitespace() {
                    last_whitespace_idx = current_line.len();
                }
            }
        }
        
        if !current_line.is_empty() {
            result.push(current_line);
        }
        
        result
    }
    
    fn get_all_wrapped_lines(&self) -> Vec<(String, usize, usize)> {
        // Returns (wrapped_line, original_line_index, char_position_in_original)
        let mut result = Vec::new();
        
        for (line_idx, line) in self.lines.iter().enumerate() {
            let wrapped = self.wrap_line(line);
            let mut char_pos = 0;
            for wrapped_line in wrapped {
                result.push((wrapped_line.clone(), line_idx, char_pos));
                char_pos += wrapped_line.len();
            }
        }
        
        result
    }
    
    fn find_wrapped_line_for_cursor(&self) -> usize {
        let wrapped_lines = self.get_all_wrapped_lines();
        
        for (i, (wrapped_line, line_idx, char_pos_in_original)) in wrapped_lines.iter().enumerate() {
            if *line_idx == self.cursor_line {
                if self.cursor_pos >= *char_pos_in_original && 
                   self.cursor_pos <= *char_pos_in_original + wrapped_line.len() {
                    return i;
                }
            }
        }
        
        // Fallback: cursor is at the end
        wrapped_lines.len().saturating_sub(1)
    }
    
    fn find_cursor_for_wrapped_line(&self, target_wrapped_idx: usize) -> (usize, usize) {
        let wrapped_lines = self.get_all_wrapped_lines();
        
        if target_wrapped_idx < wrapped_lines.len() {
            let (wrapped_line, original_line_idx, char_pos_in_original) = &wrapped_lines[target_wrapped_idx];
            
            // Try to maintain similar X position
            let current_x = self.get_cursor_x_position();
            let mut best_pos = 0;
            let mut best_distance = usize::MAX;
            
            // Check each character position in the target wrapped line
            for pos_in_wrapped in 0..=wrapped_line.len() {
                let prefix: String = wrapped_line.chars().take(pos_in_wrapped).collect();
                let x_pos = LEFT_MARGIN + self.calculate_text_width(&prefix, false);
                
                let distance = if x_pos >= current_x {
                    x_pos - current_x
                } else {
                    current_x - x_pos
                };
                
                if distance < best_distance {
                    best_distance = distance;
                    best_pos = pos_in_wrapped;
                }
            }
            
            return (*original_line_idx, *char_pos_in_original + best_pos);
        }
        
        // Fallback
        (self.cursor_line, self.cursor_pos)
    }
    
    fn ensure_cursor_visible(&mut self) {
        let wrapped_cursor_line = self.find_wrapped_line_for_cursor();
        let total_wrapped = self.get_all_wrapped_lines().len();
        
        // Start scrolling when cursor reaches 5th visible line (0-indexed 4)
        const SCROLL_THRESHOLD: usize = 4;
        
        if wrapped_cursor_line < self.scroll_offset {
            self.scroll_offset = wrapped_cursor_line;
        } else if wrapped_cursor_line >= self.scroll_offset + SCROLL_THRESHOLD {
            self.scroll_offset = wrapped_cursor_line - SCROLL_THRESHOLD + 1;
        }
        
        if total_wrapped > MAX_VISIBLE_LINES {
            let max_scroll = total_wrapped.saturating_sub(MAX_VISIBLE_LINES);
            self.scroll_offset = self.scroll_offset.min(max_scroll);
        } else {
            self.scroll_offset = 0;
        }
    }
    
    fn get_cursor_x_position(&self) -> usize {
        let line = &self.lines[self.cursor_line];
        let prefix: String = line.chars().take(self.cursor_pos).collect();
        LEFT_MARGIN + self.calculate_text_width(&prefix, false)
    }
}

impl Page for WriteMenuPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        if self.font_bitmap.is_some() && !self.char_widths.is_empty() {
            let start_y = 10;
            
            // Get all wrapped lines with metadata
            let wrapped_lines = self.get_all_wrapped_lines();
            
            // Draw visible wrapped lines
            for i in 0..MAX_VISIBLE_LINES {
                let wrapped_idx = i + self.scroll_offset;
                if wrapped_idx < wrapped_lines.len() {
                    let line_y = start_y + i * (self.font_char_height + LINE_SPACING);
                    let (text, original_line_idx, char_pos_in_original) = &wrapped_lines[wrapped_idx];
                    self.draw_text_line(display, LEFT_MARGIN, line_y, text, false);
                    
                    // Draw cursor if this wrapped line contains cursor
                    if *original_line_idx == self.cursor_line {
                        let cursor_in_original = self.cursor_pos;
                        let cursor_in_wrapped = cursor_in_original.saturating_sub(*char_pos_in_original);
                        
                        if cursor_in_wrapped <= text.len() {
                            // Get characters up to cursor position
                            let mut before_cursor = String::new();
                            let mut count = 0;
                            for c in text.chars() {
                                if count >= cursor_in_wrapped { break; }
                                before_cursor.push(c);
                                count += 1;
                            }
                            let cursor_x = LEFT_MARGIN + self.calculate_text_width(&before_cursor, false);
                            for dy in 0..self.font_char_height {
                                display.draw_pixel(cursor_x, line_y + dy, Pixel::Black);
                            }
                        }
                    }
                }
            }
            
            // Draw scroll indicators
            if self.scroll_offset > 0 {
                for dy in 0..6 {
                    display.draw_pixel(5, 5 + dy, Pixel::Black);
                }
            }
            
            let total_wrapped = wrapped_lines.len();
            if total_wrapped > self.scroll_offset + MAX_VISIBLE_LINES {
                for dy in 0..6 {
                    display.draw_pixel(5, 230 + dy, Pixel::Black);
                }
            }
            
            // Draw black bar at the bottom (from y=220 to y=240)
            for y in 220..240 {
                for x in 0..400 {
                    display.draw_pixel(x, y, Pixel::Black);
                }
            }
            
            // Draw word count on the black bar in white text
            let word_text = format!("words: {}", self.word_count);
            let text_x = LEFT_MARGIN;
            let text_y = 224; // Center in the black bar (bar is from 220-240, font is 12px tall)
            
            self.draw_text_line(display, text_x, text_y, &word_text, true);
            
        } else {
            display.draw_text(150, 100, "NO FONT LOADED");
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        let needs_word_count_update = match key {
            Key::Char('\n') => {
                // Always create a new line, even if we're at the end
                if self.cursor_line >= self.lines.len() {
                    self.lines.push(String::new());
                }
                
                let line = &self.lines[self.cursor_line];
                let at_end = self.cursor_pos >= line.chars().count();
                
                if at_end {
                    // Insert new empty line after current line
                    self.lines.insert(self.cursor_line + 1, String::new());
                    self.cursor_line += 1;
                    self.cursor_pos = 0;
                } else {
                    // Split current line at cursor
                    let mut chars = line.chars();
                    let left: String = chars.by_ref().take(self.cursor_pos).collect();
                    let right: String = chars.collect();
                    
                    self.lines[self.cursor_line] = left;
                    self.lines.insert(self.cursor_line + 1, right);
                    self.cursor_line += 1;
                    self.cursor_pos = 0;
                }
                self.ensure_cursor_visible();
                true
            }
            Key::Char(c) => {
                if self.cursor_line >= self.lines.len() {
                    self.lines.push(String::new());
                }
                let line = &mut self.lines[self.cursor_line];
                
                // Insert at character position
                let mut new_line = String::new();
                let mut inserted = false;
                for (i, ch) in line.chars().enumerate() {
                    if i == self.cursor_pos && !inserted {
                        new_line.push(c);
                        inserted = true;
                    }
                    new_line.push(ch);
                }
                if !inserted {
                    new_line.push(c);
                }
                *line = new_line;
                self.cursor_pos += 1;
                self.ensure_cursor_visible();
                true
            }
            Key::Backspace => {
                if self.cursor_pos > 0 {
                    let line = &mut self.lines[self.cursor_line];
                    let mut new_line = String::new();
                    for (i, ch) in line.chars().enumerate() {
                        if i != self.cursor_pos - 1 {
                            new_line.push(ch);
                        }
                    }
                    *line = new_line;
                    self.cursor_pos -= 1;
                } else if self.cursor_line > 0 {
                    let current_line = self.lines.remove(self.cursor_line);
                    self.cursor_line -= 1;
                    let prev_line = &mut self.lines[self.cursor_line];
                    self.cursor_pos = prev_line.chars().count();
                    prev_line.push_str(&current_line);
                }
                self.ensure_cursor_visible();
                true
            }
            Key::Delete => {
                let line = &mut self.lines[self.cursor_line];
                let char_count = line.chars().count();
                if self.cursor_pos < char_count {
                    let mut new_line = String::new();
                    for (i, ch) in line.chars().enumerate() {
                        if i != self.cursor_pos {
                            new_line.push(ch);
                        }
                    }
                    *line = new_line;
                } else if self.cursor_line < self.lines.len() - 1 {
                    let next_line = self.lines.remove(self.cursor_line + 1);
                    self.lines[self.cursor_line].push_str(&next_line);
                }
                self.ensure_cursor_visible();
                true
            }
            Key::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_pos = self.lines[self.cursor_line].chars().count();
                }
                self.ensure_cursor_visible();
                false
            }
            Key::Right => {
                let char_count = self.lines[self.cursor_line].chars().count();
                if self.cursor_pos < char_count {
                    self.cursor_pos += 1;
                } else if self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    self.cursor_pos = 0;
                }
                self.ensure_cursor_visible();
                false
            }
            Key::Up => {
                let current_wrapped_idx = self.find_wrapped_line_for_cursor();
                
                if current_wrapped_idx > 0 {
                    // Find position in the wrapped line above
                    let (new_line, new_pos) = self.find_cursor_for_wrapped_line(current_wrapped_idx - 1);
                    self.cursor_line = new_line;
                    self.cursor_pos = new_pos;
                } else if self.cursor_line > 0 {
                    // Move to previous line, end of line
                    self.cursor_line -= 1;
                    self.cursor_pos = self.lines[self.cursor_line].chars().count();
                }
                self.ensure_cursor_visible();
                false
            }
            Key::Down => {
                let current_wrapped_idx = self.find_wrapped_line_for_cursor();
                let wrapped_lines = self.get_all_wrapped_lines();
                
                if current_wrapped_idx < wrapped_lines.len() - 1 {
                    // Find position in the wrapped line below
                    let (new_line, new_pos) = self.find_cursor_for_wrapped_line(current_wrapped_idx + 1);
                    self.cursor_line = new_line;
                    self.cursor_pos = new_pos;
                } else if self.cursor_line < self.lines.len() - 1 {
                    // Move to next line, start of line
                    self.cursor_line += 1;
                    self.cursor_pos = 0;
                }
                self.ensure_cursor_visible();
                false
            }
            Key::PageUp => {
                if self.scroll_offset > 0 {
                    self.scroll_offset = self.scroll_offset.saturating_sub(MAX_VISIBLE_LINES);
                    // Keep cursor visible
                    self.ensure_cursor_visible();
                }
                false
            }
            Key::PageDown => {
                let total_wrapped = self.get_all_wrapped_lines().len();
                if self.scroll_offset + MAX_VISIBLE_LINES < total_wrapped {
                    self.scroll_offset = (self.scroll_offset + MAX_VISIBLE_LINES).min(total_wrapped - 1);
                    // Keep cursor visible
                    self.ensure_cursor_visible();
                }
                false
            }
            Key::Esc => return Ok(Some(PageId::Menu)),
            Key::Ctrl('s') => {
                println!("Save not implemented yet");
                false
            }
            Key::Ctrl('x') => return Ok(Some(PageId::Menu)),
            _ => false,
        };
        
        if needs_word_count_update {
            self.update_word_count();
        }
        
        Ok(None)
    }
}