use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::time::{Duration, Instant};
use rand::Rng;

pub struct ZeugtrisMenuPage {
    background_pieces: Vec<BackgroundPiece>,
    last_update: Instant,
    frame_count: u32,
    blink_timer: Instant,
    show_press_enter: bool,
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
            size: 6, // Smaller than game pieces
        }
    }
    
    fn update(&mut self) {
        self.y += self.speed;
        self.rotation = (self.rotation as f32 + self.rotation_speed) as usize % 4;
        self.x += self.drift * self.drift_direction;
        
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
                            // Draw with a checkerboard pattern for a semi-transparent look
                            if (bx + by) % 2 == 0 {
                                display.draw_pixel(draw_x as usize, draw_y as usize, Pixel::Black);
                            }
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
        
        // Create 8-12 background pieces
        for _ in 0..rng.gen_range(8..13) {
            background_pieces.push(BackgroundPiece::new());
        }
        
        Ok(Self {
            background_pieces,
            last_update: Instant::now(),
            frame_count: 0,
            blink_timer: Instant::now(),
            show_press_enter: true,
        })
    }
    
    fn update_background(&mut self) {
        let now = Instant::now();
        
        // Update background pieces
        for piece in &mut self.background_pieces {
            piece.update();
            
            // Reset pieces that have fallen off screen
            if piece.is_off_screen() {
                piece.reset();
            }
        }
        
        // Update blink for "PRESS ENTER" indicator
        if now.duration_since(self.blink_timer) >= Duration::from_millis(500) {
            self.show_press_enter = !self.show_press_enter;
            self.blink_timer = now;
        }
        
        self.last_update = now;
        self.frame_count += 1;
    }
    
    fn draw_press_enter_indicator(&self, display: &mut SharpDisplay) {
        if !self.show_press_enter {
            return;
        }
        
        // Draw a simple arrow pointing down in the center bottom
        let center_x = 200;
        let center_y = 200;
        
        // Draw arrow (simple triangle)
        for y in 0..10 {
            let width = 10 - y;
            for x in 0..width {
                if center_x - x >= 0 && center_x - x < 400 && center_y + y < 240 {
                    display.draw_pixel(center_x - x, center_y + y, Pixel::Black);
                }
                if center_x + x < 400 && center_y + y < 240 {
                    display.draw_pixel(center_x + x, center_y + y, Pixel::Black);
                }
            }
        }
        
        // Draw horizontal line
        for x in 0..30 {
            if center_x - 15 + x >= 0 && center_x - 15 + x < 400 && center_y + 10 < 240 {
                display.draw_pixel(center_x - 15 + x as usize, center_y + 10, Pixel::Black);
            }
        }
    }
    
    fn draw_title(&self, display: &mut SharpDisplay) {
        // Draw "ZEUGTRIS" using Tetris blocks
        // Each letter made of 3x5 blocks
        let title_y = 40;
        
        // Z
        for y in 0..5 {
            for x in 0..3 {
                let block_x = 100 + x * 12;
                let block_y = title_y + y * 12;
                
                if (y == 0 || y == 4) || (y == 1 && x == 2) || (y == 2 && x == 1) || (y == 3 && x == 0) {
                    self.draw_title_block(display, block_x, block_y);
                }
            }
        }
        
        // E (offset by 40 pixels)
        for y in 0..5 {
            for x in 0..3 {
                let block_x = 140 + x * 12;
                let block_y = title_y + y * 12;
                
                if (y == 0 || y == 2 || y == 4) || (x == 0) {
                    self.draw_title_block(display, block_x, block_y);
                }
            }
        }
        
        // U (offset by 80 pixels)
        for y in 0..5 {
            for x in 0..3 {
                let block_x = 180 + x * 12;
                let block_y = title_y + y * 12;
                
                if (y < 4 && x == 0) || (y < 4 && x == 2) || (y == 4 && (x == 0 || x == 1 || x == 2)) {
                    self.draw_title_block(display, block_x, block_y);
                }
            }
        }
        
        // G (offset by 120 pixels)
        for y in 0..5 {
            for x in 0..3 {
                let block_x = 220 + x * 12;
                let block_y = title_y + y * 12;
                
                if (y == 0 || y == 4) || (x == 0) || (y == 2 && x > 0) || (y == 3 && x == 2) {
                    self.draw_title_block(display, block_x, block_y);
                }
            }
        }
        
        // T (offset by 160 pixels)
        for y in 0..5 {
            for x in 0..3 {
                let block_x = 260 + x * 12;
                let block_y = title_y + y * 12;
                
                if (y == 0) || (x == 1 && y > 0) {
                    self.draw_title_block(display, block_x, block_y);
                }
            }
        }
        
        // R (offset by 200 pixels)
        for y in 0..5 {
            for x in 0..3 {
                let block_x = 300 + x * 12;
                let block_y = title_y + y * 12;
                
                if (y == 0 || y == 2) || (x == 0) || (y == 1 && x == 2) || (y == 3 && x == 1) || (y == 4 && x == 2) {
                    self.draw_title_block(display, block_x, block_y);
                }
            }
        }
    }
    
    fn draw_title_block(&self, display: &mut SharpDisplay, x: usize, y: usize) {
        // Draw a solid block for title
        for by in 0..8 {
            for bx in 0..8 {
                if x + bx < 400 && y + by < 240 {
                    display.draw_pixel(x + bx, y + by, Pixel::Black);
                }
            }
        }
    }
}

impl Page for ZeugtrisMenuPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        // Update background animation
        self.update_background();
        
        display.clear()?;
        
        // Draw background pieces
        for piece in &self.background_pieces {
            piece.draw(display);
        }
        
        // Draw title
        self.draw_title(display);
        
        // Draw "PRESS ENTER" indicator
        self.draw_press_enter_indicator(display);
        
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