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

struct TextEditor {
    display: MemoryDisplay,
    buffer: MemoryDisplayBuffer,
    text: Vec<String>,  // Lines of text
    cursor_x: usize,    // Column position
    cursor_y: usize,    // Row position
    scroll_offset: usize, // How many lines we've scrolled
}

impl TextEditor {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let display = MemoryDisplay::new(
            Bus::Spi0,
            SlaveSelect::Ss0,
            25,
            WIDTH,
            HEIGHT as u8,
        )?;
        
        let buffer = MemoryDisplayBuffer::new(WIDTH, HEIGHT as u8);
        
        Ok(TextEditor {
            display,
            buffer,
            text: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset: 0,
        })
    }
    
    fn draw_char(&mut self, x: usize, y: usize, c: char) {
        let bitmap = Self::char_to_bitmap(c);
        let screen_x = x * CHAR_WIDTH;
        let screen_y = y * CHAR_HEIGHT;
        
        for dy in 0..CHAR_HEIGHT {
            for dx in 0..CHAR_WIDTH {
                if dy < bitmap.len() && dx < bitmap[dy].len() && bitmap[dy][dx] {
                    self.buffer.set_pixel(
                        screen_x + dx,
                        (screen_y + dy) as u8,
                        Pixel::Black
                    );
                }
            }
        }
    }
    
    fn char_to_bitmap(c: char) -> [[bool; 6]; 8] {
        // Simple 6x8 font (ASCII only)
        match c {
            'A' => [
                [false, true, true, true, false, false],
                [true, false, false, false, true, false],
                [true, false, false, false, true, false],
                [true, true, true, true, true, false],
                [true, false, false, false, true, false],
                [true, false, false, false, true, false],
                [true, false, false, false, true, false],
                [true, false, false, false, true, false],
            ],
            'B' => [
                [true, true, true, true, false, false],
                [true, false, false, false, true, false],
                [true, false, false, false, true, false],
                [true, true, true, true, false, false],
                [true, false, false, false, true, false],
                [true, false, false, false, true, false],
                [true, false, false, false, true, false],
                [true, true, true, true, false, false],
            ],
            // Add more characters as needed...
            ' ' => [[false; 6]; 8],
            _ => [[false; 6]; 8], // Default empty
        }
    }
    
    fn clear_screen(&mut self) {
        self.buffer.fill(Pixel::White);
    }
    
    fn draw_text(&mut self) {
        self.clear_screen();
        
        // Draw visible lines
        for line_idx in 0..ROWS {
            let text_line_idx = line_idx + self.scroll_offset;
            if text_line_idx < self.text.len() {
                let line = &self.text[text_line_idx];
                for (col, c) in line.chars().enumerate().take(COLS) {
                    self.draw_char(col, line_idx, c);
                }
            }
        }
        
        // Draw cursor
        let cursor_screen_y = self.cursor_y - self.scroll_offset;
        if cursor_screen_y < ROWS {
            for dy in 0..CHAR_HEIGHT {
                self.buffer.set_pixel(
                    self.cursor_x * CHAR_WIDTH,
                    (cursor_screen_y * CHAR_HEIGHT + dy) as u8,
                    Pixel::Black
                );
            }
        }
        
        self.display.update(&self.buffer).unwrap();
    }
    
    fn insert_char(&mut self, c: char) {
        if self.cursor_y >= self.text.len() {
            self.text.push(String::new());
        }
        
        let line = &mut self.text[self.cursor_y];
        if self.cursor_x <= line.len() {
            line.insert(self.cursor_x, c);
            self.cursor_x += 1;
        }
    }
    
    fn delete_char(&mut self) {
        if self.cursor_x > 0 && self.cursor_y < self.text.len() {
            let line = &mut self.text[self.cursor_y];
            line.remove(self.cursor_x - 1);
            self.cursor_x -= 1;
        } else if self.cursor_x == 0 && self.cursor_y > 0 {
            // Merge with previous line
            let current_line = self.text.remove(self.cursor_y);
            self.cursor_y -= 1;
            let prev_line = &mut self.text[self.cursor_y];
            self.cursor_x = prev_line.len();
            prev_line.push_str(&current_line);
        }
    }
    
    fn new_line(&mut self) {
        let line = if self.cursor_y < self.text.len() {
            let line = self.text[self.cursor_y].clone();
            let (left, right) = line.split_at(self.cursor_x);
            self.text[self.cursor_y] = left.to_string();
            right.to_string()
        } else {
            String::new()
        };
        
        self.text.insert(self.cursor_y + 1, line);
        self.cursor_y += 1;
        self.cursor_x = 0;
    }
    
    fn move_cursor(&mut self, dx: isize, dy: isize) {
        let new_x = self.cursor_x as isize + dx;
        let new_y = self.cursor_y as isize + dy;
        
        if new_y >= 0 && new_y < self.text.len() as isize {
            self.cursor_y = new_y as usize;
            let line_len = self.text[self.cursor_y].len();
            self.cursor_x = new_x.max(0).min(line_len as isize) as usize;
        }
        
        // Scroll if needed
        if self.cursor_y < self.scroll_offset {
            self.scroll_offset = self.cursor_y;
        } else if self.cursor_y >= self.scroll_offset + ROWS {
            self.scroll_offset = self.cursor_y - ROWS + 1;
        }
    }
    
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::{stdin, stdout};
        use termion::input::TermRead;
        use termion::raw::IntoRawMode;
        
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;
        
        write!(stdout, "{}", termion::cursor::Hide)?;
        
        self.draw_text();
        
        for c in stdin.keys() {
            match c.unwrap() {
                termion::event::Key::Char('\n') => {
                    self.new_line();
                }
                termion::event::Key::Char(c) => {
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
                termion::event::Key::Ctrl('c') => break,
                _ => {}
            }
            
            self.draw_text();
            stdout.flush()?;
        }
        
        write!(stdout, "{}", termion::cursor::Show)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = TextEditor::new()?;
    editor.run()
}