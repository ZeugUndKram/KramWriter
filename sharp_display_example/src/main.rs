// src/main.rs
mod font_renderer;
mod display;

use anyhow::Result;
use crate::display::Display;

fn main() -> Result<()> {
    println!("Sharp Display Font Test");
    println!("=======================");
    
    // Initialize display (400x240 typical for Sharp Memory Display)
    let mut display = Display::new(400, 240)?;
    
    // Load your BebasNeue font
    let font_path = "/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf";
    
    println!("Loading font: {}", font_path);
    match display.load_font(font_path, 24.0) {
        Ok(_) => println!("Font loaded successfully!"),
        Err(e) => {
            eprintln!("Failed to load font: {}", e);
            eprintln!("Using placeholder font instead");
        }
    }
    
    // Clear display
    display.clear()?;
    
    // Draw some text
    println!("Drawing test text...");
    display.draw_text(10, 10, "Sharp Display Editor")?;
    display.draw_text(10, 50, "ABCDEFGHIJKLM")?;
    display.draw_text(10, 90, "NOPQRSTUVWXYZ")?;
    display.draw_text(10, 130, "0123456789")?;
    display.draw_text(10, 170, "Hello, World!")?;
    
    // Draw a cursor
    display.draw_cursor(10, 210, 24)?;
    
    // Update the display
    display.update()?;
    
    println!("Display updated!");
    println!("Text should be visible on the Sharp display.");
    println!("Press Ctrl+C to exit.");
    
    // Keep program running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}