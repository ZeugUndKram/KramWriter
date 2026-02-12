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
        if data.len() < 54 || &data[0..2] != b"BM" { return None; }

        let pixel_offset = u32::from_le_bytes(data[10..14].try_into().ok()?) as usize;
        let width = i32::from_le_bytes(data[18..22].try_into().ok()?) as usize;
        let height = i32::from_le_bytes(data[22..26].try_into().ok()?) as usize;
        let bpp = u16::from_le_bytes(data[28..30].try_into().ok()?);

        let mut pixels = Vec::with_capacity(width * height);

        match bpp {
            1 => {
                let row_padded = (width + 31) / 32 * 4;
                for y in (0..height).rev() {
                    let row_start = pixel_offset + y * row_padded;
                    for x in 0..width {
                        let byte = data[row_start + (x / 8)];
                        let bit = 7 - (x % 8);
                        // MATCH OLD VERSION: 1 is Black, 0 is White
                        pixels.push(if (byte >> bit) & 1 == 1 { Pixel::Black } else { Pixel::White });
                    }
                }
            }
            24 | 32 => {
                let bytes_per_pixel = (bpp / 8) as usize;
                let row_padded = (width * bytes_per_pixel + 3) & !3;
                for y in (0..height).rev() {
                    let row_start = pixel_offset + y * row_padded;
                    for x in 0..width {
                        let px_start = row_start + (x * bytes_per_pixel);
                        let b = data[px_start] as u32;
                        let g = data[px_start + 1] as u32;
                        let r = data[px_start + 2] as u32;
                        // MATCH OLD VERSION: Bright pixels are White, Dark are Black
                        let avg = (r + g + b) / 3;
                        pixels.push(if avg > 127 { Pixel::White } else { Pixel::Black });
                    }
                }
            }
            _ => return None,
        }
        Some(Self { width, height, pixels })
    }
}