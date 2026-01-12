// src/main.rs (simplified test)
mod font_renderer;
mod display;

use crate::display::Display;
use anyhow::Result;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("=== Final Font Test ===");
    
    // 1. Initialize display
    println!("1. Initializing display...");
    let mut display = Display::new(400, 240)?;
    display.clear()?;
    
    // 2. Load font
    println!("2. Loading font...");
    let font_path = "/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf";
    
    // Try multiple sizes
    let sizes = [48.0, 36.0, 24.0, 16.0];
    let mut loaded = false;
    
    for &size in &sizes {
        println!("  Trying size: {}px", size);
        if display.load_font(font_path, size).is_ok() {
            println!("  ✓ Font loaded at {}px", size);
            loaded = true;
            break;
        }
    }
    
    if !loaded {
        println!("✗ Could not load font at any size");
        println!("Drawing test pattern instead...");
        display.draw_border()?;
        
        // Draw a smiley
        for &(x, y) in &[(200, 120), (190, 110), (210, 110), (185, 130), (215, 130)] {
            for dx in -2..=2 {
                for dy in -2..=2 {
                    display.draw_pixel((x as isize + dx) as usize, (y as isize + dy) as usize)?;
                }
            }
        }
        
        display.update()?;
        thread::sleep(Duration::from_secs(5));
        display.clear()?;
        return Ok(());
    }
    
    // 3. Test rendering
    println!("3. Testing rendering...");
    display.clear()?;
    
    // Draw text at different positions
    let texts = [
        (20, 40, "BEBAS"),
        (20, 100, "NEUE"),
        (20, 160, "12345"),
        (20, 220, "HELLO"),
    ];
    
    for &(x, y, text) in &texts {
        if y < 240 {
            println!("  Drawing '{}' at ({}, {})", text, x, y);
            display.draw_text(x, y, text)?;
        }
    }
    
    display.update()?;
    
    println!("4. Text should be visible!");
    println!("5. Waiting 10 seconds...");
    thread::sleep(Duration::from_secs(10));
    
    println!("6. Clearing display...");
    display.clear()?;
    
    println!("Test complete!");
    Ok(())
}