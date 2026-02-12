use rpi_memory_display::Pixel;
use anyhow::Result;
use std::fs;

pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Pixel>,
}

impl Bitmap {
    pub fn load(path: &str) -> Result<Self> {
        let data = fs::read(path)?;
        Self::parse(&data).ok_or_else(|| anyhow::anyhow!("Failed to parse BMP: {}", path))
    }

    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 54 || &data[0..2] != b"BM" {
            return None;
        }

        let file_size = u32::from_le_bytes(data[2..6].try_into().ok()?);
        let pixel_offset = u32::from_le_bytes(data[10..14].try_into().ok()?) as usize;
        let width = u32::from_le_bytes(data[18..22].try_into().ok()?) as usize;
        let height = u32::from_le_bytes(data[22..26].try_into().ok()?) as usize;
        let bits_per_pixel = u16::from_le_bytes(data[28..30].try_into().ok()?);

        if bits_per_pixel != 1 {
            println!("Unsupported BMP bpp: {}", bits_per_pixel);
            return None;
        }

        let row_padded = (width + 31) / 32 * 4;
        let mut pixels = Vec::with_capacity(width * height);

        for y in (0..height).rev() {
            let row_start = pixel_offset + y * row_padded;
            if row_start + row_padded > data.len() {
                continue;
            }

            for x in 0..width {
                let byte = data[row_start + (x / 8)];
                let bit = 7 - (x % 8);
                // 1 in BMP usually means White, 0 means Black.
                // Adjust this mapping if your images look inverted.
                let pixel = if (byte >> bit) & 1 == 1 { Pixel::White } else { Pixel::Black };
                pixels.push(pixel);
            }
        }

        Some(Self { width, height, pixels })
    }
}