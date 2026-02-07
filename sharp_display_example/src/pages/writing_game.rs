#[derive(Debug, Clone)]
pub struct WritingDocument {
    // Document state - store as String which handles UTF-8
    text: String,
    cursor_position: usize, // Position in bytes, not characters!
    scroll_offset: usize,
    
    // Editing state
    dirty: bool,
    
    // Display state - lines store the actual text
    lines: Vec<String>,
    visible_lines: usize,
    
    // File handling
    file_path: Option<String>,
}

impl WritingDocument {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            dirty: false,
            lines: vec![String::new()],
            visible_lines: 6,
            file_path: None,
        }
    }
    
    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.insert_newline();
        } else {
            // Convert char to string to ensure proper UTF-8 insertion
            let char_str = c.to_string();
            
            // Insert at the current byte position
            self.text.insert_str(self.cursor_position, &char_str);
            
            // Move cursor forward by the number of bytes in the character
            self.cursor_position += char_str.len();
            self.dirty = true;
        }
        self.update_lines();
    }
    
    pub fn insert_newline(&mut self) {
        // Insert newline character
        self.text.insert_str(self.cursor_position, "\n");
        self.cursor_position += 1; // Newline is 1 byte in UTF-8
        self.dirty = true;
        self.update_lines();
    }
    
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            // Find the start of the previous character
            let mut char_start = self.cursor_position;
            
            // Move back until we find a valid UTF-8 character boundary
            while char_start > 0 && !self.text.is_char_boundary(char_start) {
                char_start -= 1;
            }
            
            // Now char_start is at the beginning of a character
            if char_start < self.cursor_position {
                // Remove the character
                self.text.drain(char_start..self.cursor_position);
                self.cursor_position = char_start;
                self.dirty = true;
                self.update_lines();
            }
        }
    }
    
    pub fn delete_forward(&mut self) {
        if self.cursor_position < self.text.len() {
            // Find the end of the current character
            let mut char_end = self.cursor_position;
            
            // Move forward until we find a valid UTF-8 character boundary
            while char_end < self.text.len() && !self.text.is_char_boundary(char_end) {
                char_end += 1;
            }
            
            // Now find the end of this character
            if char_end < self.text.len() {
                // Find the start of the next character
                let next_char_start = char_end + 1;
                let mut next_char_boundary = next_char_start;
                
                while next_char_boundary < self.text.len() && !self.text.is_char_boundary(next_char_boundary) {
                    next_char_boundary += 1;
                }
                
                // Remove the character
                self.text.drain(char_end..next_char_boundary);
                self.dirty = true;
                self.update_lines();
            }
        }
    }
    
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            // Move back to the previous character boundary
            let mut new_pos = self.cursor_position - 1;
            while new_pos > 0 && !self.text.is_char_boundary(new_pos) {
                new_pos -= 1;
            }
            self.cursor_position = new_pos;
        }
    }
    
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            // Move forward to the next character boundary
            let mut new_pos = self.cursor_position + 1;
            while new_pos < self.text.len() && !self.text.is_char_boundary(new_pos) {
                new_pos += 1;
            }
            self.cursor_position = new_pos;
        }
    }
    
    pub fn move_cursor_up(&mut self) {
        let current_line = self.get_current_line_index();
        if current_line > 0 {
            let prev_line_end = self.get_line_end_position(current_line - 1);
            let current_col = self.get_cursor_column();
            let prev_line_len = self.get_line_byte_length(current_line - 1);
            
            // Calculate byte position in previous line
            let prev_line_byte_pos = self.get_line_start_byte_position(current_line - 1);
            let target_char_pos = current_col.min(self.get_line_char_count(current_line - 1));
            
            // Move to the target character position in previous line
            self.cursor_position = prev_line_byte_pos + self.char_index_to_byte_offset(current_line - 1, target_char_pos);
        }
    }
    
    pub fn move_cursor_down(&mut self) {
        let current_line = self.get_current_line_index();
        if current_line + 1 < self.lines.len() {
            let current_line_start = self.get_line_start_byte_position(current_line);
            let next_line_start = self.get_line_start_byte_position(current_line + 1);
            let current_col = self.get_cursor_column();
            let next_line_char_count = self.get_line_char_count(current_line + 1);
            
            let target_char_pos = current_col.min(next_line_char_count);
            
            // Move to the target character position in next line
            self.cursor_position = next_line_start + self.char_index_to_byte_offset(current_line + 1, target_char_pos);
        }
    }
    
    pub fn move_cursor_home(&mut self) {
        let current_line = self.get_current_line_index();
        let line_start = self.get_line_start_byte_position(current_line);
        self.cursor_position = line_start;
    }
    
    pub fn move_cursor_end(&mut self) {
        let current_line = self.get_current_line_index();
        self.cursor_position = self.get_line_end_byte_position(current_line);
    }
    
    // Helper method to convert character index to byte offset within a line
    fn char_index_to_byte_offset(&self, line_index: usize, char_index: usize) -> usize {
        let line = &self.lines[line_index];
        let mut byte_offset = 0;
        let mut char_count = 0;
        
        for c in line.chars() {
            if char_count >= char_index {
                break;
            }
            byte_offset += c.len_utf8();
            char_count += 1;
        }
        
        byte_offset
    }
    
    // Get number of characters in a line
    fn get_line_char_count(&self, line_index: usize) -> usize {
        self.lines[line_index].chars().count()
    }
    
    // Get byte length of a line (including newline)
    fn get_line_byte_length(&self, line_index: usize) -> usize {
        self.lines[line_index].len() + 1 // +1 for newline
    }
    
    // Make these public
    pub fn get_current_line_index(&self) -> usize {
        let mut byte_pos = 0;
        for (i, line) in self.lines.iter().enumerate() {
            byte_pos += line.len() + 1; // +1 for newline
            if byte_pos > self.cursor_position {
                return i;
            }
        }
        self.lines.len().saturating_sub(1)
    }
    
    // Make this public
    pub fn get_cursor_column(&self) -> usize {
        let current_line = self.get_current_line_index();
        let line_start_byte = self.get_line_start_byte_position(current_line);
        
        // Count characters from line start to cursor
        let line_text = &self.lines[current_line];
        let prefix = &self.text[line_start_byte..self.cursor_position];
        
        // Count characters in the prefix
        prefix.chars().count()
    }
    
    // Get byte position of line start
    fn get_line_start_byte_position(&self, line_index: usize) -> usize {
        self.lines.iter().take(line_index).map(|l| l.len() + 1).sum()
    }
    
    // Get byte position of line end (including newline)
    fn get_line_end_byte_position(&self, line_index: usize) -> usize {
        self.get_line_start_byte_position(line_index) + self.lines[line_index].len()
    }
    
    fn update_lines(&mut self) {
        self.lines = self.text.split('\n').map(|s| s.to_string()).collect();
    }
    
    pub fn ensure_cursor_visible(&mut self) {
        let current_line = self.get_current_line_index();
        
        if current_line < self.scroll_offset {
            self.scroll_offset = current_line;
        } else if current_line >= self.scroll_offset + self.visible_lines {
            self.scroll_offset = current_line - self.visible_lines + 1;
        }
    }
    
    pub fn get_text(&self) -> &str {
        &self.text
    }
    
    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }
    
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }
    
    pub fn get_scroll_offset(&self) -> usize {
        self.scroll_offset
    }
    
    pub fn set_visible_lines(&mut self, visible_lines: usize) {
        self.visible_lines = visible_lines;
    }
    
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }
    
    pub fn get_file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }
    
    pub fn set_file_path(&mut self, path: String) {
        self.file_path = Some(path);
    }
    
    pub fn load_text(&mut self, text: String) {
        self.text = text;
        self.cursor_position = 0;
        self.scroll_offset = 0;
        self.dirty = false;
        self.update_lines();
    }
}