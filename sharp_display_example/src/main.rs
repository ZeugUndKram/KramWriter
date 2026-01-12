use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Bus, SlaveSelect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing...");
    
    // Use the correct constructor signature from the docs
    let mut display = MemoryDisplay::new(
        Bus::Spi0,
        SlaveSelect::Ss0,
        6,  // CS pin (GPIO 25)
        400,
        240,
    )?;
    
    println!("Display created!");
    display.clear()?;
    println!("Cleared");
    
    let mut buffer = MemoryDisplayBuffer::new(400, 240);
    buffer.fill(Pixel::White);
    
    // Update the drawing part:
let mut buffer = MemoryDisplayBuffer::new(400, 240);
buffer.fill(Pixel::White);

// Draw checkerboard pattern
for y in 0..240 {
    for x in 0..400 {
        if (x / 20 + y / 20) % 2 == 0 {
            buffer.set_pixel(x, y as u8, Pixel::Black);
        }
    }
}
    display.update(&buffer)?;
    println!("Display updated!");
    
    Ok(())
}