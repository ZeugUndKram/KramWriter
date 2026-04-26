use rpi_memory_display::Pixel;
use std::fs;

pub struct BlockSprites {
    pub i_sprite: Option<Vec<Pixel>>,
    pub o_sprite: Option<Vec<Pixel>>,
    pub s_sprite: Option<Vec<Pixel>>,
    pub z_sprite: Option<Vec<Pixel>>,
    pub t_sprite: Option<Vec<Pixel>>,
    pub l_sprite: Option<Vec<Pixel>>,
    pub j_sprite: Option<Vec<Pixel>>,
    pub sprite_width: usize,
    pub sprite_height: usize,
}

impl BlockSprites {
    pub fn new() -> Result<Self, String> {
        let sprite_width = 12;
        let sprite_height = 12;
        
        let base_path = "/home/kramwriter/KramWriter/assets/zeugtris/";
        
        println!("Loading Tetris sprites from: {}", base_path);
        
        let i_sprite = Self::load_sprite(&format!("{}zeugtris_i.bmp", base_path), sprite_width, sprite_height);
        let o_sprite = Self::load_sprite(&format!("{}zeugtris_o.bmp", base_path), sprite_width, sprite_height);
        let s_sprite = Self::load_sprite(&format!("{}zeugtris_s.bmp", base_path), sprite_width, sprite_height);
        let z_sprite = Self::load_sprite(&format!("{}zeugtris_z.bmp", base_path), sprite_width, sprite_height);
        let t_sprite = Self::load_sprite(&format!("{}zeugtris_t.bmp", base_path), sprite_width, sprite_height);
        let l_sprite = Self::load_sprite(&format!("{}zeugtris_l.bmp", base_path), sprite_width, sprite_height);
        let j_sprite = Self::load_sprite(&format!("{}zeugtris_j.bmp", base_path), sprite_width, sprite_height);
        
        // Check if any sprites loaded successfully
        let sprites_loaded = [&i_sprite, &o_sprite, &s_sprite, &z_sprite, &t_sprite, &l_sprite, &j_sprite]
            .iter()
            .any(|sprite| sprite.is_some());
        
        if !sprites_loaded {
            println!("Warning: No Tetris sprites loaded successfully");
        }
        
        Ok(Self {
            i_sprite,
            o_sprite,
            s_sprite,
            z_sprite,
            t_sprite,
            l_sprite,
            j_sprite,
            sprite_width,
            sprite_height,
        })
    }
    
    fn load_sprite(path: &str, expected_width: usize, expected_height: usize) -> Option<Vec<Pixel>> {
        match fs::read(path) {
            Ok(data) => {
                println!("Loaded sprite: {} ({} bytes)", path, data.len());
                match Self::parse_bmp(&data, expected_width, expected_height) {
                    Some(pixels) => {
                        println!("Successfully parsed sprite: {}", path);
                        Some(pixels)
                    }
                    None => {
                        println!("Failed to parse sprite BMP: {}", path);
                        None
                    }
                }
            }
            Err(e) => {
                println!("Failed to read sprite {}: {}", path, e);
                None
            }
        }
    }
    
    fn parse_bmp(data: &[u8], expected_width: usize, expected_height: usize) -> Option<Vec<Pixel>> {
        if data.len() < 54 { return None; }
        if data[0] != 0x42 || data[1] != 0x4D { return None; }
        
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]) as usize;
        let data_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        
        // Verify dimensions match expected 12x12
        if width != expected_width || height != expected_height {
            println!("Sprite dimensions mismatch: expected {}x{}, got {}x{}", 
                     expected_width, expected_height, width, height);
            return None;
        }
        
        if data_offset >= data.len() { return None; }
        
        let mut pixels = Vec::with_capacity(width * height);
        
        match bits_per_pixel {
            32 => {
                let row_bytes = width * 4;
                for y in 0..height {
                    let row_start = data_offset + (height - 1 - y) * row_bytes;
                    for x in 0..width {
                        let pixel_start = row_start + x * 4;
                        if pixel_start + 3 >= data.len() {
                            pixels.push(Pixel::White);
                            continue;
                        }
                        let b = data[pixel_start] as u32;
                        let g = data[pixel_start + 1] as u32;
                        let r = data[pixel_start + 2] as u32;
                        let a = data[pixel_start + 3] as u32;
                        
                        let luminance = (r * 299 + g * 587 + b * 114) / 1000;
                        let alpha = a;
                        
                        let pixel = if alpha < 128 {
                            Pixel::White
                        } else if luminance > 128 {
                            Pixel::White
                        } else {
                            Pixel::Black
                        };
                        pixels.push(pixel);
                    }
                }
            }
            24 => {
                let row_bytes = ((width * 3 + 3) / 4) * 4;
                for y in 0..height {
                    let row_start = data_offset + (height - 1 - y) * row_bytes;
                    for x in 0..width {
                        let pixel_start = row_start + x * 3;
                        if pixel_start + 2 >= data.len() {
                            pixels.push(Pixel::White);
                            continue;
                        }
                        let b = data[pixel_start] as u32;
                        let g = data[pixel_start + 1] as u32;
                        let r = data[pixel_start + 2] as u32;
                        
                        let luminance = (r * 299 + g * 587 + b * 114) / 1000;
                        let pixel = if luminance > 128 { Pixel::White } else { Pixel::Black };
                        pixels.push(pixel);
                    }
                }
            }
            1 => {
                let row_bytes = ((width + 31) / 32) * 4;
                for y in 0..height {
                    let row_start = data_offset + (height - 1 - y) * row_bytes;
                    for x in 0..width {
                        if row_start + (x / 8) >= data.len() {
                            pixels.push(Pixel::White);
                            continue;
                        }
                        let byte = data[row_start + (x / 8)];
                        let bit = 7 - (x % 8);
                        let pixel = if (byte >> bit) & 1 == 1 { Pixel::Black } else { Pixel::White };
                        pixels.push(pixel);
                    }
                }
            }
            _ => {
                println!("Unsupported BMP format: {} bpp", bits_per_pixel);
                return None;
            }
        }
        
        Some(pixels)
    }
    
    pub fn get_sprite(&self, piece_type: usize) -> Option<&Vec<Pixel>> {
        match piece_type {
            0 => self.i_sprite.as_ref(),
            1 => self.o_sprite.as_ref(),
            2 => self.s_sprite.as_ref(),
            3 => self.z_sprite.as_ref(),
            4 => self.t_sprite.as_ref(),
            5 => self.l_sprite.as_ref(),
            6 => self.j_sprite.as_ref(),
            _ => None,
        }
    }
    
    pub fn has_sprites(&self) -> bool {
        self.i_sprite.is_some() || 
        self.o_sprite.is_some() || 
        self.s_sprite.is_some() || 
        self.z_sprite.is_some() || 
        self.t_sprite.is_some() || 
        self.l_sprite.is_some() || 
        self.j_sprite.is_some()
    }
}