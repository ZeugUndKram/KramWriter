// src/main.rs
mod font_renderer;
mod display;

use anyhow::Result;
use crate::display::Display;

fn main() -> Result<()> {
    println!("=== Sharp Display Font Test ===");
    
    // Initialize display (400x240 typical for Sharp Memory Display)
    let mut display = Display::new(400, 240)?;
    
    // Load your BebasNeue font
    let font_path = "/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf";
    
    println!("\n1. Loading font...");
    display.load_font(font_path, 24.0)?;
    
    println!("\n2. Clearing display...");
    display.clear()?;
    
    println!("\n3. Drawing test patterns...");
    
    // Draw border around screen
    let (width, height) = display.dimensions();
    println!("Screen size: {}x{}", width, height);
    
    // Draw border
    for x in 0..width {
        display.buffer.set_pixel(x, 0, Pixel::Black);
        display.buffer.set_pixel(x, (height - 1) as u8, Pixel::Black);
    }
    for y in 0..height {
        display.buffer.set_pixel(0, y as u8, Pixel::Black);
        display.buffer.set_pixel(width - 1, y as u8, Pixel::Black);
    }
    
    // Draw simple test pattern first (without font)
    println!("Drawing test pattern...");
    for i in 0..20 {
        let x = 10 + i * 10;
        let y = 10 + i * 10;
        if x < width && y < height {
            display.buffer.set_pixel(x, y as u8, Pixel::Black);
        }
    }
    
    // Update to show we're alive
    display.update()?;
    println!("Test pattern drawn.");
    
    // Wait a bit
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    // Clear and try text
    println!("\n4. Testing text rendering...");
    display.clear()?;
    
    // Try drawing a simple 'A' first
    display.draw_char(50, 50, 'A')?;
    display.update()?;
    println!("Single character drawn.");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    // Clear and draw more text
    display.clear()?;
    
    // Draw text at different positions
    display.draw_text(20, 30, "HELLO")?;
    display.draw_text(20, 70, "WORLD")?;
    display.draw_text(20, 110, "TEST")?;
    display.draw_text(20, 150, "ABCD")?;
    
    display.update()?;
    println!("Text drawn.");
    
    // Draw cursor
    display.draw_cursor(20, 190, 24)?;
    display.update()?;
    
    println!("\n5. Display updated!");
    println!("Text should be visible on the Sharp display.");
    println!("Press Ctrl+C to exit.");
    
    // Keep program running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}