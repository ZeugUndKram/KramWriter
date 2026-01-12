use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Bus, SlaveSelect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a display instance
    // Common configuration for 400x240 Sharp Memory Display:
    let mut display = MemoryDisplay::new(
        Bus::Spi0,          // SPI0 on Raspberry Pi
        SlaveSelect::Ss0,   // CE0 (pin 24)
        25,                 // GPIO pin for CS (chip select)
        400,                // width in pixels
        240,                // height in pixels
    )?;
    
    // Clear the display
    display.clear()?;
    
    // Create a display buffer
    let mut buffer = MemoryDisplayBuffer::new(400, 240);
    
    // Draw a white background
    buffer.fill(Pixel::White);
    
    // Draw some black pixels/lines
    for x in 100..300 {
        buffer.set_pixel(x, 100, Pixel::Black);
        buffer.set_pixel(x, 101, Pixel::Black);
    }
    
    // Draw a rectangle
    for y in 50..150 {
        for x in 50..150 {
            buffer.set_pixel(x, y, Pixel::Black);
        }
    }
    
    // Update the display with our buffer
    display.update(&buffer)?;
    
    Ok(())
}