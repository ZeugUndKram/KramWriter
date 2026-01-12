// src/main.rs
use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Bus, SlaveSelect};

const WIDTH: usize = 400;
const HEIGHT: usize = 240;

struct TextEditor {
    display: MemoryDisplay,
    buffer: MemoryDisplayBuffer,
    cursor_x: usize,
    cursor_y: usize,
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
            cursor_x: 0,
            cursor_y: 0,
        })
    }
    
    fn clear_screen(&mut self) {
        self.buffer.fill(Pixel::White);
    }
    
    fn draw_cursor(&mut self) {
        let x = self.cursor_x * 6;
        let y = self.cursor_y * 8;
        
        // Draw vertical line cursor
        for dy in 0..8 {
            if y + dy < HEIGHT {
                self.buffer.set_pixel(x, (y + dy) as u8, Pixel::Black);
                if x + 1 < WIDTH {
                    self.buffer.set_pixel(x + 1, (y + dy) as u8, Pixel::Black);
                }
            }
        }
    }
    
    fn draw_text(&mut self, text: &str) {
        self.clear_screen();
        
        // Simple: just draw the text at cursor position
        let mut x = 0;
        let mut y = 0;
        
        for c in text.chars() {
            if x + 6 >= WIDTH {
                x = 0;
                y += 8;
                if y + 8 >= HEIGHT {
                    break;
                }
            }
            
            self.draw_char(x, y, c);
            x += 6;
        }
        
        self.draw_cursor();
        self.display.update(&self.buffer).unwrap();
    }
    
    fn draw_char(&mut self, x: usize, y: usize, c: char) {
        // Simple 6x8 font for demo
        let pattern = match c {
            'A' => [0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
            'B' => [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00],
            'C' => [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00],
            ' ' => [0x00; 8],
            _ => [0x00; 8],
        };
        
        for dy in 0..8 {
            let row = pattern[dy];
            for dx in 0..6 {
                if (row >> (5 - dx)) & 1 == 1 {
                    let px = x + dx;
                    let py = y + dy;
                    if px < WIDTH && py < HEIGHT {
                        self.buffer.set_pixel(px, py as u8, Pixel::Black);
                    }
                }
            }
        }
    }
    
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::{stdin, stdout, Read};
        
        println!("Simple Text Editor");
        println!("Type text. Press Ctrl+C to exit.");
        
        let mut input = String::new();
        
        loop {
            self.draw_text(&input);
            
            let mut buffer = [0; 1];
            stdin().read_exact(&mut buffer)?;
            let c = buffer[0] as char;
            
            match c {
                '\n' => {
                    input.push('\n');
                    self.cursor_y += 1;
                    self.cursor_x = 0;
                }
                '\x7f' => { // Backspace
                    if !input.is_empty() {
                        input.pop();
                        if self.cursor_x > 0 {
                            self.cursor_x -= 1;
                        }
                    }
                }
                '\x03' => break, // Ctrl+C
                _ => {
                    if c.is_ascii_graphic() || c == ' ' {
                        input.push(c);
                        self.cursor_x += 1;
                    }
                }
            }
        }
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = TextEditor::new()?;
    editor.run()
}