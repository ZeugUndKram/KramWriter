// src/main.rs
use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Bus, SlaveSelect};
use std::io::{self, Read, Write};
use std::time::Duration;

const WIDTH: usize = 400;
const HEIGHT: usize = 240;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing display...");
    
    let mut display = MemoryDisplay::new(
        Bus::Spi0,
        SlaveSelect::Ss0,
        6,
        WIDTH,
        HEIGHT as u8,
    )?;
    
    println!("Display ready!");
    println!("Type text. Press Enter for new line, Ctrl+C to exit.");
    println!("Note: Characters appear as you type (no need for Enter).");
    
    // Clear display first
    display.clear()?;
    
    // Create buffer with correct size
    let mut buffer = MemoryDisplayBuffer::new(WIDTH, HEIGHT as u8);
    buffer.fill(Pixel::White);
    
    // Set terminal to non-blocking mode
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    
    let mut input = String::new();
    let mut cursor_x = 0;
    let mut cursor_y = 0;
    
    // Clear any pending input
    let mut dummy = [0; 1024];
    let _ = stdin.read(&mut dummy);
    
    loop {
        // Draw current state
        buffer.fill(Pixel::White);
        draw_text(&mut buffer, &input, cursor_x, cursor_y);
        display.update(&buffer)?;
        
        // Check for input without blocking
        let mut byte = [0];
        let result = stdin.read(&mut byte);
        
        match result {
            Ok(1) => {
                match byte[0] as char {
                    '\n' => {
                        input.push('\n');
                        cursor_y += 1;
                        cursor_x = 0;
                        println!("New line");
                    }
                    '\x7f' => { // Backspace
                        if !input.is_empty() {
                            input.pop();
                            cursor_x = cursor_x.saturating_sub(1);
                            println!("Backspace");
                        }
                    }
                    '\x03' => { // Ctrl+C
                        println!("Exiting...");
                        break;
                    }
                    c if c.is_ascii() && !c.is_control() => {
                        input.push(c);
                        cursor_x += 1;
                        println!("Typed: {}", c);
                    }
                    _ => {}
                }
            }
            _ => {
                // No input available, small sleep to prevent CPU spinning
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
    
    display.clear()?;
    println!("Goodbye!");
    
    Ok(())
}

fn draw_text(buffer: &mut MemoryDisplayBuffer, text: &str, cursor_x: usize, cursor_y: usize) {
    let mut x = 10;
    let mut y = 10;
    let char_width = 6;
    let char_height = 8;
    
    // Draw text
    for c in text.chars() {
        if c == '\n' {
            x = 10;
            y += char_height + 2;
            continue;
        }
        
        if x + char_width >= WIDTH {
            x = 10;
            y += char_height + 2;
        }
        
        if y + char_height >= HEIGHT {
            break;
        }
        
        draw_char(buffer, x, y, c);
        x += char_width + 1;
    }
    
    // Draw cursor
    let cursor_screen_x = 10 + cursor_x * (char_width + 1);
    let cursor_screen_y = 10 + cursor_y * (char_height + 2);
    
    for dy in 0..char_height {
        if cursor_screen_y + dy < HEIGHT {
            buffer.set_pixel(cursor_screen_x, (cursor_screen_y + dy) as u8, Pixel::Black);
            if cursor_screen_x + 1 < WIDTH {
                buffer.set_pixel(cursor_screen_x + 1, (cursor_screen_y + dy) as u8, Pixel::Black);
            }
        }
    }
}

fn draw_char(buffer: &mut MemoryDisplayBuffer, x: usize, y: usize, c: char) {
    // Convert to uppercase for simplicity
    let c = c.to_ascii_uppercase();
    
    // Simple 5x7 font (smaller for visibility)
    let pattern = match c {
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
        ' ' => [0x00; 7],
        _ => [0x15, 0x0A, 0x15, 0x0A, 0x15, 0x0A, 0x15], // Pattern for unknown chars
    };
    
    for dy in 0..7 {
        let row = pattern[dy];
        for dx in 0..5 {
            if (row >> (4 - dx)) & 1 == 1 {
                let px = x + dx;
                let py = y + dy;
                if px < WIDTH && py < HEIGHT {
                    buffer.set_pixel(px, py as u8, Pixel::Black);
                }
            }
        }
    }
}