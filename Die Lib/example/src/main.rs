use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Bus, SlaveSelect};

const WIDTH: usize = 400;
const HEIGHT: u8 = 240;
const CS_GPIO: u8 = 12;

fn main() {
    let mut display = MemoryDisplay::new(
        Bus::Spi0,
        SlaveSelect::Ss0,
        CS_GPIO,
        WIDTH,
        HEIGHT,
    ).unwrap();

    let mut buffer = MemoryDisplayBuffer::new(WIDTH, HEIGHT);

    display.clear().unwrap();

    // Fill with vertical lines left to right. This requires an update
    // to every row on the display.
    println!("Vertical fill");
    buffer.fill(Pixel::White);
    for x in 0..400 {
        for y in 0..240 {
            buffer.set_pixel(x, y, Pixel::Black);
        }
        // update display
        display.update(&buffer).unwrap();
    }

    // Fill with horitzonal lines top to bottom. This only updates one
    // row at a time and should be much faster.
    println!("Horizontal fill");
    buffer.fill(Pixel::White);
    for y in 0..240 {
        for x in 0..400 {
            buffer.set_pixel(x, y, Pixel::Black);
        }
        // update display
        display.update(&buffer).unwrap();
    }
}
