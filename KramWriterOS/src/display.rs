use rpi_memory_display::{MemoryDisplay, MemoryDisplayBuffer, Pixel};
use rppal::spi::{Bus, SlaveSelect};
use anyhow::Result;
use crate::context::Context;

const WIDTH: usize = 400;
const HEIGHT: usize = 240;

pub struct SharpDisplay {
    inner: MemoryDisplay,
    buffer: MemoryDisplayBuffer,
}

impl SharpDisplay {
    pub fn new(cs_pin: u8) -> Result<Self> {
        let mut inner = MemoryDisplay::new(
            Bus::Spi0,
            SlaveSelect::Ss0,
            cs_pin,
            WIDTH,
            HEIGHT as u8,
        )?;
        
        inner.clear()?;
        let buffer = MemoryDisplayBuffer::new(WIDTH, HEIGHT as u8);
        
        Ok(Self { inner, buffer })
    }

    pub fn clear(&mut self) {
        // We always fill with White (the "Off" state)
        // Dark mode inversion happens at the draw_pixel level
        self.buffer.fill(Pixel::White);
    }

    pub fn update(&mut self) -> Result<()> {
        self.inner.update(&self.buffer)?;
        Ok(())
    }

    /// The "Smart" draw pixel that looks at global context for Dark Mode
    pub fn draw_pixel(&mut self, x: usize, y: usize, mut pixel: Pixel, ctx: &Context) {
        if x < WIDTH && y < HEIGHT {
            // If dark mode is on, we invert the logic:
            // What was supposed to be Black (ink) becomes White (paper)
            if ctx.dark_mode {
                pixel = match pixel {
                    Pixel::Black => Pixel::White,
                    Pixel::White => Pixel::Black,
                };
            }
            self.buffer.set_pixel(x, y as u8, pixel);
        }
    }

    // Adapt your text/char drawing to pass the context down
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, ctx: &Context) {
        for (i, c) in text.chars().enumerate() {
            if x + i * 6 < WIDTH {
                self.draw_char(x + i * 6, y, c, ctx);
            }
        }
    }

    fn draw_char(&mut self, x: usize, y: usize, _c: char, ctx: &Context) {
    // Temporary: Draw a 4x4 square for each character
    for i in 0..4 {
        for j in 0..4 {
            self.draw_pixel(x + i, y + j, rpi_memory_display::Pixel::Black, ctx);
            }
        }
    }
}