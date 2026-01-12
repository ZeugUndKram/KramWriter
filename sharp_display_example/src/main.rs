mod font_renderer;
mod display;

use crate::display::Display;
use anyhow::Result;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("=== BebasNeue Font Test ===");
    
    // 1. Initialize display
    println!("1. Initializing display...");
    let mut display = Display::new(400, 240)?;
    
    // 2. Load BebasNeue font
    println!("2. Loading font...");
    let font_path = "/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf";
    display.load_font(font_path, 36.0)?; // Try 36px size
    
    // 3. Clear display
    println!("3. Clearing display...");
    display.clear()?;
    thread::sleep(Duration::from_millis(500));
    
    // 4. Draw border to verify display works
    println!("4. Drawing border...");
    let buffer = display.buffer_mut();
    for x in 0..400 {
        buffer.set_pixel(x, 0, Pixel::Black);
        buffer.set_pixel(x, 239, Pixel::Black);
    }
    for y in 0..240 {
        buffer.set_pixel(0, y, Pixel::Black);
        buffer.set_pixel(399, y, Pixel::Black);
    }
    display.update()?;
    thread::sleep(Duration::from_secs(1));
    
    // 5. Clear inside border and draw text
    println!("5. Drawing text...");
    
    // Clear inside area
    for x in 1..399 {
        for y in 1..238 {
            buffer.set_pixel(x, y, Pixel::White);
        }
    }
    
    // Draw text
    display.draw_text(20, 50, "BEBAS NEUE")?;
    display.draw_text(20, 100, "ABCDEFGHIJ")?;
    display.draw_text(20, 150, "KLMNOPQRST")?;
    display.draw_text(20, 200, "UVWXYZ 123")?;
    
    display.update()?;
    
    println!("6. Text should be visible!");
    println!("7. Waiting 10 seconds...");
    thread::sleep(Duration::from_secs(10));
    
    println!("8. Clearing display...");
    display.clear()?;
    
    println!("Test complete!");
    Ok(())
}

// Need to import Pixel at module level
use rpi_memory_display::Pixel;