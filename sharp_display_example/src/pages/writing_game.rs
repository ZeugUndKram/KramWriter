#[derive(Debug, Clone)]
pub struct WritingDocument {
    // Document state
    text: String,
    cursor_position: usize,
    scroll_offset: usize,
    
    // Editing state
    dirty: bool,
    
    // Display state
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
            self.text.insert(self.cursor_position, c);
            self.cursor_position += 1;
            self.dirty = true;
        }
        self.update_lines();
    }
    
    pub fn insert_newline(&mut self) {
        self.text.insert(self.cursor_position, '\n');
        self.cursor_position += 1;
        self.dirty = true;
        self.update_lines();
    }
    
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.text.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.dirty = true;
            self.update_lines();
        }
    }
    
    pub fn delete_forward(&mut self) {
        if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
            self.dirty = true;
            self.update_lines();
        }
    }
    
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }
    
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
    }
    
    pub fn move_cursor_up(&mut self) {
        let current_line = self.get_current_line_index();
        if current_line > 0 {
            let prev_line_end = self.get_line_end_position(current_line - 1);
            let current_col = self.get_cursor_column();
            let prev_line_len = self.lines[current_line - 1].len();
            
            self.cursor_position = prev_line_end.saturating_sub(prev_line_len) + current_col.min(prev_line_len);
        }
    }
    
    pub fn move_cursor_down(&mut self) {
        let current_line = self.get_current_line_index();
        if current_line + 1 < self.lines.len() {
            let current_line_start = self.get_line_start_position(current_line);
            let next_line_start = self.get_line_start_position(current_line + 1);
            let current_col = self.cursor_position - current_line_start;
            let next_line_len = self.lines[current_line + 1].len();
            
            self.cursor_position = next_line_start + current_col.min(next_line_len);
        }
    }
    
    pub fn move_cursor_home(&mut self) {
        let current_line = self.get_current_line_index();
        let line_start = self.get_line_start_position(current_line);
        self.cursor_position = line_start;
    }
    
    pub fn move_cursor_end(&mut self) {
        let current_line = self.get_current_line_index();
        self.cursor_position = self.get_line_end_position(current_line);
    }
    
    fn get_current_line_index(&self) -> usize {
        let mut pos = 0;
        for (i, line) in self.lines.iter().enumerate() {
            pos += line.len() + 1; // +1 for newline
            if pos > self.cursor_position {
                return i;
            }
        }
        self.lines.len().saturating_sub(1)
    }
    
    fn get_cursor_column(&self) -> usize {
        let current_line = self.get_current_line_index();
        let line_start = self.get_line_start_position(current_line);
        self.cursor_position - line_start
    }
    
    fn get_line_start_position(&self, line_index: usize) -> usize {
        self.lines.iter().take(line_index).map(|l| l.len() + 1).sum()
    }
    
    fn get_line_end_position(&self, line_index: usize) -> usize {
        self.get_line_start_position(line_index) + self.lines[line_index].len()
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