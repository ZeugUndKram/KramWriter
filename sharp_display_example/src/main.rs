mod font_renderer;
mod display;

use crate::display::Display;
use anyhow::Result;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("=== Font Rendering Fix Test ===");
    
    // 1. Initialize display
    println!("1. Initializing display...");
    let mut display = Display::new(400, 240)?;
    
    // Clear and draw border
    display.clear()?;
    display.draw_border()?;
    display.update()?;
    println!("Border visible - display works!");
    thread::sleep(Duration::from_secs(1));
    
    // 2. Load font with smaller size (easier to debug)
    println!("2. Loading font...");
    let font_path = "/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf";
    
    // Try different sizes
    for size in &[48.0, 36.0, 24.0, 16.0] {
        println!("  Trying size: {}px", size);
        match display.load_font(font_path, *size) {
            Ok(_) => {
                println!("  ✓ Font loaded at {}px", size);
                break;
            }
            Err(e) => println!("  ✗ Failed at {}px: {}", size, e),
        }
    }
    
    // 3. Test individual characters
    println!("3. Testing individual characters...");
    display.clear()?;
    
    let test_chars = ['A', 'B', 'C', '1', '2', '3'];
let mut x = 30;
let mut y = 100;  // Add 'mut' here

for &ch in test_chars.iter() {  // Remove the index
    println!("  Drawing '{}' at ({}, {})", ch, x, y);
    
    // Draw position marker
    for dx in 0..3 {
        for dy in 0..3 {
            display.draw_pixel(x + dx, y + dy)?;
        }
    }
    
    // Draw the character
    display.draw_char(x, y, ch)?;
    
    x += 40;
    if x > 350 {
        x = 30;
        y += 50;
    }
}
    
    display.update()?;
    println!("4. Characters should be visible (with markers)");
    println!("5. Waiting 5 seconds...");
    thread::sleep(Duration::from_secs(5));
    
    // 4. Try text
    println!("6. Testing text rendering...");
    display.clear()?;
    
    // Try at different positions
    display.draw_text(20, 30, "HELLO")?;
    display.draw_text(20, 80, "BEBAS")?;
    display.draw_text(20, 130, "12345")?;
    
    display.update()?;
    
    println!("7. Text should be visible!");
    println!("8. Waiting 5 seconds...");
    thread::sleep(Duration::from_secs(5));
    
    println!("9. Clearing display...");
    display.clear()?;
    
    println!("Test complete!");
    Ok(())
}