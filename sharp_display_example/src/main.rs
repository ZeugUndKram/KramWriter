mod font_renderer;
mod display;

use crate::display::Display;
use anyhow::Result;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("=== Simple BebasNeue Test ===");
    
    // 1. Initialize display
    println!("Step 1: Initializing display...");
    let mut display = Display::new(400, 240)?;
    
    // Wait a moment
    thread::sleep(Duration::from_millis(500));
    
    // 2. Test with border first (no font)
    println!("Step 2: Drawing border (basic test)...");
    display.clear()?;
    display.draw_border()?;
    display.update()?;
    
    println!("Border should be visible now!");
    thread::sleep(Duration::from_secs(2));
    
    // 3. Try loading font
    println!("Step 3: Loading font...");
    let font_path = "/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf";
    
    match display.load_font(font_path, 32.0) {
        Ok(_) => println!("Font loaded successfully!"),
        Err(e) => {
            eprintln!("Failed to load font: {}", e);
            eprintln!("Continuing with fallback font...");
        }
    }
    
    // 4. Clear and draw text
    println!("Step 4: Drawing text...");
    display.clear()?;
    
    // Draw some text
    display.draw_text(20, 50, "HELLO")?;
    display.draw_text(20, 100, "WORLD")?;
    
    display.update()?;
    
    println!("Step 5: Text should be visible!");
    println!("Waiting 5 seconds...");
    thread::sleep(Duration::from_secs(5));
    
    println!("Step 6: Clearing display...");
    display.clear()?;
    
    println!("Test complete!");
    Ok(())
}