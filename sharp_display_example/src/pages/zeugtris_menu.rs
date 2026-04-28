use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::time::Instant;
use rand::Rng;

pub struct ZeugtrisMenuPage {
    background_pieces: Vec<BackgroundPiece>,
    last_update_time: Instant,
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
            y: rng.gen_range(-100.0..240.0), // Initial spread
            speed: rng.gen_range(0.5..2.0),
            rotation_speed: rng.gen_range(-0.05..0.05),
            drift: rng.gen_range(0.0..0.3),
            drift_direction: if rng.gen_bool(0.5) { 1.0 } else { -1.0 },
            size: 8,
        }
    }
    
    fn update(&mut self, delta_time: f32) {
        self.y += self.speed * delta_time * 60.0;
        self.rotation = (self.rotation as f32 + self.rotation_speed * delta_time * 60.0) as usize % 4;
        self.x += self.drift * self.drift_direction * delta_time * 60.0;
        
        if self.x < -50.0 { self.x = 450.0; } 
        else if self.x > 450.0 { self.x = -50.0; }
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
    }
    
    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        let piece_data = match self.piece_type {
            0 => [[0,0,0,0, 1,1,1,1, 0,0,0,0, 0,0,0,0], [0,0,1,0, 0,0,1,0, 0,0,1,0, 0,0,1,0], [0,0,0,0, 0,0,0,0, 1,1,1,1, 0,0,0,0], [0,1,0,0, 0,1,0,0, 0,1,0,0, 0,1,0,0]],
            1 => [[0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0]; 4],
            2 => [[0,0,0,0, 0,0,1,1, 0,1,1,0, 0,0,0,0], [0,0,1,0, 0,0,1,1, 0,0,0,1, 0,0,0,0], [0,0,0,0, 0,1,1,0, 1,1,0,0, 0,0,0,0], [0,1,0,0, 0,1,1,0, 0,0,1,0, 0,0,0,0]],
            3 => [[0,0,0,0, 1,1,0,0, 0,1,1,0, 0,0,0,0], [0,0,0,1, 0,0,1,1, 0,0,1,0, 0,0,0,0], [0,0,0,0, 0,1,1,0, 0,0,1,1, 0,0,0,0], [0,0,1,0, 0,1,1,0, 0,1,0,0, 0,0,0,0]],
            4 => [[0,0,0,0, 0,1,0,0, 1,1,1,0, 0,0,0,0], [0,0,1,0, 0,1,1,0, 0,0,1,0, 0,0,0,0], [0,0,0,0, 1,1,1,0, 0,1,0,0, 0,0,0,0], [0,0,1,0, 0,0,1,1, 0,0,1,0, 0,0,0,0]],
            5 => [[0,0,0,0, 0,0,0,1, 0,1,1,1, 0,0,0,0], [0,0,1,0, 0,0,1,0, 0,0,1,1, 0,0,0,0], [0,0,0,0, 0,1,1,1, 0,1,0,0, 0,0,0,0], [0,1,1,0, 0,0,1,0, 0,0,1,0, 0,0,0,0]],
            6 => [[0,0,0,0, 0,1,0,0, 0,1,1,1, 0,0,0,0], [0,0,1,1, 0,0,1,0, 0,0,1,0, 0,0,0,0], [0,0,0,0, 0,1,1,1, 0,0,0,1, 0,0,0,0], [0,0,1,0, 0,0,1,0, 0,1,1,0, 0,0,0,0]],
            _ => [[0; 16]; 4],
        };
        
        let rotation_data = piece_data[self.rotation % 4];
        for py in 0..4 {
            for px in 0..4 {
                if rotation_data[py * 4 + px] == 1 {
                    let sx = self.x as i32 + (px as i32 * self.size as i32);
                    let sy = self.y as i32 + (py as i32 * self.size as i32);
                    for by in 0..self.size {
                        for bx in 0..self.size {
                            let dx = sx + bx as i32;
                            let dy = sy + by as i32;
                            if dx >= 0 && dx < 400 && dy >= 0 && dy < 240 {
                                display.draw_pixel(dx as usize, dy as usize, Pixel::Black, ctx);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl ZeugtrisMenuPage {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut background_pieces = Vec::new();
        for _ in 0..15 { background_pieces.push(BackgroundPiece::new()); }
        
        let logo_path = "/home/kramwriter/KramWriter/assets/zeugtris/zeugtris_logo.bmp";
        let (logo_data, logo_width, logo_height) = match std::fs::read(logo_path) {
            Ok(data) => Self::parse_bmp(&data).map(|(p, w, h)| (Some(p), w, h)).unwrap_or((None, 0, 0)),
            Err(_) => (None, 0, 0),
        };
        
        Self {
            background_pieces,
            last_update_time: Instant::now(),
            logo_data,
            logo_width,
            logo_height,
        }
    }
    
    fn parse_bmp(data: &[u8]) -> Option<(Vec<Pixel>, usize, usize)> {
        if data.len() < 54 || data[0] != 0x42 || data[1] != 0x4D { return None; }
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let bpp = u16::from_le_bytes([data[28], data[29]]) as usize;
        let offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        
        let mut pixels = Vec::with_capacity(width * height);
        if bpp == 32 {
            for y in 0..height {
                let row_start = offset + (height - 1 - y) * width * 4;
                for x in 0..width {
                    let p = row_start + x * 4;
                    let lum = (data[p+2] as u32 * 299 + data[p+1] as u32 * 587 + data[p] as u32 * 114) / 1000;
                    pixels.push(if data[p+3] < 128 || lum > 128 { Pixel::White } else { Pixel::Black });
                }
            }
            Some((pixels, width, height))
        } else { None } // Add 24 or 1 bpp logic back if needed
    }

    fn draw_logo(&self, display: &mut SharpDisplay, ctx: &Context) {
        if let Some(pixels) = &self.logo_data {
            let start_x = (400usize.saturating_sub(self.logo_width)) / 2;
            let start_y = (240usize.saturating_sub(self.logo_height)) / 2;
            for y in 0..self.logo_height {
                for x in 0..self.logo_width {
                    if pixels[y * self.logo_width + x] == Pixel::Black {
                        let dx = start_x + x;
                        let dy = start_y + y;
                        if dx < 400 && dy < 240 {
                            display.draw_pixel(dx, dy, Pixel::Black, ctx);
                        }
                    }
                }
            }
        }
    }
}

impl Page for ZeugtrisMenuPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        // Handle Background Animation Timing
        let now = Instant::now();
        let delta = now.duration_since(self.last_update_time).as_secs_f32();
        self.last_update_time = now;

        for piece in &mut self.background_pieces {
            piece.update(delta);
            if piece.is_off_screen() { piece.reset(); }
        }

        match key {
            Key::Char('\n') => Action::None, // Start game here
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        display.clear(ctx);
        for piece in &self.background_pieces {
            piece.draw(display, ctx);
        }
        self.draw_logo(display, ctx);
    }
}