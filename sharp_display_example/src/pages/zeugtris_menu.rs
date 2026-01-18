use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::time::{Duration, Instant};
use rand::Rng;

pub struct ZeugtrisMenuPage {
    background_pieces: Vec<BackgroundPiece>,
    last_update_time: Instant,
    frame_count: u32,
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
        // Tetromino definitions
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
        
        // Create 12-18 background pieces
        for _ in 0..rng.gen_range(12..19) {
            background_pieces.push(BackgroundPiece::new());
        }
        
        Ok(Self {
            background_pieces,
            last_update_time: Instant::now(),
            frame_count: 0,
        })
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
