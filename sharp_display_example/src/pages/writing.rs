use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use super::writing_game::WritingDocument;
use super::writing_renderer::WritingRenderer;

const MAX_VISIBLE_LINES: usize = 6;

pub struct WritingPage {
    document: WritingDocument,
    renderer: WritingRenderer,
    show_status_bar: bool,
    last_frame_time: std::time::Instant,
}

impl WritingPage {
    pub fn new() -> Result<Self> {
        Ok(Self {
            document: WritingDocument::new(),
            renderer: WritingRenderer::new()?,
            show_status_bar: true,
            last_frame_time: std::time::Instant::now(),
        })
    }
    
    pub fn load_file(&mut self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.document.load_text(content);
        self.document.set_file_path(path.to_string());
        Ok(())
    }
    
    fn draw_cursor(&self, display: &mut SharpDisplay) {
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
    
    fn handle_text_input(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
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
            Key::Char('\n') => {
                self.document.insert_newline();
                self.document.ensure_cursor_visible();
            }
            Key::PageUp => {
                if self.document.get_scroll_offset() > 0 {
                    let new_offset = self.document.get_scroll_offset().saturating_sub(MAX_VISIBLE_LINES);
                    // We'll handle scroll offset differently
                    self.document.ensure_cursor_visible();
                }
            }
            Key::PageDown => {
                let lines = self.document.get_lines();
                if self.document.get_scroll_offset() + MAX_VISIBLE_LINES < lines.len() {
                    let new_offset = (self.document.get_scroll_offset() + MAX_VISIBLE_LINES)
                        .min(lines.len().saturating_sub(1));
                    // We'll handle scroll offset differently
                    self.document.ensure_cursor_visible();
                }
            }
            Key::Ctrl('s') => {
                // Simple save functionality
                if let Some(path) = self.document.get_file_path() {
                    if let Err(e) = std::fs::write(path, self.document.get_text()) {
                        println!("Failed to save: {}", e);
                    } else {
                        self.document.mark_saved();
                        println!("Saved to {}", path);
                    }
                } else {
                    println!("No file path set. Use Ctrl+Shift+S to save as.");
                }
            }
            Key::Ctrl('o') => {
                println!("Open file dialog would appear here");
                // TODO: Implement file dialog
            }
            Key::Ctrl('n') => {
                self.document = WritingDocument::new();
            }
            Key::Ctrl('q') => {
                return Ok(Some(PageId::Menu));
            }
            Key::F1 => {
                self.show_status_bar = !self.show_status_bar;
            }
            _ => {}
        }
        
        Ok(None)
    }
}

impl Page for WritingPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        self.renderer.render_document(display, &self.document);
        self.draw_cursor(display);
        
        if self.show_status_bar {
            self.renderer.draw_status_bar(display, &self.document);
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
            Key::Esc => Ok(Some(PageId::Menu)),
            _ => self.handle_text_input(key),
        }
    }
}