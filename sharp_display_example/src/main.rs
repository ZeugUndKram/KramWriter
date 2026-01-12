mod font_renderer;
mod display;

use crate::display::Display;
use anyhow::Result;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("=== Font Debug Test ===");
    
    // 1. Initialize display
    println!("1. Initializing display...");
    let mut display = Display::new(400, 240)?;
    
    // 2. Clear and show we're working
    display.clear()?;
    display.draw_border()?;
    display.update()?;
    println!("Border drawn - display works!");
    thread::sleep(Duration::from_secs(1));
    
    // 3. Try to load font
    println!("2. Loading BebasNeue font...");
    let font_path = "/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf";
    
    match display.load_font(font_path, 32.0) {
        Ok(_) => println!("✓ Font loaded successfully"),
        Err(e) => {
            eprintln!("✗ Failed to load font: {}", e);
            eprintln!("Trying smaller font size...");
            display.load_font(font_path, 16.0)?;
        }
    }
    
    // 4. Clear inside border
    println!("3. Clearing inside area...");
    display.clear()?;
    display.draw_border()?;
    
    // 5. Test drawing a single character with debug
    println!("4. Testing single character 'A'...");
    
    // Draw at a specific position
    let test_x = 50;
    let test_y = 100;
    
    // First, mark the position with a red dot (we'll use a box)
    for dx in 0..5 {
        for dy in 0..5 {
            display.draw_pixel(test_x + dx, test_y + dy)?;
        }
    }
    
    display.update()?;
    println!("Red dot at ({}, {}) should be visible", test_x, test_y);
    thread::sleep(Duration::from_secs(1));
    
    // Now try to draw 'A' at that position
    println!("5. Drawing 'A' character...");
    display.draw_char(test_x, test_y, 'A')?;
    display.update()?;
    
    println!("6. 'A' should appear near the red dot");
    println!("7. Waiting 3 seconds...");
    thread::sleep(Duration::from_secs(3));
    
    // 6. Clear and try simple pattern
    println!("8. Testing fallback characters...");
    display.clear()?;
    
    // Draw a grid of fallback characters (should work even without font)
    for i in 0..5 {
        let x = 20 + i * 40;
        let y = 50;
        display.draw_fallback_char(x, y)?;
    }
    
    display.update()?;
    println!("9. Fallback boxes should be visible");
    thread::sleep(Duration::from_secs(2));
    
    // 7. Finally try text
    println!("10. Testing text rendering...");
    display.clear()?;
    
    // Try small font size
    display.load_font(font_path, 20.0)?;
    
    // Draw simple text
    display.draw_text(30, 50, "TEST")?;
    display.draw_text(30, 100, "ABCD")?;
    
    display.update()?;
    
    println!("11. Text should be visible!");
    println!("12. Waiting 5 seconds...");
    thread::sleep(Duration::from_secs(5));
    
    println!("13. Clearing display...");
    display.clear()?;
    
    println!("Debug complete!");
    Ok(())
}