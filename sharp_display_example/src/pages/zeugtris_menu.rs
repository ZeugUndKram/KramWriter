use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::time::Instant;
use rand::Rng;

pub struct ZeugtrisMenuPage {
    background_pieces: Vec<BackgroundPiece>,
    last_update_time: Instant,
    frame_count: u32,
    logo_data: Option<Vec<Pixel>>,
    logo_width: usize,
    logo_height: usize,
}

struct BackgroundPiece {
    piece_type: usize,
    rotation: usize,
    x: f32,
    y: f32,
    speed: f32,
    rotation_speed: f32,
    drift: f32,
    drift_direction: f32,
    size: usize,
}

impl BackgroundPiece {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            piece_type: rng.gen_range(0..7),
            rotation: rng.gen_range(0..4),
            x: rng.gen_range(0.0..400.0),
            y: rng.gen_range(-100.0..0.0), // Start above screen
            speed: rng.gen_range(0.5..2.0),
            rotation_speed: rng.gen_range(-0.05..0.05),
            drift: rng.gen_range(0.0..0.3),
            drift_direction: if rng.gen_bool(0.5) { 1.0 } else { -1.0 },
            size: 8,
        }
    }
    
    fn update(&mut self, delta_time: f32) {
        // Update position based on delta time for smooth animation
        self.y += self.speed * delta_time * 60.0;
        self.rotation = (self.rotation as f32 + self.rotation_speed * delta_time * 60.0) as usize % 4;
        self.x += self.drift * self.drift_direction * delta_time * 60.0;
        
        // Wrap around screen edges
        if self.x < -50.0 {
            self.x = 450.0;
        } else if self.x > 450.0 {
            self.x = -50.0;
        }
    }
    
    fn is_off_screen(&self) -> bool {
        self.y > 300.0
    }
    
    fn reset(&mut self) {
        let mut rng = rand::thread_rng();
        self.piece_type = rng.gen_range(0..7);
        self.rotation = rng.gen_range(0..4);
        self.x = rng.gen_range(0.0..400.0);
        self.y = rng.gen_range(-100.0..-50.0);
        self.speed = rng.gen_range(0.5..2.0);
        self.rotation_speed = rng.gen_range(-0.05..0.05);
        self.drift = rng.gen_range(0.0..0.3);
        self.drift_direction = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
    }
    
    fn draw(&self, display: &mut SharpDisplay) {
        // Tetromino definitions (simplified for menu background)
        let piece_data = match self.piece_type {
            0 => [ // I piece
                [0,0,0,0, 1,1,1,1, 0,0,0,0, 0,0,0,0],
                [0,0,1,0, 0,0,1,0, 0,0,1,0, 0,0,1,0],
                [0,0,0,0, 0,0,0,0, 1,1,1,1, 0,0,0,0],
                [0,1,0,0, 0,1,0,0, 0,1,0,0, 0,1,0,0],
            ],
            1 => [ // O piece
                [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
                [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
                [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
                [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
            ],
            2 => [ // S piece
                [0,0,0,0, 0,0,1,1, 0,1,1,0, 0,0,0,0],
                [0,0,1,0, 0,0,1,1, 0,0,0,1, 0,0,0,0],
                [0,0,0,0, 0,1,1,0, 1,1,0,0, 0,0,0,0],
                [0,1,0,0, 0,1,1,0, 0,0,1,0, 0,0,0,0],
            ],
            3 => [ // Z piece
                [0,0,0,0, 1,1,0,0, 0,1,1,0, 0,0,0,0],
                [0,0,0,1, 0,0,1,1, 0,0,1,0, 0,0,0,0],
                [0,0,0,0, 0,1,1,0, 0,0,1,1, 0,0,0,0],
                [0,0,1,0, 0,1,1,0, 0,1,0,0, 0,0,0,0],
            ],
            4 => [ // T piece
                [0,0,0,0, 0,1,0,0, 1,1,1,0, 0,0,0,0],
                [0,0,1,0, 0,1,1,0, 0,0,1,0, 0,0,0,0],
                [0,0,0,0, 1,1,1,0, 0,1,0,0, 0,0,0,0],
                [0,0,1,0, 0,0,1,1, 0,0,1,0, 0,0,0,0],
            ],
            5 => [ // L piece
                [0,0,0,0, 0,0,0,1, 0,1,1,1, 0,0,0,0],
                [0,0,1,0, 0,0,1,0, 0,0,1,1, 0,0,0,0],
                [0,0,0,0, 0,1,1,1, 0,1,0,0, 0,0,0,0],
                [0,1,1,0, 0,0,1,0, 0,0,1,0, 0,0,0,0],
            ],
            6 => [ // J piece
                [0,0,0,0, 0,1,0,0, 0,1,1,1, 0,0,0,0],
                [0,0,1,1, 0,0,1,0, 0,0,1,0, 0,0,0,0],
                [0,0,0,0, 0,1,1,1, 0,0,0,1, 0,0,0,0],
                [0,0,1,0, 0,0,1,0, 0,1,1,0, 0,0,0,0],
            ],
            _ => [[0; 16]; 4],
        };
        
        let rotation_data = piece_data[self.rotation];
        
        for py in 0..4 {
            for px in 0..4 {
                let index = py * 4 + px;
                if rotation_data[index] == 0 {
                    continue;
                }
                
                let screen_x = self.x as i32 + (px as i32 * self.size as i32);
                let screen_y = self.y as i32 + (py as i32 * self.size as i32);
                
                // Draw the block
                for by in 0..self.size {
                    for bx in 0..self.size {
                        let draw_x = screen_x + bx as i32;
                        let draw_y = screen_y + by as i32;
                        
                        if draw_x >= 0 && draw_x < 400 && draw_y >= 0 && draw_y < 240 {
                            display.draw_pixel(draw_x as usize, draw_y as usize, Pixel::Black);
                        }
                    }
                }
            }
        }
    }
}

impl ZeugtrisMenuPage {
    pub fn new() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let mut background_pieces = Vec::new();
        
        // Create background pieces
        for _ in 0..rng.gen_range(12..19) {
            background_pieces.push(BackgroundPiece::new());
        }
        
        // Load the Zeugtris logo
        let logo_path = "/home/kramwriter/KramWriter/assets/zeugtris/zeugtris_logo.bmp";
        println!("Loading Zeugtris logo from: {}", logo_path);
        
        let (logo_data, logo_width, logo_height) = match std::fs::read(logo_path) {
            Ok(data) => {
                println!("Loaded logo: {} bytes", data.len());
                match Self::parse_bmp(&data) {
                    Some((pixels, width, height)) => {
                        println!("Parsed logo BMP: {}x{}, {} pixels", width, height, pixels.len());
                        (Some(pixels), width, height)
                    }
                    None => {
                        println!("Failed to parse logo BMP");
                        (None, 0, 0)
                    }
                }
            }
            Err(e) => {
                println!("Failed to read logo: {}", e);
                (None, 0, 0)
            }
        };
        
        Ok(Self {
            background_pieces,
            last_update_time: Instant::now(),
            frame_count: 0,
            logo_data,
            logo_width,
            logo_height,
        })
    }
    
    fn parse_bmp(data: &[u8]) -> Option<(Vec<Pixel>, usize, usize)> {
        if data.len() < 54 { return None; }
        if data[0] != 0x42 || data[1] != 0x4D { return None; }
        
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]) as usize;
        let data_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        
        println!("Logo BMP: {}x{}, {} bpp, offset: {}", width, height, bits_per_pixel, data_offset);
        
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
        
        Some((pixels, width, height))
    }
    
    fn update_background(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update_time).as_secs_f32();
        self.frame_count = self.frame_count.wrapping_add(1);
        
        // Always update background pieces regardless of redraw
        for piece in &mut self.background_pieces {
            piece.update(delta_time);
            
            // Reset pieces that have fallen off screen
            if piece.is_off_screen() {
                piece.reset();
            }
        }
        
        self.last_update_time = now;
    }
    
    fn draw_logo(&self, display: &mut SharpDisplay) {
        if let Some(logo_pixels) = &self.logo_data {
            // Calculate center position
            let start_x = (400usize.saturating_sub(self.logo_width)) / 2;
            let start_y = (240usize.saturating_sub(self.logo_height)) / 2;
            
            println!("Drawing logo at: ({}, {}), size: {}x{}", start_x, start_y, self.logo_width, self.logo_height);
            
            for y in 0..self.logo_height.min(240) {
                for x in 0..self.logo_width.min(400) {
                    let pixel = logo_pixels[y * self.logo_width + x];
                    // Only draw black pixels (skip white/transparent)
                    if pixel == Pixel::Black {
                        let screen_x = start_x + x;
                        let screen_y = start_y + y;
                        if screen_x < 400 && screen_y < 240 {
                            display.draw_pixel(screen_x, screen_y, pixel);
                        }
                    }
                }
            }
        } else {
            println!("No logo data to draw");
            // Draw placeholder text
            let text = "ZEUGTRIS";
            let text_width = text.len() * 8; // Approximate width
            let start_x = (400 - text_width) / 2;
            let start_y = 100;
            
            // Simple text drawing
            for (i, c) in text.chars().enumerate() {
                // Draw a simple box for each character
                let char_x = start_x + i * 10;
                for y in 0..10 {
                    for x in 0..8 {
                        if char_x + x < 400 && start_y + y < 240 {
                            display.draw_pixel(char_x + x, start_y + y, Pixel::Black);
                        }
                    }
                }
            }
        }
    }
}

impl Page for ZeugtrisMenuPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        // Always update background animation
        self.update_background();
        
        // Clear and redraw every time - for smooth animation
        display.clear()?;
        
        // Draw all background pieces
        for piece in &self.background_pieces {
            piece.draw(display);
        }
        
        // Draw the Zeugtris logo in the center
        self.draw_logo(display);
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
            Key::Char('\n') => Ok(Some(PageId::Zeugtris)),
            Key::Esc => Ok(Some(PageId::Menu)),
            _ => Ok(None),
        }
    }
}