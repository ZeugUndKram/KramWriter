use rpi_memory_display::Pixel;
use crate::display::SharpDisplay;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;

const ITEMS_PER_PAGE: usize = 8;
const ITEM_HEIGHT: usize = 25;
const LEFT_MARGIN: usize = 10;
const TOP_MARGIN: usize = 10;

pub struct FileBrowser {
    current_path: PathBuf,
    items: Vec<FileItem>,
    selected_index: usize,
    scroll_offset: usize,
}

#[derive(Debug, Clone)]
pub enum FileItem {
    Directory(PathBuf),
    File(PathBuf),
    ParentDirectory,
}

impl FileBrowser {
    pub fn new(start_path: &Path) -> Result<Self> {
        let mut browser = Self {
            current_path: start_path.to_path_buf(),
            items: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
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
        let entries = fs::read_dir(&self.current_path)?;
        
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let metadata = entry.metadata()?;
                
                if metadata.is_dir() {
                    dirs.push(FileItem::Directory(path));
                } else if metadata.is_file() {
                    // Only show text files and files without extension
                    if let Some(ext) = path.extension() {
                        if ext == "txt" || ext == "md" || ext == "rs" {
                            files.push(FileItem::File(path));
                        }
                    } else {
                        // Files without extension
                        files.push(FileItem::File(path));
                    }
                }
            }
        }
        
        // Sort directories alphabetically
        dirs.sort_by(|a, b| {
            let a_name = Self::get_display_name(a);
            let b_name = Self::get_display_name(b);
            a_name.cmp(&b_name)
        });
        
        // Sort files alphabetically
        files.sort_by(|a, b| {
            let a_name = Self::get_display_name(a);
            let b_name = Self::get_display_name(b);
            a_name.cmp(&b_name)
        });
        
        // Add directories first, then files
        self.items.extend(dirs);
        self.items.extend(files);
        
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
        self.draw_path(display);
        
        // Draw file items
        let start_y = TOP_MARGIN + 30;
        let visible_range = self.scroll_offset..(self.scroll_offset + ITEMS_PER_PAGE).min(self.items.len());
        
        for (i, item_idx) in visible_range.enumerate() {
            let y = start_y + i * ITEM_HEIGHT;
            let is_selected = item_idx == self.selected_index;
            
            self.draw_item(display, &self.items[item_idx], y, is_selected);
        }
        
        display.update()?;
        Ok(())
    }
    
    fn draw_path(&self, display: &mut SharpDisplay) {
        let path_str = self.current_path.to_string_lossy();
        
        // Simple text drawing for path
        let mut x = LEFT_MARGIN;
        let y = TOP_MARGIN;
        
        for c in path_str.chars() {
            if x < 400 {
                // Draw a simple representation of each character
                if c != ' ' {
                    for dy in 0..8 {
                        for dx in 0..6 {
                            display.draw_pixel(x + dx, y + dy, Pixel::Black);
                        }
                    }
                }
                x += 8;
            }
        }
    }
    
    fn draw_item(&self, display: &mut SharpDisplay, item: &FileItem, y: usize, selected: bool) {
        let x = LEFT_MARGIN;
        
        // Draw selection background
        if selected {
            for dx in 0..380 {
                for dy in 0..ITEM_HEIGHT {
                    if y + dy < 240 {
                        display.draw_pixel(x + dx, y + dy, Pixel::Black);
                    }
                }
            }
        }
        
        // Draw item icon and name
        let display_name = Self::get_display_name(item);
        let icon = match item {
            FileItem::Directory(_) => "[DIR]",
            FileItem::ParentDirectory => "[UP]",
            FileItem::File(_) => "[FILE]",
        };
        
        let item_text = format!("{} {}", icon, display_name);
        
        // Simple text drawing
        let mut text_x = x + 5;
        for c in item_text.chars() {
            if text_x < 400 && y + 8 < 240 {
                // Draw character as a simple 6x8 bitmap
                if c != ' ' {
                    for dy in 0..8 {
                        for dx in 0..6 {
                            let pixel_color = if selected { Pixel::White } else { Pixel::Black };
                            display.draw_pixel(text_x + dx, y + dy + 5, pixel_color);
                        }
                    }
                }
                text_x += 7;
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