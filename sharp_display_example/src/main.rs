// src/main.rs
use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use rppal::gpio::Gpio;
use std::{thread, time};

fn main() {
    match run() {
        Ok(_) => println!("Program completed successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing Sharp Memory Display...");
    
    let gpio = Gpio::new()?;
    println!("GPIO initialized");
    
    let spi = Spi::new(
        Bus::Spi0,
        SlaveSelect::Ss0,
        2_000_000,
        Mode::Mode0,
    )?;
    println!("SPI initialized");
    
    let cs_pin = gpio.get(6)?.into_output();
    println!("CS pin configured");
    
    let mut display = MemoryDisplay::new(spi, cs_pin, 400, 240)?;
    println!("Display created");
    
    display.clear()?;
    println!("Display cleared");
    
    let mut buffer = MemoryDisplayBuffer::new(400, 240);
    buffer.fill(Pixel::White);
    
    // Simple horizontal line
    for x in 0..400 {
        buffer.set_pixel(x, 120, Pixel::Black);
    }
    
    display.update(&buffer)?;
    println!("Pattern displayed");
    
    thread::sleep(time::Duration::from_secs(5));
    
    display.clear()?;
    println!("Display cleared");
    
    Ok(())
}