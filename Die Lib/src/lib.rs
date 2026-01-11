//! Library to drive the Sharp Memory Display from a Raspberry Pi.

use rppal::gpio::{self, Gpio, OutputPin};
use rppal::spi::{self, Spi, Bus, SlaveSelect};
use std::fmt;

// In LSB format
const WRITECMD: u8 = 0x80;
const VCOM: u8 = 0x40;
const CLEAR: u8 = 0x20;
const PADDING: u8 = 0;

#[derive(Debug)]
pub enum Error {
    Spi(spi::Error),
    Gpio(gpio::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Spi(ref err) => Some(err),
            Error::Gpio(ref err) => Some(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            Error::Spi(ref err) => format!("{}", err),
            Error::Gpio(ref err) => format!("{}", err),
        };
        write!(f, "Memory Display error: {}", reason)
    }
}

impl From<spi::Error> for Error {
    fn from(err: spi::Error) -> Self {
        Error::Spi(err)
    }
}

impl From<gpio::Error> for Error {
    fn from(err: gpio::Error) -> Self {
        Error::Gpio(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Sharp Memory Display
#[derive(Debug)]
pub struct MemoryDisplay {
    /// Width of the display in pixels,
    width: usize,
    /// Height of the screen in pixels. Height is u8 because the y
    /// axis is used for the line address byte (meaning a screen
    /// cannot exceed 255 pixels high but can be wider).
    height: u8,
    vcom: u8,
    spi: Spi,
    /// Alternate SlaveSelect pin. SlaveSelect doesn't seem to respect
    /// polarity changes (not sure if rppal lib or hardware) and the
    /// MemoryDisplay uses the non-default ActiveHigh. Instead of
    /// using the real Spi SlaveSelect pin we leave it disconnected
    /// and manually set a separate GPIO pin high/low when writing
    /// data. Memory Display libraries for other languages seem to do
    /// the same.
    cs: OutputPin,
    /// The previous pixel buffer. Used to diff against new pixel
    /// values so only updated lines are written to the display.
    prev_pixels: Vec<u8>,
    /// All SPI data is buffered in memory first so it can be sent
    /// efficiently in batches matching the Raspberry Pi spidev.bufsiz
    /// option in boot/cmdline.txt (default settings is 4096).
    cmd_buffer: Vec<u8>,
    /// The value of spidev.bufsiz.
    spidev_bufsiz: usize,
}

impl MemoryDisplay {
    /// Initializes a new MemoryDisplay. The `spi_ss` pin should be
    /// left disconnected (but will still get toggled by the
    /// hardware). It is replaced by the `cs` GPIO pin.
    pub fn new(
        spi_bus: Bus,
        spi_ss: SlaveSelect,
        cs_pin: u8,
        width: usize,
        height: u8
    ) -> Result<Self> {
        // Required number of bytes to represent all pixels in a row.
        let row_size = width / 8;
        // Required number of bytes for each line update: line address
        // byte, row pixels, trailing padding byte.
        let line_size = 1 + row_size + 1;
        // Required number of bytes required for a full screen
        // refresh: leading command byte, line updates for
        // every row, trailing padding byte.
        let cmd_buffer_size: usize = 1 + (line_size * height as usize) + 1;
        let cmd_buffer = Vec::with_capacity(cmd_buffer_size);

        let cs = Gpio::new()?.get(cs_pin)?.into_output();
        let spi = Spi::new(
            spi_bus,
            spi_ss,
            2000000,
            spi::Mode::Mode0,
        )?;

        Ok(MemoryDisplay {
            width,
            height,
            vcom: VCOM,
            spi,
            cs,
            cmd_buffer,
            prev_pixels: vec![],
            spidev_bufsiz: 4096 // (default value on rpi is 4096)
        })
    }

    /// Sets the SPI clock frequency in Hz.
    ///
    /// Default value is 2Mhz (2000000). The spec for the Memory
    /// Display states a typical clock speed of 1Mhz, and a maximum
    /// speed of 2Mhz. The screens often do respond at a higher speed
    /// (e.g. 8MHz) but that will raise the internal temperatures and
    /// may lower the functional life of your screen.
    pub fn set_clock_speed(&mut self, clock_hz: u32) -> Result<()> {
        Ok(self.spi.set_clock_speed(clock_hz)?)
    }

    /// Gets the SPI clock frequency in Hz.
    pub fn clock_speed(&self) -> Result<u32> {
        Ok(self.spi.clock_speed()?)
    }

    /// Sets the SPI device buffer size (as configured by the
    /// spidev.bufsiz option in boot/cmdline.txt on the Raspberry Pi).
    ///
    /// You can read the current spidev.bufsiz by doing:
    /// cat /sys/module/spidev/parameters/bufsiz
    ///
    /// The default value for the Pi (and this library) is 4096 bytes.
    pub fn set_spidev_bufsiz(&mut self, spidev_bufsiz: usize) {
        self.spidev_bufsiz = spidev_bufsiz;
    }

    /// Gets the currently configured spidev.bufsiz for this library
    /// only. To get the real configured value for the hardware read
    /// /sys/module/spidev/parameters/bufsiz.
    pub fn spidev_bufsiz(&self) -> usize {
        self.spidev_bufsiz
    }

    /// Clears the screen.
    pub fn clear(&mut self) -> Result<()> {
        self.cmd_buffer.push(self.vcom | CLEAR);
        self.cmd_buffer.push(PADDING);
        self.prev_pixels = vec![0b11111111; self.width / 8 * self.height as usize];
        self.write_cmd()
    }

    /// Updates the display. Only lines that have changed since the
    /// last clear/update/refresh are sent to the display.
    ///
    /// The `pixels` buffer can be any [u8] slice where each bit
    /// represents a single black/white pixel (8 pixels per u8).
    /// MemoryDisplayBuffer provides a convenient interface to such a
    /// buffer.
    pub fn update<T: AsRef<[u8]>>(&mut self, pixels: T) -> Result<()> {
        let pixels = pixels.as_ref();
        let compare = self.prev_pixels.len() == pixels.len();
        self.cmd_buffer.push(self.vcom | WRITECMD);
        let mut offset = 0;
        for y in 0..self.height {
            let end = offset + self.width / 8;
            let row = &pixels[offset..end];
            if !compare || row != &self.prev_pixels[offset..end] {
                let line_addr = reverse_bits_in_byte(y + 1);
                self.cmd_buffer.push(line_addr);
                self.cmd_buffer.extend_from_slice(&row);
                self.cmd_buffer.push(PADDING);
            }
            offset += self.width / 8;
        }
        self.cmd_buffer.push(PADDING);
        self.prev_pixels.copy_from_slice(pixels);
        self.write_cmd()
    }

    fn write_cmd(&mut self) -> Result<()> {
        // Manually set chip select because the Memory Display uses
        // the less common ActiveHigh polarity and I can't get the
        // hardware SPI device to support it. Other Memory Display
        // libraries do the same thing.
        self.cs.set_high();
        // Write command in chunks matching the spidev_bufsiz for
        // optimal transfer speed.
        for chunk in self.cmd_buffer.chunks(self.spidev_bufsiz) {
            self.spi.write(chunk)?;
        }
        self.cs.set_low();
        // toggle vcom after every command
        self.vcom ^= VCOM;
        // clear command buffer ready for next command
        self.cmd_buffer.clear();
        Ok(())
    }
}

fn reverse_bits_in_byte(byte: u8) -> u8 {
    let mut tmp: [u8; 1] = [byte];
    spi::reverse_bits(&mut tmp);
    tmp[0]
}

/// A display buffer suitable for use with MemoryDisplay::update().
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDisplayBuffer {
    width: usize,
    height: u8,
    buffer: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pixel {
    Black,
    White,
}

impl MemoryDisplayBuffer {
    /// Creates a new buffer.
    pub fn new(width: usize, height: u8) -> Self {
        MemoryDisplayBuffer {
            width,
            height,
            // 1 is white, 0 is black
            buffer: vec![0b11111111; width / 8 * height as usize],
        }
    }

    /// Set all pixels in buffer to `value`.
    pub fn fill(&mut self, value: Pixel) {
        let b = match value {
            Pixel::White => 0b11111111,
            Pixel::Black => 0b00000000,
        };
        // TODO: use self.buffer.fill(b) once fill() API is stablised.
        // https://github.com/rust-lang/rust/issues/70758
        self.buffer = vec![b; self.width / 8 * self.height as usize];
    }

    /// Sets the pixel at the given co-ordinates. Set `value` true for
    /// white, false for black.
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is out of the bounds `(width, height)`.
    pub fn set_pixel(&mut self, x: usize, y: u8, value: Pixel) {
        assert!(x < self.width);
        assert!(y < self.height);
        let bytes_per_row = self.width / 8;
        let index = (y as usize * bytes_per_row) + (x / 8);
        let byte = &mut self.buffer[index];
        let bit_offset = x % 8;
        match value {
            Pixel::White => *byte |= match bit_offset {
                0 => 0b10000000,
                1 => 0b01000000,
                2 => 0b00100000,
                3 => 0b00010000,
                4 => 0b00001000,
                5 => 0b00000100,
                6 => 0b00000010,
                7 => 0b00000001,
                _ => unreachable!(),
            },
            Pixel::Black => *byte &= match bit_offset {
                0 => !0b10000000,
                1 => !0b01000000,
                2 => !0b00100000,
                3 => !0b00010000,
                4 => !0b00001000,
                5 => !0b00000100,
                6 => !0b00000010,
                7 => !0b00000001,
                _ => unreachable!(),
            },
        }
    }

    /// Returns value of pixel at given co-ordinates. Returns true for
    /// white, false for black.
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is out of the bounds `(width, height)`.
    pub fn get_pixel(&self, x: usize, y: u8) -> Pixel {
        assert!(x < self.width);
        assert!(y < self.height);
        let bytes_per_row = self.width / 8;
        let index = (y as usize * bytes_per_row) + (x / 8);
        let byte = &self.buffer[index];
        let bit_offset = x % 8;
        let mask = match bit_offset {
            0 => 0b10000000,
            1 => 0b01000000,
            2 => 0b00100000,
            3 => 0b00010000,
            4 => 0b00001000,
            5 => 0b00000100,
            6 => 0b00000010,
            7 => 0b00000001,
            _ => unreachable!(),
        };
        if *byte & mask == mask {
            Pixel::White
        } else {
            Pixel::Black
        }
    }
}

impl AsRef<[u8]> for MemoryDisplayBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.buffer[..]
    }
}
