use rpi_memory_display::Pixel;
use crate::display::SharpDisplay;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;

const ITEMS_PER_PAGE: usize = 8;
const ITEM_HEIGHT: usize = 30;
const LEFT_MARGIN: usize = 10;
const TOP_MARGIN: usize = 40;

pub struct FileBrowser {
    current_path: PathBuf,
    items: Vec<FileItem>,
    selected_index: usize,
    scroll_offset: usize,
    renderer: FileBrowserRenderer,
}

#[derive(Debug, Clone)]
pub enum FileItem {
    Directory(PathBuf),
    File(PathBuf),
    ParentDirectory,
}

pub struct FileBrowserRenderer {
    font_bitmap: Option<(Vec<Pixel>, usize, usize)>,
    font_char_width: usize,
    font_char_height: usize,
    chars_per_row: usize,
    char_widths: Vec<usize>,
}

impl FileBrowserRenderer {
    pub fn new() -> Result<Self> {
        let font_path = "/home/kramwriter/KramWriter/fonts/libsans20.bmp";
        
        let (font_bitmap, char_widths) = match std::fs::read(font_path) {
            Ok(data) => {
                match Self::parse_font_bmp(&data) {
                    Some((bitmap, width, height)) => {
                        let widths = Self::measure_char_widths(&bitmap, width, 30, 30, 19);
                        (Some((bitmap, width, height)), widths)
                    }
                    None => {
                        println!("Failed to parse font BMP for file browser");
                        (None, Vec::new())
                    }
                }
            }
            Err(e) => {
                println!("Failed to read font for file browser: {}", e);
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
                        
                        // For file browser, we want black text on white background
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
    
    fn get_char_index(c: char) -> Option<usize> {
        let printable_chars = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        printable_chars.find(c)
    }
    
    fn draw_char_cropped(&self, display: &mut SharpDisplay, x: usize, y: usize, c: char) {
        if c == ' ' {
            return;
        }
        
        if let Some((pixels, font_width, _)) = &self.font_bitmap {
            if let Some(char_index) = Self::get_char_index(c) {
                if char_index >= self.char_widths.len() {
                    return;
                }
                
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
    }
    
    fn draw_text_line(&self, display: &mut SharpDisplay, x: usize, y: usize, text: &str) {
        let mut current_x = x;
        for c in text.chars() {
            let char_width = if let Some(char_index) = Self::get_char_index(c) {
                if char_index < self.char_widths.len() { 
                    self.char_widths[char_index] 
                } else { 
                    8
                }
            } else {
                8
            };
            
            self.draw_char_cropped(display, current_x, y, c);
            current_x += char_width + 2; // Letter spacing
        }
    }
    
    fn calculate_text_width(&self, text: &str) -> usize {
        let mut width = 0;
        for c in text.chars() {
            let char_width = if let Some(char_index) = Self::get_char_index(c) {
                if char_index < self.char_widths.len() { 
                    self.char_widths[char_index] 
                } else { 
                    8
                }
            } else {
                8
            };
            width += char_width + 2;
        }
        if width > 0 { width - 2 } else { 0 }
    }
    
    pub fn draw_path(&self, display: &mut SharpDisplay, path: &Path) {
        let path_str = path.to_string_lossy();
        
        // Draw background for path area
        for y in 0..30 {
            for x in 0..400 {
                display.draw_pixel(x, y, Pixel::White);
            }
        }
        
        // Draw path text
        self.draw_text_line(display, LEFT_MARGIN, 5, &format!("Path: {}", path_str));
    }
    
    pub fn draw_item(&self, display: &mut SharpDisplay, item: &FileItem, y: usize, selected: bool) {
        let x = LEFT_MARGIN;
        
        // Clear item area
        for dy in 0..ITEM_HEIGHT {
            for dx in 0..380 {
                let pixel_y = y + dy;
                if pixel_y < 240 {
                    display.draw_pixel(x + dx, pixel_y, Pixel::White);
                }
            }
        }
        
        // Draw selection background if selected
        if selected {
            for dy in 0..ITEM_HEIGHT {
                for dx in 0..380 {
                    let pixel_y = y + dy;
                    if pixel_y < 240 {
                        display.draw_pixel(x + dx, pixel_y, Pixel::Black);
                    }
                }
            }
        }
        
        // Draw item text
        let display_name = Self::get_display_name(item);
        let icon = match item {
            FileItem::Directory(_) => "[DIR]",
            FileItem::ParentDirectory => "[UP]",
            FileItem::File(_) => "[FILE]",
        };
        
        let item_text = format!("{} {}", icon, display_name);
        
        // Draw text with appropriate color
        let text_color = if selected { Pixel::White } else { Pixel::Black };
        
        // Simple text drawing - we'll use a simplified version since we can't easily change colors
        // We'll draw by inverting pixel logic for selected items
        self.draw_text_line_with_color(display, x + 5, y + 5, &item_text, selected);
    }
    
    fn draw_text_line_with_color(&self, display: &mut SharpDisplay, x: usize, y: usize, text: &str, inverted: bool) {
        let mut current_x = x;
        for c in text.chars() {
            if c == ' ' {
                current_x += 8;
                continue;
            }
            
            if let Some(char_index) = Self::get_char_index(c) {
                if char_index < self.char_widths.len() {
                    let char_width = self.char_widths[char_index];
                    
                    if let Some((pixels, font_width, _)) = &self.font_bitmap {
                        let chars_per_row = self.chars_per_row;
                        let char_height = self.font_char_height;
                        
                        let grid_x = char_index % chars_per_row;
                        let grid_y = char_index / chars_per_row;
                        
                        let src_x = grid_x * self.font_char_width;
                        let src_y = grid_y * char_height;
                        
                        let mut leftmost = self.font_char_width;
                        let mut rightmost = 0;
                        
                        for dx in 0..self.font_char_width {
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
                                        let should_draw = if inverted {
                                            // For inverted, draw where font is white (background)
                                            pixels[pixel_index] == Pixel::White
                                        } else {
                                            // For normal, draw where font is black (text)
                                            pixels[pixel_index] == Pixel::Black
                                        };
                                        
                                        if should_draw {
                                            let screen_x = current_x + dx - leftmost;
                                            let screen_y = y + dy;
                                            
                                            if screen_x < 400 && screen_y < 240 {
                                                let pixel_color = if inverted { Pixel::White } else { Pixel::Black };
                                                display.draw_pixel(screen_x, screen_y, pixel_color);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    current_x += char_width + 2;
                }
            } else {
                // Unknown character - skip
                current_x += 8;
            }
        }
    }
    
    fn get_display_name(item: &FileItem) -> String {
        match item {
            FileItem::Directory(path) => {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            }
            FileItem::ParentDirectory => "..".to_string(),
            FileItem::File(path) => {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            }
        }
    }
}

impl FileBrowser {
    pub fn new(start_path: &Path) -> Result<Self> {
        let renderer = FileBrowserRenderer::new()?;
        
        let mut browser = Self {
            current_path: start_path.to_path_buf(),
            items: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            renderer,
        };
        
        browser.refresh_items()?;
        Ok(browser)
    }
    
    pub fn refresh_items(&mut self) -> Result<()> {
        self.items.clear();
        
        // Add parent directory entry if not at root
        if self.current_path.parent().is_some() {
            self.items.push(FileItem::ParentDirectory);
        }
        
        // Read directory entries
        if let Ok(entries) = fs::read_dir(&self.current_path) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();
            
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            dirs.push(FileItem::Directory(path));
                        } else if metadata.is_file() {
                            // Show all files in file browser
                            files.push(FileItem::File(path));
                        }
                    }
                }
            }
            
            // Sort directories alphabetically
            dirs.sort_by(|a, b| {
                let a_name = FileBrowserRenderer::get_display_name(a);
                let b_name = FileBrowserRenderer::get_display_name(b);
                a_name.cmp(&b_name)
            });
            
            // Sort files alphabetically
            files.sort_by(|a, b| {
                let a_name = FileBrowserRenderer::get_display_name(a);
                let b_name = FileBrowserRenderer::get_display_name(b);
                a_name.cmp(&b_name)
            });
            
            // Add directories first, then files
            self.items.extend(dirs);
            self.items.extend(files);
        }
        
        // Reset selection
        self.selected_index = 0;
        self.scroll_offset = 0;
        
        Ok(())
    }
    
    pub fn get_current_path(&self) -> &Path {
        &self.current_path
    }
    
    pub fn get_selected_item(&self) -> Option<&FileItem> {
        self.items.get(self.selected_index)
    }
    
    pub fn navigate_into(&mut self) -> Result<bool> {
        if let Some(item) = self.get_selected_item() {
            match item {
                FileItem::Directory(path) => {
                    self.current_path = path.clone();
                    self.refresh_items()?;
                    return Ok(true);
                }
                FileItem::ParentDirectory => {
                    if let Some(parent) = self.current_path.parent() {
                        self.current_path = parent.to_path_buf();
                        self.refresh_items()?;
                        return Ok(true);
                    }
                }
                FileItem::File(_) => {
                    // File selected - signal to open it
                    return Ok(false);
                }
            }
        }
        Ok(false)
    }
    
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            
            // Scroll if needed
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }
    
    pub fn move_selection_down(&mut self) {
        if self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
            
            // Scroll if needed
            if self.selected_index >= self.scroll_offset + ITEMS_PER_PAGE {
                self.scroll_offset = self.selected_index - ITEMS_PER_PAGE + 1;
            }
        }
    }
    
    pub fn draw(&self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        // Draw current path at top
        self.renderer.draw_path(display, &self.current_path);
        
        // Draw file items
        let start_y = TOP_MARGIN;
        let visible_range = self.scroll_offset..(self.scroll_offset + ITEMS_PER_PAGE).min(self.items.len());
        
        for (i, item_idx) in visible_range.enumerate() {
            let y = start_y + i * ITEM_HEIGHT;
            let is_selected = item_idx == self.selected_index;
            
            self.renderer.draw_item(display, &self.items[item_idx], y, is_selected);
        }
        
        // Draw instructions at bottom
        self.draw_instructions(display);
        
        display.update()?;
        Ok(())
    }
    
    fn draw_instructions(&self, display: &mut SharpDisplay) {
        let instructions = "↑↓:Navigate  Enter:Select  Esc:Cancel";
        let y = 240 - 20;
        
        // Center the instructions
        let text_width = self.renderer.calculate_text_width(instructions);
        let x = (400 - text_width) / 2;
        
        self.renderer.draw_text_line_with_color(display, x, y, instructions, false);
    }
}