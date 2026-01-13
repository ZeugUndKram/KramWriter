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

// Tetromino definitions (7 shapes, 4 rotations each, 4x4 grid)
const TETROMINOES: [[[u8; 16]; 4]; 7] = [
    // I
    [[0,0,0,0,1,1,1,1,0,0,0,0,0,0,0,0],
     [0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0],
     [0,0,0,0,0,0,0,0,1,1,1,1,0,0,0,0],
     [0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0]],
    // O
    [[0,0,0,0,0,1,1,0,0,1,1,0,0,0,0,0],
     [0,0,0,0,0,1,1,0,0,1,1,0,0,0,0,0],
     [0,0,0,0,0,1,1,0,0,1,1,0,0,0,0,0],
     [0,0,0,0,0,1,1,0,0,1,1,0,0,0,0,0]],
    // S
    [[0,0,0,0,0,0,1,1,0,1,1,0,0,0,0,0],
     [0,0,1,0,0,0,1,1,0,0,0,1,0,0,0,0],
     [0,0,0,0,0,0,1,1,0,1,1,0,0,0,0,0],
     [0,0,1,0,0,0,1,1,0,0,0,1,0,0,0,0]],
    // Z
    [[0,0,0,0,1,1,0,0,0,1,1,0,0,0,0,0],
     [0,0,0,1,0,0,1,1,0,0,0,0,0,0,0,0],
     [0,0,0,0,1,1,0,0,0,1,1,0,0,0,0,0],
     [0,0,0,1,0,0,1,1,0,0,0,0,0,0,0,0]],
    // T
    [[0,0,0,0,0,1,0,0,1,1,1,0,0,0,0,0],
     [0,0,1,0,0,0,1,1,0,0,1,0,0,0,0,0],
     [0,0,0,0,0,0,0,0,1,1,1,0,0,1,0,0],
     [0,0,1,0,0,1,1,0,0,0,1,0,0,0,0,0]],
    // L
    [[0,0,0,0,1,0,0,0,1,1,1,0,0,0,0,0],
     [0,0,1,1,0,0,1,0,0,0,1,0,0,0,0,0],
     [0,0,0,0,0,0,0,0,1,1,1,0,0,0,1,0],
     [0,0,1,0,0,0,1,0,0,0,1,1,0,0,0,0]],
    // J
    [[0,0,0,0,0,0,0,1,0,1,1,1,0,0,0,0],
     [0,0,1,0,0,0,1,0,0,0,1,1,0,0,0,0],
     [0,0,0,0,0,0,0,0,1,1,1,0,1,0,0,0],
     [0,0,1,1,0,0,1,0,0,0,1,0,0,0,0,0]],
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
            drop_interval: Duration::from_millis(500),  // Start with 500ms drop interval
            next_piece: rand::random::<usize>() % 7,
        };
        
        game.new_piece();
        Ok(game)
    }
    
    fn new_piece(&mut self) {
        self.current_piece = self.next_piece;
        self.next_piece = rand::random::<usize>() % 7;
        self.current_rotation = 0;
        self.current_x = ARENA_WIDTH as i32 / 2 - 2;
        self.current_y = 0;
        
        // Check if game over (new piece can't be placed)
        if !self.valid_position(self.current_piece, self.current_rotation, self.current_x, self.current_y) {
            self.game_over = true;
        }
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
                
                if arena_x < 0 || arena_x >= ARENA_WIDTH as i32 || arena_y >= ARENA_HEIGHT as i32 {
                    return false;
                }
                
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
                
                let arena_x = self.current_x + px as i32;
                let arena_y = self.current_y + py as i32;
                
                if arena_x >= 0 && arena_x < ARENA_WIDTH as i32 && arena_y >= 0 && arena_y < ARENA_HEIGHT as i32 {
                    self.arena[arena_y as usize][arena_x as usize] = self.current_piece as u8 + 1;
                }
            }
        }
        
        self.check_lines();
        self.new_piece();
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
                    for x in 0..ARENA_WIDTH {
                        self.arena[yy][x] = self.arena[yy - 1][x];
                    }
                }
                
                // Clear top line
                for x in 0..ARENA_WIDTH {
                    self.arena[0][x] = 0;
                }
                
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
            
            // Increase speed slightly for every 10 lines
            let speed_increase = (self.score / 1000) as u32;
            self.drop_interval = Duration::from_millis(500u64.saturating_sub((speed_increase * 25) as u64).max(100));
        }
    }
    
    fn draw_arena(&self, display: &mut SharpDisplay) {
        // Draw arena border
        let border_left = ARENA_X - 2;
        let border_top = ARENA_Y - 2;
        let border_right = ARENA_X + ARENA_WIDTH * BLOCK_SIZE + 1;
        let border_bottom = ARENA_Y + ARENA_HEIGHT * BLOCK_SIZE + 1;
        
        // Draw border lines
        for x in border_left..=border_right {
            if x < 400 {
                display.draw_pixel(x, border_top, Pixel::Black);
                display.draw_pixel(x, border_bottom, Pixel::Black);
            }
        }
        
        for y in border_top..=border_bottom {
            if y < 240 {
                display.draw_pixel(border_left, y, Pixel::Black);
                if border_right < 400 {
                    display.draw_pixel(border_right, y, Pixel::Black);
                }
            }
        }
        
        // Draw placed blocks
        for y in 0..ARENA_HEIGHT {
            for x in 0..ARENA_WIDTH {
                if self.arena[y][x] != 0 {
                    let block_x = ARENA_X + x * BLOCK_SIZE;
                    let block_y = ARENA_Y + y * BLOCK_SIZE;
                    
                    // Draw filled block
                    for by in 0..BLOCK_SIZE - 1 {
                        for bx in 0..BLOCK_SIZE - 1 {
                            if block_x + bx < 400 && block_y + by < 240 {
                                display.draw_pixel(block_x + bx, block_y + by, Pixel::Black);
                            }
                        }
                    }
                    
                    // Draw highlight (top and left edges)
                    for bx in 0..BLOCK_SIZE - 1 {
                        if block_x + bx < 400 && block_y < 240 {
                            display.draw_pixel(block_x + bx, block_y, Pixel::White);
                        }
                    }
                    for by in 0..BLOCK_SIZE - 1 {
                        if block_x < 400 && block_y + by < 240 {
                            display.draw_pixel(block_x, block_y + by, Pixel::White);
                        }
                    }
                }
            }
        }
        
        // Draw current piece
        for py in 0..4 {
            for px in 0..4 {
                let index = py * 4 + px;
                if TETROMINOES[self.current_piece][self.current_rotation][index] == 0 {
                    continue;
                }
                
                let screen_x = ARENA_X + (self.current_x + px as i32) as usize * BLOCK_SIZE;
                let screen_y = ARENA_Y + (self.current_y + py as i32) as usize * BLOCK_SIZE;
                
                if screen_y >= ARENA_Y && screen_y < ARENA_Y + ARENA_HEIGHT * BLOCK_SIZE {
                    for by in 0..BLOCK_SIZE - 1 {
                        for bx in 0..BLOCK_SIZE - 1 {
                            if screen_x + bx < 400 && screen_y + by < 240 {
                                display.draw_pixel(screen_x + bx, screen_y + by, Pixel::Black);
                            }
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
        
        // Draw next piece preview
        for py in 0..4 {
            for px in 0..4 {
                if TETROMINOES[self.next_piece][0][py * 4 + px] == 0 {
                    continue;
                }
                
                let screen_x = next_x + px * (BLOCK_SIZE - 5);
                let screen_y = next_y + py * (BLOCK_SIZE - 5);
                
                for by in 0..(BLOCK_SIZE - 5) {
                    for bx in 0..(BLOCK_SIZE - 5) {
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
        }
    }
}

impl Page for ZeugtrisPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        // Auto-drop piece based on time
        if !self.game_over && self.last_update.elapsed() >= self.drop_interval {
            if self.valid_position(self.current_piece, self.current_rotation, self.current_x, self.current_y + 1) {
                self.current_y += 1;
            } else {
                self.lock_piece();
            }
            self.last_update = Instant::now();
        }
        
        self.draw_arena(display);
        self.draw_next_piece(display);
        self.draw_score(display);
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        if self.game_over {
            match key {
                Key::Esc => return Ok(Some(PageId::Menu)),
                Key::Ctrl('r') => {
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
                }
            }
            Key::Left => {
                if self.valid_position(self.current_piece, self.current_rotation, self.current_x - 1, self.current_y) {
                    self.current_x -= 1;
                }
            }
            Key::Right => {
                if self.valid_position(self.current_piece, self.current_rotation, self.current_x + 1, self.current_y) {
                    self.current_x += 1;
                }
            }
            Key::Down => {
                if self.valid_position(self.current_piece, self.current_rotation, self.current_x, self.current_y + 1) {
                    self.current_y += 1;
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
            Key::Ctrl('r') => {
                // Restart game
                *self = ZeugtrisPage::new()?;
            }
            _ => {}
        }
        
        Ok(None)
    }
}