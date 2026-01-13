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
    font_sizes: Vec<FontSize>,
    current_font_index: usize,
    lines: Vec<String>,
    cursor_line: usize,
    cursor_pos: usize,
    scroll_offset: usize,
}

struct FontSize {
    path: &'static str,
    char_width: usize,
    char_height: usize,
    chars_per_row: usize,
    bitmap: Option<(Vec<Pixel>, usize, usize)>,
    char_widths: Vec<usize>,
}

impl WriteMenuPage {
    pub fn new() -> Result<Self> {
        // Define all available font sizes
        let mut font_sizes = vec![
            FontSize::new("/home/kramwriter/KramWriter/fonts/libsans12.bmp", 12, 12, 32),
            FontSize::new("/home/kramwriter/KramWriter/fonts/libsans14.bmp", 14, 14, 32),
            FontSize::new("/home/kramwriter/KramWriter/fonts/libsans16.bmp", 16, 16, 32),
            FontSize::new("/home/kramwriter/KramWriter/fonts/libsans18.bmp", 18, 18, 32),
            FontSize::new("/home/kramwriter/KramWriter/fonts/libsans20.bmp", 20, 20, 32),
            FontSize::new("/home/kramwriter/KramWriter/fonts/libsans22.bmp", 22, 22, 32),
            FontSize::new("/home/kramwriter/KramWriter/fonts/libsans24.bmp", 24, 24, 32),
            FontSize::new("/home/kramwriter/KramWriter/fonts/libsans26.bmp", 26, 26, 32),
        ];
        
        // Start with libsans20 (index 4)
        let current_font_index = 4;
        
        // Load the initial font
        if current_font_index < font_sizes.len() {
            if let Err(e) = font_sizes[current_font_index].load() {
                eprintln!("Failed to load font {}: {}", font_sizes[current_font_index].path, e);
                eprintln!("Error: {}", e);
            } else {
                println!("Successfully loaded font: {}", font_sizes[current_font_index].path);
                println!("Char widths loaded: {}", font_sizes[current_font_index].char_widths.len());
            }
        }
        
        Ok(Self {
            font_sizes,
            current_font_index,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_pos: 0,
            scroll_offset: 0,
        })
    }
    
    fn current_font(&self) -> &FontSize {
        &self.font_sizes[self.current_font_index]
    }
    
    fn current_font_mut(&mut self) -> &mut FontSize {
        &mut self.font_sizes[self.current_font_index]
    }
    
    fn decrease_font_size(&mut self) {
        if self.current_font_index > 0 {
            self.current_font_index -= 1;
            if let Err(e) = self.current_font_mut().load() {
                eprintln!("Failed to load font {}: {}", self.current_font().path, e);
            } else {
                println!("Decreased font size to: {}px", self.current_font().char_height);
            }
        }
    }
    
    fn increase_font_size(&mut self) {
        if self.current_font_index < self.font_sizes.len() - 1 {
            self.current_font_index += 1;
            if let Err(e) = self.current_font_mut().load() {
                eprintln!("Failed to load font {}: {}", self.current_font().path, e);
            } else {
                println!("Increased font size to: {}px", self.current_font().char_height);
            }
        }
    }
    
    fn get_char_index(c: char) -> usize {
        let printable_chars = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        printable_chars.find(c).unwrap_or(0)
    }
    
    fn draw_char_cropped(&self, display: &mut SharpDisplay, x: usize, y: usize, c: char) {
        let font = self.current_font();
        if let Some((pixels, font_width, _)) = &font.bitmap {
            let char_index = Self::get_char_index(c);
            let chars_per_row = font.chars_per_row;
            let char_width = font.char_width;
            let char_height = font.char_height;
            
            let grid_x = char_index % chars_per_row;
            let grid_y = char_index / chars_per_row;
            
            let src_x = grid_x * char_width;
            let src_y = grid_y * char_height;
            
            // For libsans fonts, they're likely monospaced, so draw the full character
            for dy in 0..char_height {
                for dx in 0..char_width {
                    let src_pixel_x = src_x + dx;
                    let src_pixel_y = src_y + dy;
                    let pixel_index = src_pixel_y * font_width + src_pixel_x;
                    
                    if pixel_index < pixels.len() {
                        let pixel = pixels[pixel_index];
                        if pixel == Pixel::Black {
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
    }
    
    fn draw_text_line(&self, display: &mut SharpDisplay, x: usize, y: usize, text: &str) {
        let font = self.current_font(); // FIXED: Added this line
        let mut current_x = x;
        for c in text.chars() {
            let char_index = Self::get_char_index(c);
            // For libsans, use the full character width (monospaced)
            let char_width = font.char_width;
            
            self.draw_char_cropped(display, current_x, y, c);
            current_x += char_width + LETTER_SPACING;
        }
    }
    
    fn calculate_text_width(&self, text: &str) -> usize {
        let font = self.current_font();
        let char_count = text.chars().count();
        if char_count == 0 {
            return 0;
        }
        // For monospaced fonts: width = (char_width + spacing) * char_count - spacing
        char_count * (font.char_width + LETTER_SPACING) - LETTER_SPACING
    }
    
    fn wrap_line(&self, line: &str) -> Vec<String> {
        let font = self.current_font();
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
            let char_width = font.char_width + LETTER_SPACING;
            
            // Check if adding this character would overflow
            if current_width + char_width > MAX_LINE_WIDTH && !current_line.is_empty() {
                // Try to break at last whitespace if possible
                if last_whitespace_idx > 0 {
                    // Split at the last whitespace
                    let (keep, move_to_next) = current_line.split_at(last_whitespace_idx);
                    result.push(keep.trim_end().to_string());
                    
                    // Start new line with the word that was after whitespace
                    current_line = move_to_next.trim_start().to_string();
                    current_width = self.calculate_text_width(&current_line) + LETTER_SPACING;
                    
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
                let x_pos = LEFT_MARGIN + self.calculate_text_width(&prefix);
                
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
        LEFT_MARGIN + self.calculate_text_width(&prefix)
    }
}

impl FontSize {
    fn new(path: &'static str, char_width: usize, char_height: usize, chars_per_row: usize) -> Self {
        Self {
            path,
            char_width,
            char_height,
            chars_per_row,
            bitmap: None,
            char_widths: Vec::new(),
        }
    }
    
    fn load(&mut self) -> Result<()> {
        if self.bitmap.is_some() {
            return Ok(()); // Already loaded
        }
        
        match std::fs::read(self.path) {
            Ok(data) => {
                match Self::parse_font_bmp(&data) {
                    Some((bitmap, width, height)) => {
                        // For libsans, we'll assume monospaced, so char_widths is not needed
                        // But we'll still measure for consistency
                        self.char_widths = Self::measure_char_widths(&bitmap, width, self.char_width, self.char_height, self.chars_per_row);
                        self.bitmap = Some((bitmap, width, height));
                        println!("Loaded font {}: {}x{}, char widths measured: {}", 
                                self.path, width, height, self.char_widths.len());
                        Ok(())
                    }
                    None => anyhow::bail!("Failed to parse font bitmap"),
                }
            }
            Err(e) => anyhow::bail!("Failed to read font file: {}", e),
        }
    }
    
    fn parse_font_bmp(data: &[u8]) -> Option<(Vec<Pixel>, usize, usize)> {
        if data.len() < 54 { return None; }
        if data[0] != 0x42 || data[1] != 0x4D { return None; }
        
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]) as usize;
        let data_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        
        println!("BMP info: {}x{}px, {}bpp, offset: {}", width, height, bits_per_pixel, data_offset);
        
        if data_offset >= data.len() { 
            println!("Data offset out of bounds");
            return None; 
        }
        
        let mut pixels = Vec::with_capacity(width * height);
        
        match bits_per_pixel {
            32 => {
                println!("Parsing 32-bit BMP");
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
                        
                        // For libsans fonts (black text on white background with alpha)
                        let pixel = if alpha < 128 {
                            Pixel::White
                        } else if luminance < 128 { // Black text
                            Pixel::Black
                        } else {
                            Pixel::White
                        };
                        pixels.push(pixel);
                    }
                }
            }
            24 => {
                println!("Parsing 24-bit BMP");
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
                        
                        // For 24-bit libsans fonts - black text on white background
                        let pixel = if luminance < 128 { // Black text
                            Pixel::Black
                        } else {
                            Pixel::White
                        };
                        pixels.push(pixel);
                    }
                }
            }
            8 => {
                println!("Parsing 8-bit BMP (palette)");
                // For 8-bit BMPs, we need to read the color palette
                let palette_start = 54;
                let _palette_size = 256 * 4; // 256 colors * 4 bytes each (prefix with underscore to avoid warning)
                
                let row_bytes = ((width + 3) / 4) * 4; // Padded to 4 bytes
                
                // Read palette
                let mut palette = Vec::with_capacity(256);
                for i in 0..256 {
                    let palette_offset = palette_start + i * 4;
                    if palette_offset + 2 >= data.len() {
                        palette.push((0, 0, 0)); // Default to black
                        continue;
                    }
                    let b = data[palette_offset] as u32;
                    let g = data[palette_offset + 1] as u32;
                    let r = data[palette_offset + 2] as u32;
                    palette.push((r, g, b));
                }
                
                // Read pixels
                for y in 0..height {
                    let row_start = data_offset + (height - 1 - y) * row_bytes;
                    for x in 0..width {
                        let pixel_start = row_start + x;
                        if pixel_start >= data.len() {
                            pixels.push(Pixel::White);
                            continue;
                        }
                        let palette_index = data[pixel_start] as usize;
                        if palette_index >= palette.len() {
                            pixels.push(Pixel::White);
                            continue;
                        }
                        let (r, g, b) = palette[palette_index];
                        let luminance = (r * 299 + g * 587 + b * 114) / 1000;
                        
                        let pixel = if luminance < 128 {
                            Pixel::Black
                        } else {
                            Pixel::White
                        };
                        pixels.push(pixel);
                    }
                }
            }
            1 => {
                println!("Parsing 1-bit BMP (monochrome)");
                let row_bytes = ((width + 31) / 32) * 4; // Padded to 4 bytes
                for y in 0..height {
                    let row_start = data_offset + (height - 1 - y) * row_bytes;
                    for x in 0..width {
                        let byte_offset = row_start + (x / 8);
                        let bit_offset = 7 - (x % 8); // BMP bits are stored MSB first
                        
                        if byte_offset >= data.len() {
                            pixels.push(Pixel::White);
                            continue;
                        }
                        
                        let byte = data[byte_offset];
                        let bit = (byte >> bit_offset) & 1;
                        
                        // 1 = white, 0 = black (typically for monochrome)
                        let pixel = if bit == 1 {
                            Pixel::White
                        } else {
                            Pixel::Black
                        };
                        pixels.push(pixel);
                    }
                }
            }
            _ => {
                println!("Unsupported bits per pixel: {}", bits_per_pixel);
                return None;
            }
        }
        
        println!("Successfully parsed BMP with {} pixels", pixels.len());
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
                char_width // Default to full width for monospaced
            };
            
            widths.push(actual_width);
        }
        
        widths
    }
}

impl Page for WriteMenuPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        let font = self.current_font();
        if font.bitmap.is_some() {
            let start_y = 10;
            
            // Get all wrapped lines with metadata
            let wrapped_lines = self.get_all_wrapped_lines();
            
            // Draw visible wrapped lines
            for i in 0..MAX_VISIBLE_LINES {
                let wrapped_idx = i + self.scroll_offset;
                if wrapped_idx < wrapped_lines.len() {
                    let line_y = start_y + i * (font.char_height + LINE_SPACING);
                    let (text, original_line_idx, char_pos_in_original) = &wrapped_lines[wrapped_idx];
                    self.draw_text_line(display, LEFT_MARGIN, line_y, text);
                    
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
                            let cursor_x = LEFT_MARGIN + self.calculate_text_width(&before_cursor);
                            for dy in 0..font.char_height {
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
        } else {
            display.draw_text(150, 100, "NO FONT LOADED");
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
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
                Ok(None)
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
                Ok(None)
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
                Ok(None)
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
                Ok(None)
            }
            Key::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_pos = self.lines[self.cursor_line].chars().count();
                }
                self.ensure_cursor_visible();
                Ok(None)
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
                Ok(None)
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
                Ok(None)
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
                Ok(None)
            }
            Key::PageUp => {
                if self.scroll_offset > 0 {
                    self.scroll_offset = self.scroll_offset.saturating_sub(MAX_VISIBLE_LINES);
                    // Keep cursor visible
                    self.ensure_cursor_visible();
                }
                Ok(None)
            }
            Key::PageDown => {
                let total_wrapped = self.get_all_wrapped_lines().len();
                if self.scroll_offset + MAX_VISIBLE_LINES < total_wrapped {
                    self.scroll_offset = (self.scroll_offset + MAX_VISIBLE_LINES).min(total_wrapped - 1);
                    // Keep cursor visible
                    self.ensure_cursor_visible();
                }
                Ok(None)
            }
            Key::Ctrl('=') => {
                self.increase_font_size();
                self.ensure_cursor_visible();
                Ok(None)
            }
            Key::Ctrl('+') => {
                self.increase_font_size();
                self.ensure_cursor_visible();
                Ok(None)
            }
            Key::Ctrl('-') => {
                self.decrease_font_size();
                self.ensure_cursor_visible();
                Ok(None)
            }
            Key::Esc => Ok(Some(PageId::Menu)),
            Key::Ctrl('s') => {
                println!("Save not implemented yet");
                Ok(None)
            }
            Key::Ctrl('x') => Ok(Some(PageId::Menu)),
            _ => Ok(None),
        }
    }
}