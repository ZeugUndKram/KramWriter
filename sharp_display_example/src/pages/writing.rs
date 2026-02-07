use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use super::writing_game::WritingDocument;
use super::writing_renderer::WritingRenderer;
use super::file_browser::FileBrowser;
use std::path::PathBuf;

const MAX_VISIBLE_LINES: usize = 6;

pub struct WritingPage {
    document: WritingDocument,
    renderer: WritingRenderer,
    show_status_bar: bool,
    last_frame_time: std::time::Instant,
    mode: WritingMode,
    file_browser: Option<FileBrowser>,
}

#[derive(PartialEq)]
enum WritingMode {
    Editing,
    FileBrowser,
    SaveAs,
}

impl WritingPage {
    pub fn new() -> Result<Self> {
        Ok(Self {
            document: WritingDocument::new(),
            renderer: WritingRenderer::new()?,
            show_status_bar: true,
            last_frame_time: std::time::Instant::now(),
            mode: WritingMode::Editing,
            file_browser: None,
        })
    }
    
    fn draw_cursor(&self, display: &mut SharpDisplay) {
        if self.mode != WritingMode::Editing {
            return;
        }
        
        let current_line = self.document.get_current_line_index();
        let cursor_col = self.document.get_cursor_column();
        let scroll_offset = self.document.get_scroll_offset();
        
        if current_line >= scroll_offset && current_line < scroll_offset + MAX_VISIBLE_LINES {
            let lines = self.document.get_lines();
            let line_text = &lines[current_line];
            
            // Get wrapped segments for this line
            let wrapped_segments = self.renderer.calculate_wrapped_line_positions(line_text);
            
            // Find which wrapped segment contains our cursor
            let mut current_col = cursor_col;
            let mut segment_y = self.renderer.get_top_margin() + 
                (current_line - scroll_offset) * (self.renderer.get_font_height() + self.renderer.get_line_spacing());
            
            for (segment_text, _) in wrapped_segments {
                let segment_len = segment_text.len();
                
                if current_col <= segment_len {
                    // Cursor is in this segment
                    let before_cursor: String = segment_text.chars().take(current_col).collect();
                    let cursor_x = self.renderer.get_left_margin() + 
                        self.renderer.calculate_text_width(&before_cursor);
                    
                    // Draw vertical cursor line
                    for dy in 0..self.renderer.get_font_height() {
                        display.draw_pixel(cursor_x, segment_y + dy, rpi_memory_display::Pixel::Black);
                    }
                    break;
                } else {
                    current_col -= segment_len;
                    segment_y += self.renderer.get_font_height() + self.renderer.get_line_spacing();
                }
            }
        }
    }
    
    fn show_file_browser(&mut self, mode: WritingMode) -> Result<()> {
        self.mode = mode;
        let start_path = PathBuf::from("/home/kramwriter/KramWriter/documents");
        
        // Create directory if it doesn't exist
        if !start_path.exists() {
            std::fs::create_dir_all(&start_path)?;
        }
        
        self.file_browser = Some(FileBrowser::new(&start_path)?);
        Ok(())
    }
    
    fn hide_file_browser(&mut self) {
        self.mode = WritingMode::Editing;
        self.file_browser = None;
    }
    
    fn handle_file_browser_input(&mut self, key: Key) -> Result<Option<PageId>> {
        if let Some(browser) = &mut self.file_browser {
            match key {
                Key::Up => {
                    browser.move_selection_up();
                }
                Key::Down => {
                    browser.move_selection_down();
                }
                Key::Char('\n') => {
                    // Enter pressed
                    if browser.navigate_into()? {
                        // Navigated into directory
                    } else {
                        // File selected or couldn't navigate
                        if let Some(item) = browser.get_selected_item() {
                            match item {
                                super::file_browser::FileItem::File(path) => {
                                    // Load the file
                                    let content = std::fs::read_to_string(path)?;
                                    self.document.load_text(content);
                                    self.document.set_file_path(path.to_string_lossy().to_string());
                                    self.hide_file_browser();
                                }
                                _ => {
                                    // Shouldn't happen, but just in case
                                    self.hide_file_browser();
                                }
                            }
                        }
                    }
                }
                Key::Esc => {
                    self.hide_file_browser();
                }
                _ => {}
            }
        }
        Ok(None)
    }
    
    fn handle_editing_input(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
            Key::Char('\n') => {
                self.document.insert_newline();
                self.document.ensure_cursor_visible();
            }
            Key::Char('s') if self.show_status_bar => {
                // Use 's' key to toggle status bar
                self.show_status_bar = false;
            }
            Key::Char('S') if !self.show_status_bar => {
                // Use 'S' key to toggle status bar back on
                self.show_status_bar = true;
            }
            Key::Char(c) => {
                self.document.insert_char(c);
                self.document.ensure_cursor_visible();
            }
            Key::Backspace => {
                self.document.delete_char();
                self.document.ensure_cursor_visible();
            }
            Key::Delete => {
                self.document.delete_forward();
                self.document.ensure_cursor_visible();
            }
            Key::Left => {
                self.document.move_cursor_left();
                self.document.ensure_cursor_visible();
            }
            Key::Right => {
                self.document.move_cursor_right();
                self.document.ensure_cursor_visible();
            }
            Key::Up => {
                self.document.move_cursor_up();
                self.document.ensure_cursor_visible();
            }
            Key::Down => {
                self.document.move_cursor_down();
                self.document.ensure_cursor_visible();
            }
            Key::Home => {
                self.document.move_cursor_home();
                self.document.ensure_cursor_visible();
            }
            Key::End => {
                self.document.move_cursor_end();
                self.document.ensure_cursor_visible();
            }
            Key::PageUp => {
                if self.document.get_scroll_offset() > 0 {
                    self.document.ensure_cursor_visible();
                }
            }
            Key::PageDown => {
                self.document.ensure_cursor_visible();
            }
            Key::Ctrl('s') => {
                // Save file
                if let Some(path) = self.document.get_file_path() {
                    let path_copy = path.to_string();
                    if let Err(e) = std::fs::write(&path_copy, self.document.get_text()) {
                        println!("Failed to save: {}", e);
                    } else {
                        self.document.mark_saved();
                        println!("Saved to {}", path_copy);
                    }
                } else {
                    // No file path - show save as dialog
                    self.show_file_browser(WritingMode::SaveAs)?;
                }
            }
            Key::Ctrl('o') => {
                // Open file
                self.show_file_browser(WritingMode::FileBrowser)?;
            }
            Key::Ctrl('n') => {
                // New file
                self.document = WritingDocument::new();
            }
            Key::Ctrl('q') => {
                return Ok(Some(PageId::Menu));
            }
            _ => {}
        }
        
        Ok(None)
    }
}

impl Page for WritingPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        match self.mode {
            WritingMode::Editing => {
                self.renderer.render_document(display, &self.document);
                self.draw_cursor(display);
                
                if self.show_status_bar {
                    self.renderer.draw_status_bar(display, &self.document);
                }
            }
            tingMode::FileBrowser | WritingMode::SaveAs => {
    if let Some(browser) = &self.file_browser {
        let _ = browser.draw(display);
        
        // Draw mode indicator at top
        let mode_text = match self.mode {
            WritingMode::SaveAs => "SAVE AS - Select file or press Enter for new file",
            _ => "OPEN FILE - Select file and press Enter",
        };
        
        // Draw mode text using a simple method
        let y = 30;
        let text_width = self.renderer.calculate_text_width(mode_text);
        let x = (400 - text_width) / 2;
        
        // Clear area for mode text
        for dy in 0..self.renderer.get_font_height() {
            for dx in 0..text_width + 20 {
                if x + dx < 400 && y + dy < 240 {
                    display.draw_pixel(x + dx, y + dy, rpi_memory_display::Pixel::White);
                }
            }
        }
        
        // Draw mode text
        let mut current_x = x;
        for c in mode_text.chars() {
            if current_x < 400 {
                self.renderer.draw_char_cropped(display, current_x, y, c);
                let char_width = if let Some(char_index) = crate::pages::writing_renderer::WritingRenderer::get_char_index(c) {
                    if char_index < self.renderer.char_widths.len() { 
                        self.renderer.char_widths[char_index] 
                    } else { 
                        8
                    }
                } else {
                    8
                };
                current_x += char_width + 2;
            }
        }
    }
}
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match self.mode {
            WritingMode::Editing => {
                match key {
                    Key::Esc => Ok(Some(PageId::Menu)),
                    _ => self.handle_editing_input(key),
                }
            }
            WritingMode::FileBrowser | WritingMode::SaveAs => {
                self.handle_file_browser_input(key)
            }
        }
    }
}