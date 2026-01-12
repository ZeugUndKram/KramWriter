// src/main.rs
use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Bus, SlaveSelect};
use std::io::{self, Write};
use std::collections::VecDeque;

const WIDTH: usize = 400;
const HEIGHT: usize = 240;
const CHAR_WIDTH: usize = 6;
const CHAR_HEIGHT: usize = 8;
const COLS: usize = WIDTH / CHAR_WIDTH;
const ROWS: usize = HEIGHT / CHAR_HEIGHT;
const MARGIN: usize = 2;

struct TextEditor {
    display: MemoryDisplay,
    buffer: MemoryDisplayBuffer,
    lines: Vec<String>,
    cursor_x: usize,    // Character position in line
    cursor_y: usize,    // Line number
    scroll_x: usize,    // Horizontal scroll
    scroll_y: usize,    // Vertical scroll
    modified: bool,
}

impl TextEditor {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let display = MemoryDisplay::new(
            Bus::Spi0,
            SlaveSelect::Ss0,
            6,
            WIDTH,
            HEIGHT as u8,
        )?;
        
        let buffer = MemoryDisplayBuffer::new(WIDTH, HEIGHT as u8);
        
        Ok(TextEditor {
            display,
            buffer,
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            modified: false,
        })
    }
    
    fn clear_screen(&mut self) {
        self.buffer.fill(Pixel::White);
    }
    
    fn draw(&mut self) {
        self.clear_screen();
        
        // Draw visible lines
        for screen_y in 0..ROWS {
            let line_idx = self.scroll_y + screen_y;
            if line_idx < self.lines.len() {
                let line = &self.lines[line_idx];
                
                // Apply horizontal scrolling
                let start_char = self.scroll_x;
                let end_char = (self.scroll_x + COLS).min(line.len());
                
                for (char_idx, c) in line.chars().skip(start_char).take(COLS).enumerate() {
                    let x = char_idx * CHAR_WIDTH + MARGIN;
                    let y = screen_y * CHAR_HEIGHT + MARGIN;
                    self.draw_char(x, y, c);
                }
            }
        }
        
        // Draw cursor
        let cursor_screen_x = self.cursor_x.saturating_sub(self.scroll_x);
        let cursor_screen_y = self.cursor_y.saturating_sub(self.scroll_y);
        
        if cursor_screen_x < COLS && cursor_screen_y < ROWS {
            let x = cursor_screen_x * CHAR_WIDTH + MARGIN;
            let y = cursor_screen_y * CHAR_HEIGHT + MARGIN;
            
            // Draw vertical line cursor
            for dy in 0..CHAR_HEIGHT {
                if y + dy < HEIGHT {
                    self.buffer.set_pixel(x, (y + dy) as u8, Pixel::Black);
                    if x + 1 < WIDTH {
                        self.buffer.set_pixel(x + 1, (y + dy) as u8, Pixel::Black);
                    }
                }
            }
        }
        
        self.display.update(&self.buffer).unwrap();
    }
    
    fn draw_char(&mut self, x: usize, y: usize, c: char) {
        // Simple 5x7 font
        let pattern = Self::char_pattern(c);
        
        for dy in 0..7 {
            let row = pattern[dy];
            for dx in 0..5 {
                if (row >> (4 - dx)) & 1 == 1 {
                    let px = x + dx;
                    let py = y + dy;
                    if px < WIDTH && py < HEIGHT {
                        self.buffer.set_pixel(px, py as u8, Pixel::Black);
                    }
                }
            }
        }
    }
    
    fn char_pattern(c: char) -> [u8; 7] {
        let c = c.to_ascii_uppercase();
        match c {
            'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
            'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
            'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
            'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
            'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
            'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
            'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E],
            'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
            'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
            'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
            'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
            'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
            'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
            'N' => [0x11, 0x19, 0x19, 0x15, 0x13, 0x13, 0x11],
            'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
            'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
            'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
            'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
            'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
            'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
            'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
            'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
            'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
            'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
            'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
            'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
            '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
            '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
            '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
            '3' => [0x0E, 0x11, 0x01, 0x06, 0x01, 0x11, 0x0E],
            '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
            '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
            '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
            '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
            '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
            '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
            '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04],
            ',' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x04],
            '!' => [0x04, 0x04, 0x04, 0x04, 0x00, 0x00, 0x04],
            '?' => [0x0E, 0x11, 0x02, 0x04, 0x04, 0x00, 0x04],
            ';' => [0x00, 0x04, 0x00, 0x00, 0x04, 0x04, 0x00],
            ':' => [0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00],
            '\'' => [0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00],
            '"' => [0x0A, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00],
            '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
            '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F],
            '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
            '=' => [0x00, 0x00, 0x1F, 0x00, 0x1F, 0x00, 0x00],
            '(' => [0x02, 0x04, 0x08, 0x08, 0x08, 0x04, 0x02],
            ')' => [0x08, 0x04, 0x02, 0x02, 0x02, 0x04, 0x08],
            '[' => [0x0E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x0E],
            ']' => [0x0E, 0x02, 0x02, 0x02, 0x02, 0x02, 0x0E],
            '{' => [0x02, 0x04, 0x04, 0x08, 0x04, 0x04, 0x02],
            '}' => [0x08, 0x04, 0x04, 0x02, 0x04, 0x04, 0x08],
            '<' => [0x00, 0x02, 0x04, 0x08, 0x04, 0x02, 0x00],
            '>' => [0x00, 0x08, 0x04, 0x02, 0x04, 0x08, 0x00],
            '/' => [0x00, 0x01, 0x02, 0x04, 0x08, 0x10, 0x00],
            '\\' => [0x00, 0x10, 0x08, 0x04, 0x02, 0x01, 0x00],
            '|' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
            '@' => [0x0E, 0x11, 0x17, 0x15, 0x17, 0x10, 0x0E],
            '#' => [0x0A, 0x0A, 0x1F, 0x0A, 0x1F, 0x0A, 0x0A],
            '$' => [0x04, 0x0F, 0x14, 0x0E, 0x05, 0x1E, 0x04],
            '%' => [0x18, 0x19, 0x02, 0x04, 0x08, 0x13, 0x03],
            '&' => [0x0C, 0x12, 0x14, 0x08, 0x15, 0x12, 0x0D],
            '*' => [0x00, 0x04, 0x15, 0x0E, 0x15, 0x04, 0x00],
            '^' => [0x04, 0x0A, 0x11, 0x00, 0x00, 0x00, 0x00],
            '~' => [0x00, 0x00, 0x0A, 0x15, 0x00, 0x00, 0x00],
            '`' => [0x08, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00],
            ' ' => [0x00; 7],
            _ => [0x15, 0x0A, 0x15, 0x0A, 0x15, 0x0A, 0x15], // Unknown char
        }
    }
    
    fn insert_char(&mut self, c: char) {
        if self.cursor_y >= self.lines.len() {
            self.lines.push(String::new());
        }
        
        let line = &mut self.lines[self.cursor_y];
        
        // Handle line wrapping
        if line.len() >= COLS * 2 { // Allow some overflow before wrapping
            // Find last space to wrap at
            if let Some(last_space) = line.rfind(' ') {
                if last_space > COLS {
                    let remainder = line.split_off(last_space + 1);
                    self.cursor_x = last_space + 1;
                    self.insert_new_line();
                    self.lines[self.cursor_y] = remainder;
                    self.cursor_x = 0;
                }
            } else {
                // No space found, force wrap
                let remainder = line.split_off(COLS);
                self.insert_new_line();
                self.lines[self.cursor_y] = remainder;
                self.cursor_x = 0;
            }
        }
        
        if self.cursor_x <= line.len() {
            line.insert(self.cursor_x, c);
            self.cursor_x += 1;
            self.modified = true;
        }
        
        self.ensure_cursor_visible();
    }
    
    fn insert_new_line(&mut self) {
        if self.cursor_y >= self.lines.len() {
            self.lines.push(String::new());
        }
        
        let line = &self.lines[self.cursor_y];
        let (left, right) = line.split_at(self.cursor_x);
        
        self.lines[self.cursor_y] = left.to_string();
        self.lines.insert(self.cursor_y + 1, right.to_string());
        
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.modified = true;
        
        self.ensure_cursor_visible();
    }
    
    fn delete_char(&mut self) {
        if self.cursor_x > 0 {
            // Delete character before cursor
            let line = &mut self.lines[self.cursor_y];
            line.remove(self.cursor_x - 1);
            self.cursor_x -= 1;
            self.modified = true;
        } else if self.cursor_y > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            let prev_line = &mut self.lines[self.cursor_y];
            self.cursor_x = prev_line.len();
            prev_line.push_str(&current_line);
            self.modified = true;
        }
        
        self.ensure_cursor_visible();
    }
    
    fn move_cursor(&mut self, dx: isize, dy: isize) {
        let new_y = (self.cursor_y as isize + dy).max(0) as usize;
        
        if new_y < self.lines.len() {
            self.cursor_y = new_y;
            let line_len = self.lines[self.cursor_y].len();
            self.cursor_x = (self.cursor_x as isize + dx).max(0).min(line_len as isize) as usize;
        } else if new_y == self.lines.len() && dy > 0 {
            // Move to end of document
            self.cursor_y = self.lines.len() - 1;
            self.cursor_x = self.lines[self.cursor_y].len();
        }
        
        self.ensure_cursor_visible();
    }
    
    fn ensure_cursor_visible(&mut self) {
        // Vertical scrolling
        if self.cursor_y < self.scroll_y {
            self.scroll_y = self.cursor_y;
        } else if self.cursor_y >= self.scroll_y + ROWS {
            self.scroll_y = self.cursor_y - ROWS + 1;
        }
        
        // Horizontal scrolling
        if self.cursor_x < self.scroll_x {
            self.scroll_x = self.cursor_x;
        } else if self.cursor_x >= self.scroll_x + COLS {
            self.scroll_x = self.cursor_x - COLS + 1;
        }
    }
    
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use termion::input::TermRead;
        use termion::raw::IntoRawMode;
        
        let stdin = io::stdin();
        let mut stdout = io::stdout().into_raw_mode()?;
        
        write!(stdout, "{}", termion::cursor::Hide)?;
        write!(stdout, "{}", termion::clear::All)?;
        write!(stdout, "Sharp Display Editor (Ctrl+Q to quit)\r\n")?;
        write!(stdout, "Arrow keys to move, Backspace to delete\r\n")?;
        
        stdout.flush()?;
        
        self.display.clear()?;
        self.draw();
        
        for key in stdin.keys() {
            match key.unwrap() {
                termion::event::Key::Char('\n') => {
                    self.insert_new_line();
                }
                termion::event::Key::Char(c) => {
                    if c == '\x11' { // Ctrl+Q
                        break;
                    }
                    self.insert_char(c);
                }
                termion::event::Key::Backspace => {
                    self.delete_char();
                }
                termion::event::Key::Left => {
                    self.move_cursor(-1, 0);
                }
                termion::event::Key::Right => {
                    self.move_cursor(1, 0);
                }
                termion::event::Key::Up => {
                    self.move_cursor(0, -1);
                }
                termion::event::Key::Down => {
                    self.move_cursor(0, 1);
                }
                termion::event::Key::Home => {
                    self.cursor_x = 0;
                    self.ensure_cursor_visible();
                }
                termion::event::Key::End => {
                    self.cursor_x = self.lines[self.cursor_y].len();
                    self.ensure_cursor_visible();
                }
                termion::event::Key::PageUp => {
                    self.move_cursor(0, -(ROWS as isize));
                }
                termion::event::Key::PageDown => {
                    self.move_cursor(0, ROWS as isize);
                }
                termion::event::Key::Ctrl('s') => {
                    // Save functionality could be added here
                    write!(stdout, "\r\n(Not implemented) Press any key...")?;
                    stdout.flush()?;
                }
                _ => {}
            }
            
            self.draw();
            
            // Update status on terminal
            write!(stdout, "{}", termion::cursor::Goto(1, 4))?;
            write!(stdout, "{}", termion::clear::CurrentLine)?;
            write!(stdout, "Line: {}/{} Col: {} {}", 
                self.cursor_y + 1, 
                self.lines.len(),
                self.cursor_x + 1,
                if self.modified { "[MODIFIED]" } else { "" })?;
            
            stdout.flush()?;
        }
        
        write!(stdout, "{}", termion::cursor::Show)?;
        write!(stdout, "\r\nExiting editor...\r\n")?;
        
        self.display.clear()?;
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = TextEditor::new()?;
    editor.run()
}