use rpi_memory_display::Pixel;
use crate::display::SharpDisplay;

const LETTER_SPACING: usize = 2;
const LINE_SPACING: usize = 3;
const MAX_VISIBLE_LINES: usize = 6;
const MAX_LINE_WIDTH: usize = 380;
const LEFT_MARGIN: usize = 10;
const TOP_MARGIN: usize = 10;

pub struct WritingRenderer {
    font_bitmap: Option<(Vec<Pixel>, usize, usize)>,
    font_char_width: usize,
    font_char_height: usize,
    chars_per_row: usize,
    char_widths: Vec<usize>,
}

impl WritingRenderer {
    pub fn new() -> Result<Self> {
        let font_path = "/home/kramwriter/KramWriter/fonts/libsans20.bmp";
        
        let (font_bitmap, char_widths) = match std::fs::read(font_path) {
            Ok(data) => {
                match Self::parse_font_bmp(&data) {
                    Some((bitmap, width, height)) => {
                        println!("Loaded font: {}x{}", width, height);
                        let widths = Self::measure_char_widths(&bitmap, width, 30, 30, 19);
                        (Some((bitmap, width, height)), widths)
                    }
                    None => {
                        println!("Failed to parse font BMP");
                        (None, Vec::new())
                    }
                }
            }
            Err(e) => {
                println!("Failed to read font: {}", e);
                (None, Vec::new())
            }
        };
        
        Ok(Self {
            font_bitmap,
            font_char_width: 30,
            font_char_height: 30,
            chars_per_row: 19,
            char_widths,
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
        if let Some((pixels, font_width, _)) = &self.font_bitmap {
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
    
    fn calculate_text_width(&self, text: &str) -> usize {
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
    
    fn wrap_line(&self, line: &str) -> Vec<String> {
        let mut result = Vec::new();
        
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
            
            if current_width + char_width > MAX_LINE_WIDTH && !current_line.is_empty() {
                if last_whitespace_idx > 0 {
                    let (keep, move_to_next) = current_line.split_at(last_whitespace_idx);
                    result.push(keep.trim_end().to_string());
                    
                    current_line = move_to_next.trim_start().to_string();
                    current_width = self.calculate_text_width(&current_line) + LETTER_SPACING;
                    
                    current_line.push(c);
                    current_width += char_width;
                } else {
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
    
    pub fn render_document(&self, display: &mut SharpDisplay, document: &WritingDocument) {
        let visible_lines = MAX_VISIBLE_LINES.min(document.lines.len().saturating_sub(document.scroll_offset));
        let mut current_y = TOP_MARGIN;
        
        for i in 0..visible_lines {
            let line_idx = i + document.scroll_offset;
            if line_idx < document.lines.len() {
                let wrapped_lines = self.wrap_line(&document.lines[line_idx]);
                
                for wrapped_line in wrapped_lines {
                    if current_y + self.font_char_height >= 240 {
                        break;
                    }
                    
                    self.draw_text_line(display, LEFT_MARGIN, current_y, &wrapped_line);
                    current_y += self.font_char_height + LINE_SPACING;
                }
            }
        }
        
        // Draw cursor
        self.draw_cursor(display, document);
    }
    
    fn draw_cursor(&self, display: &mut SharpDisplay, document: &WritingDocument) {
        let current_line = document.get_current_line_index();
        let cursor_col = document.get_cursor_column();
        
        if current_line >= document.scroll_offset && current_line < document.scroll_offset + MAX_VISIBLE_LINES {
            // Find wrapped line position
            let mut wrapped_y = TOP_MARGIN;
            let mut line_offset = document.scroll_offset;
            
            for line_idx in document.scroll_offset..=current_line {
                if line_idx < document.lines.len() {
                    let wrapped_lines = self.wrap_line(&document.lines[line_idx]);
                    
                    if line_idx == current_line {
                        // This is the current line with cursor
                        let mut current_col = cursor_col;
                        for (wrapped_idx, wrapped_line) in wrapped_lines.iter().enumerate() {
                            let wrapped_len = wrapped_line.len();
                            
                            if current_col <= wrapped_len {
                                // Cursor is in this wrapped segment
                                let before_cursor: String = wrapped_line.chars().take(current_col).collect();
                                let cursor_x = LEFT_MARGIN + self.calculate_text_width(&before_cursor);
                                let cursor_y = wrapped_y;
                                
                                // Draw vertical cursor line
                                for dy in 0..self.font_char_height {
                                    display.draw_pixel(cursor_x, cursor_y + dy, Pixel::Black);
                                }
                                break;
                            } else {
                                current_col -= wrapped_len;
                                wrapped_y += self.font_char_height + LINE_SPACING;
                            }
                        }
                        break;
                    } else {
                        wrapped_y += (self.font_char_height + LINE_SPACING) * wrapped_lines.len();
                    }
                }
            }
        }
    }
    
    pub fn draw_status_bar(&self, display: &mut SharpDisplay, document: &WritingDocument) {
        let status_y = 240 - self.font_char_height - 5;
        
        // Clear status area
        for y in status_y..240 {
            for x in 0..400 {
                display.draw_pixel(x, y, Pixel::White);
            }
        }
        
        // Draw status info
        let current_line = document.get_current_line_index() + 1;
        let total_lines = document.lines.len();
        let cursor_col = document.get_cursor_column() + 1;
        let dirty_indicator = if document.is_dirty() { "* " } else { "  " };
        
        let status_text = format!("{}Ln {}, Col {} ({} total)", 
            dirty_indicator, current_line, cursor_col, total_lines);
        
        self.draw_text_line(display, LEFT_MARGIN, status_y, &status_text);
        
        // Draw file name if available
        if let Some(file_path) = document.get_file_path() {
            let file_name = std::path::Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path);
            
            let file_text = format!("File: {}", file_name);
            let text_width = self.calculate_text_width(&file_text);
            self.draw_text_line(display, 400 - text_width - LEFT_MARGIN, status_y, &file_text);
        }
    }
}