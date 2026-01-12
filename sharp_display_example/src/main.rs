// src/main.rs
mod font_renderer;
mod display;

use crate::display::Display;
use anyhow::Result;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("=== ab_glyph Font Test ===");
    
    // Initialize display
    let mut display = Display::new(400, 240)?;
    display.clear()?;
    
    // Draw border to confirm display works
    display.draw_border()?;
    display.update()?;
    println!("Border drawn");
    thread::sleep(Duration::from_secs(1));
    
    // Try to load font
    let font_path = "/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf";
    
    println!("Loading font with ab_glyph...");
    if display.load_font(font_path, 32.0).is_err() {
        println!("Failed to load font, drawing fallback pattern");
        display.clear()?;
        
        // Draw a test pattern
        for i in 0..20 {
            let x = 50 + i * 15;
            let y = 100;
            display.draw_fallback_char(x, y)?;
        }
        
        display.update()?;
        thread::sleep(Duration::from_secs(5));
        display.clear()?;
        return Ok(());
    }
    
    // Clear and draw text
    display.clear()?;
    
    // Test with simple text
    let test_strings = [
        (30, 50, "HELLO"),
        (30, 100, "WORLD"),
        (30, 150, "TEST"),
    ];
    
    for &(x, y, text) in &test_strings {
        println!("Drawing: '{}'", text);
        display.draw_text(x, y, text)?;
    }
    
    display.update()?;
    
    println!("Text should be visible!");
    println!("Waiting 10 seconds...");
    thread::sleep(Duration::from_secs(10));
    
    display.clear()?;
    println!("Done!");
    
    Ok(())
}