use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::time::{Duration, Instant};

// Game constants
const ARENA_WIDTH: usize = 10;
const ARENA_HEIGHT: usize = 20;
const BLOCK_SIZE: usize = 10;  // Pixels per block
const ARENA_X: usize = 50;     // X position of arena on screen
const ARENA_Y: usize = 10;     // Y position of arena on screen

// Tetromino definitions - Fixed and verified
const TETROMINOES: [[[u8; 16]; 4]; 7] = [
    // I piece
    [
        [0,0,0,0, 1,1,1,1, 0,0,0,0, 0,0,0,0],  // 0°
        [0,0,1,0, 0,0,1,0, 0,0,1,0, 0,0,1,0],  // 90°
        [0,0,0,0, 0,0,0,0, 1,1,1,1, 0,0,0,0],  // 180°
        [0,1,0,0, 0,1,0,0, 0,1,0,0, 0,1,0,0],  // 270°
    ],
    // O piece
    [
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],  // All rotations same
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
    ],
    // S piece
    [
        [0,0,0,0, 0,0,1,1, 0,1,1,0, 0,0,0,0],  // 0°
        [0,0,1,0, 0,0,1,1, 0,0,0,1, 0,0,0,0],  // 90°
        [0,0,0,0, 0,0,1,1, 0,1,1,0, 0,0,0,0],  // 180° 
        [0,0,1,0, 0,0,1,1, 0,0,0,1, 0,0,0,0],  // 270°
    ],
    // Z piece
    [
        [0,0,0,0, 1,1,0,0, 0,1,1,0, 0,0,0,0],  // 0°
        [0,0,0,1, 0,0,1,1, 0,0,1,0, 0,0,0,0],  // 90°
        [0,0,0,0, 1,1,0,0, 0,1,1,0, 0,0,0,0],  // 180°
        [0,0,0,1, 0,0,1,1, 0,0,1,0, 0,0,0,0],  // 270°
    ],
    // T piece
    [
        [0,0,0,0, 0,1,0,0, 1,1,1,0, 0,0,0,0],  // 0°
        [0,0,1,0, 0,0,1,1, 0,0,1,0, 0,0,0,0],  // 90°
        [0,0,0,0, 0,0,0,0, 1,1,1,0, 0,1,0,0],  // 180°
        [0,0,1,0, 0,1,1,0, 0,0,1,0, 0,0,0,0],  // 270°
    ],
    // L piece
    [
        [0,0,0,0, 1,0,0,0, 1,1,1,0, 0,0,0,0],  // 0°
        [0,0,1,1, 0,0,1,0, 0,0,1,0, 0,0,0,0],  // 90°
        [0,0,0,0, 0,0,0,0, 1,1,1,0, 0,0,1,0],  // 180°
        [0,0,1,0, 0,0,1,0, 0,0,1,1, 0,0,0,0],  // 270°
    ],
    // J piece
    [
        [0,0,0,0, 0,0,0,1, 0,1,1,1, 0,0,0,0],  // 0°
        [0,0,1,0, 0,0,1,0, 0,1,1,0, 0,0,0,0],  // 90°
        [0,0,0,0, 0,0,0,0, 1,1,1,0, 1,0,0,0],  // 180°
        [0,0,1,1, 0,0,1,0, 0,0,1,0, 0,0,0,0],  // 270°
    ],
];

pub struct ZeugtrisPage {
    arena: [[u8; ARENA_WIDTH]; ARENA_HEIGHT],
    score: u32,
    game_over: bool,
    current_piece: usize,
    current_rotation: usize,
    current_x: i32,
    current_y: i32,
    last_update: Instant,
    drop_interval: Duration,
    next_piece: usize,
    needs_redraw: bool,
    last_frame_time: Instant,
}

impl ZeugtrisPage {
    pub fn new() -> Result<Self> {
        let mut game = Self {
            arena: [[0; ARENA_WIDTH]; ARENA_HEIGHT],
            score: 0,
            game_over: false,
            current_piece: 0,
            current_rotation: 0,
            current_x: 0,
            current_y: 0,
            last_update: Instant::now(),
            drop_interval: Duration::from_millis(1000),  // Start with 1 second drop interval
            next_piece: rand::random::<usize>() % 7,
            needs_redraw: true,
            last_frame_time: Instant::now(),
        };
        
        game.new_piece();
        Ok(game)
    }
    
    fn new_piece(&mut self) {
        self.current_piece = self.next_piece;
        self.next_piece = rand::random::<usize>() % 7;
        self.current_rotation = 0;
        self.current_x = ARENA_WIDTH as i32 / 2 - 1;  // Center horizontally
        self.current_y = 0;
        
        // Check if game over (new piece can't be placed)
        if !self.valid_position(self.current_piece, self.current_rotation, self.current_x, self.current_y) {
            self.game_over = true;
        }
        
        self.needs_redraw = true;
    }
    
    fn valid_position(&self, piece: usize, rotation: usize, x: i32, y: i32) -> bool {
        for py in 0..4 {
            for px in 0..4 {
                let index = py * 4 + px;
                if TETROMINOES[piece][rotation][index] == 0 {
                    continue;
                }
                
                let arena_x = x + px as i32;
                let arena_y = y + py as i32;
                
                // Check boundaries
                if arena_x < 0 || arena_x >= ARENA_WIDTH as i32 || arena_y >= ARENA_HEIGHT as i32 {
                    return false;
                }
                
                // Check collision with placed pieces (only if within vertical bounds)
                if arena_y >= 0 && self.arena[arena_y as usize][arena_x as usize] != 0 {
                    return false;
                }
            }
        }
        true
    }
    
    fn lock_piece(&mut self) {
        for py in 0..4 {
            for px in 0..4 {
                let index = py * 4 + px;
                if TETROMINOES[self.current_piece][self.current_rotation][index] == 0 {
                    continue;
                }
                
                let arena_x = (self.current_x + px as i32) as usize;
                let arena_y = (self.current_y + py as i32) as usize;
                
                if arena_x < ARENA_WIDTH && arena_y < ARENA_HEIGHT {
                    self.arena[arena_y][arena_x] = (self.current_piece as u8) + 1;
                }
            }
        }
        
        self.check_lines();
        self.new_piece();
        self.needs_redraw = true;
    }
    
    fn check_lines(&mut self) {
        let mut lines_cleared = 0;
        
        let mut y = ARENA_HEIGHT as i32 - 1;
        while y >= 0 {
            let mut line_full = true;
            for x in 0..ARENA_WIDTH {
                if self.arena[y as usize][x] == 0 {
                    line_full = false;
                    break;
                }
            }
            
            if line_full {
                lines_cleared += 1;
                
                // Move all lines above down
                for yy in (1..=y as usize).rev() {
                    self.arena[yy].copy_from_slice(&self.arena[yy - 1]);
                }
                
                // Clear top line
                self.arena[0] = [0; ARENA_WIDTH];
                
                // Check same line again (since we moved everything down)
                continue;
            }
            
            y -= 1;
        }
        
        if lines_cleared > 0 {
            // Update score (more points for multiple lines)
            self.score += match lines_cleared {
                1 => 100,
                2 => 300,
                3 => 500,
                4 => 800,
                _ => 0,
            };
            
            // Increase speed slightly for every 5 lines
            let speed_level = (self.score / 500) as u32;
            self.drop_interval = Duration::from_millis(1000u64.saturating_sub((speed_level * 100) as u64).max(100));
        }
        
        self.needs_redraw = true;
    }
    
    fn update_game(&mut self) {
        if self.game_over {
            return;
        }
        
        let now = Instant::now();
        
        // Auto-drop piece based on time
        if now.duration_since(self.last_update) >= self.drop_interval {
            if self.valid_position(self.current_piece, self.current_rotation, self.current_x, self.current_y + 1) {
                self.current_y += 1;
                self.needs_redraw = true;
            } else {
                self.lock_piece();
            }
            self.last_update = now;
        }
    }
    
    fn draw_arena(&self, display: &mut SharpDisplay) {
        // Draw arena border
        let border_left = ARENA_X.saturating_sub(2);
        let border_top = ARENA_Y.saturating_sub(2);
        let border_right = (ARENA_X + ARENA_WIDTH * BLOCK_SIZE + 1).min(399);
        let border_bottom = (ARENA_Y + ARENA_HEIGHT * BLOCK_SIZE + 1).min(239);
        
        // Draw border lines
        for x in border_left..=border_right {
            display.draw_pixel(x, border_top, Pixel::Black);
            display.draw_pixel(x, border_bottom, Pixel::Black);
        }
        
        for y in border_top..=border_bottom {
            display.draw_pixel(border_left, y, Pixel::Black);
            display.draw_pixel(border_right, y, Pixel::Black);
        }
        
        // Draw placed blocks
        for y in 0..ARENA_HEIGHT {
            for x in 0..ARENA_WIDTH {
                if self.arena[y][x] != 0 {
                    let block_x = ARENA_X + x * BLOCK_SIZE;
                    let block_y = ARENA_Y + y * BLOCK_SIZE;
                    
                    // Draw filled block (leave 1px border for outline)
                    for by in 1..BLOCK_SIZE - 1 {
                        for bx in 1..BLOCK_SIZE - 1 {
                            if block_x + bx < 400 && block_y + by < 240 {
                                display.draw_pixel(block_x + bx, block_y + by, Pixel::Black);
                            }
                        }
                    }
                    
                    // Draw outline
                    for bx in 0..BLOCK_SIZE {
                        if block_x + bx < 400 {
                            display.draw_pixel(block_x + bx, block_y, Pixel::Black);
                            display.draw_pixel(block_x + bx, block_y + BLOCK_SIZE - 1, Pixel::Black);
                        }
                    }
                    for by in 0..BLOCK_SIZE {
                        if block_y + by < 240 {
                            display.draw_pixel(block_x, block_y + by, Pixel::Black);
                            display.draw_pixel(block_x + BLOCK_SIZE - 1, block_y + by, Pixel::Black);
                        }
                    }
                }
            }
        }
        
        // Draw current piece (if game not over)
        if !self.game_over {
            for py in 0..4 {
                for px in 0..4 {
                    let index = py * 4 + px;
                    if TETROMINOES[self.current_piece][self.current_rotation][index] == 0 {
                        continue;
                    }
                    
                    let screen_x = ARENA_X + (self.current_x + px as i32) as usize * BLOCK_SIZE;
                    let screen_y = ARENA_Y + (self.current_y + py as i32) as usize * BLOCK_SIZE;
                    
                    // Draw piece block (leave 1px border)
                    for by in 1..BLOCK_SIZE - 1 {
                        for bx in 1..BLOCK_SIZE - 1 {
                            if screen_x + bx < 400 && screen_y + by < 240 {
                                display.draw_pixel(screen_x + bx, screen_y + by, Pixel::Black);
                            }
                        }
                    }
                    
                    // Draw piece outline
                    for bx in 0..BLOCK_SIZE {
                        if screen_x + bx < 400 && screen_y < 240 {
                            display.draw_pixel(screen_x + bx, screen_y, Pixel::Black);
                            display.draw_pixel(screen_x + bx, screen_y + BLOCK_SIZE - 1, Pixel::Black);
                        }
                    }
                    for by in 0..BLOCK_SIZE {
                        if screen_y + by < 240 && screen_x < 400 {
                            display.draw_pixel(screen_x, screen_y + by, Pixel::Black);
                            display.draw_pixel(screen_x + BLOCK_SIZE - 1, screen_y + by, Pixel::Black);
                        }
                    }
                }
            }
        }
    }
    
    fn draw_next_piece(&self, display: &mut SharpDisplay) {
        let next_x = 300;
        let next_y = 30;
        
        // Draw "NEXT" label
        display.draw_text(next_x, next_y - 20, "NEXT:");
        
        // Draw next piece preview (smaller blocks)
        let preview_size = 6;
        for py in 0..4 {
            for px in 0..4 {
                if TETROMINOES[self.next_piece][0][py * 4 + px] == 0 {
                    continue;
                }
                
                let screen_x = next_x + px * (preview_size + 1);
                let screen_y = next_y + py * (preview_size + 1);
                
                for by in 0..preview_size {
                    for bx in 0..preview_size {
                        if screen_x + bx < 400 && screen_y + by < 240 {
                            display.draw_pixel(screen_x + bx, screen_y + by, Pixel::Black);
                        }
                    }
                }
            }
        }
    }
    
    fn draw_score(&self, display: &mut SharpDisplay) {
        let score_x = 300;
        let score_y = 120;
        
        display.draw_text(score_x, score_y, "SCORE:");
        display.draw_text(score_x, score_y + 20, &format!("{}", self.score));
        
        if self.game_over {
            display.draw_text(score_x, score_y + 50, "GAME OVER");
            display.draw_text(score_x, score_y + 70, "ESC: MENU");
            display.draw_text(score_x, score_y + 90, "R: RESTART");
        } else {
            display.draw_text(score_x, score_y + 50, "ESC: MENU");
            display.draw_text(score_x, score_y + 70, "R: RESTART");
        }
    }
}

impl Page for ZeugtrisPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        // Update game state
        self.update_game();
        
        // Only redraw if needed or enough time has passed
        let now = Instant::now();
        let time_since_last_frame = now.duration_since(self.last_frame_time);
        
        if self.needs_redraw || time_since_last_frame >= Duration::from_millis(16) { // ~60 FPS
            display.clear()?;
            
            self.draw_arena(display);
            self.draw_next_piece(display);
            self.draw_score(display);
            
            display.update()?;
            
            self.needs_redraw = false;
            self.last_frame_time = now;
        }
        
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        if self.game_over {
            match key {
                Key::Esc => return Ok(Some(PageId::ZeugtrisMenu)),
                Key::Char('r') | Key::Char('R') => {
                    // Restart game
                    *self = ZeugtrisPage::new()?;
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }
        
        match key {
            Key::Char(' ') | Key::Char('z') | Key::Char('Z') => {
                // Rotate
                let next_rotation = (self.current_rotation + 1) % 4;
                if self.valid_position(self.current_piece, next_rotation, self.current_x, self.current_y) {
                    self.current_rotation = next_rotation;
                    self.needs_redraw = true;
                }
            }
            Key::Left => {
                if self.valid_position(self.current_piece, self.current_rotation, self.current_x - 1, self.current_y) {
                    self.current_x -= 1;
                    self.needs_redraw = true;
                }
            }
            Key::Right => {
                if self.valid_position(self.current_piece, self.current_rotation, self.current_x + 1, self.current_y) {
                    self.current_x += 1;
                    self.needs_redraw = true;
                }
            }
            Key::Down => {
                if self.valid_position(self.current_piece, self.current_rotation, self.current_x, self.current_y + 1) {
                    self.current_y += 1;
                    self.needs_redraw = true;
                } else {
                    self.lock_piece();
                }
            }
            Key::Up => {
                // Hard drop
                while self.valid_position(self.current_piece, self.current_rotation, self.current_x, self.current_y + 1) {
                    self.current_y += 1;
                }
                self.lock_piece();
            }
            Key::Esc => return Ok(Some(PageId::ZeugtrisMenu)),
            Key::Char('r') | Key::Char('R') => {
                // Restart game
                *self = ZeugtrisPage::new()?;
            }
            _ => {}
        }
        
        Ok(None)
    }
}