use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::time::{Duration, Instant};
use rand::Rng;

// Game constants
const ARENA_WIDTH: usize = 10;
const ARENA_HEIGHT: usize = 20;
const BLOCK_SIZE: usize = 10;  // Pixels per block
const ARENA_X: usize = 50;     // X position of arena on screen
const ARENA_Y: usize = 10;     // Y position of arena on screen
const NEXT_X: usize = 300;     // X position of next piece preview
const NEXT_Y: usize = 30;      // Y position of next piece preview
const HOLD_X: usize = 300;     // X position of hold piece preview
const HOLD_Y: usize = 160;     // Y position of hold piece preview

// SRS Wall kick data
const WALL_KICKS: [[(i32, i32); 5]; 8] = [
    // 0>>1 (0° to 90°)
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    // 1>>0 (90° to 0°)
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    // 1>>2 (90° to 180°)
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    // 2>>1 (180° to 90°)
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    // 2>>3 (180° to 270°)
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    // 3>>2 (270° to 180°)
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    // 3>>0 (270° to 0°)
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    // 0>>3 (0° to 270°)
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
];

// I piece special wall kicks
const WALL_KICKS_I: [[(i32, i32); 5]; 8] = [
    // 0>>1
    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
    // 1>>0
    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
    // 1>>2
    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
    // 2>>1
    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
    // 2>>3
    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
    // 3>>2
    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
    // 3>>0
    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
    // 0>>3
    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
];

// Fixed SRS tetromino definitions
const TETROMINOES: [[[u8; 16]; 4]; 7] = [
    // I piece
    [
        [0,0,0,0, 1,1,1,1, 0,0,0,0, 0,0,0,0],  // 0°
        [0,0,1,0, 0,0,1,0, 0,0,1,0, 0,0,1,0],  // 90°
        [0,0,0,0, 0,0,0,0, 1,1,1,1, 0,0,0,0],  // 180°
        [0,1,0,0, 0,1,0,0, 0,1,0,0, 0,1,0,0],  // 270°
    ],
    // O piece - all rotations same
    [
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],  // 0°
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],  // 90°
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],  // 180°
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],  // 270°
    ],
    // S piece
    [
        [0,0,0,0, 0,0,1,1, 0,1,1,0, 0,0,0,0],  // 0°
        [0,0,1,0, 0,0,1,1, 0,0,0,1, 0,0,0,0],  // 90°
        [0,0,0,0, 0,1,1,0, 1,1,0,0, 0,0,0,0],  // 180°
        [0,1,0,0, 0,1,1,0, 0,0,1,0, 0,0,0,0],  // 270°
    ],
    // Z piece
    [
        [0,0,0,0, 1,1,0,0, 0,1,1,0, 0,0,0,0],  // 0°
        [0,0,0,1, 0,0,1,1, 0,0,1,0, 0,0,0,0],  // 90°
        [0,0,0,0, 0,1,1,0, 0,0,1,1, 0,0,0,0],  // 180°
        [0,0,1,0, 0,1,1,0, 0,1,0,0, 0,0,0,0],  // 270°
    ],
    // T piece
    [
        [0,0,0,0, 0,1,0,0, 1,1,1,0, 0,0,0,0],  // 0°
        [0,0,1,0, 0,1,1,0, 0,0,1,0, 0,0,0,0],  // 90°
        [0,0,0,0, 1,1,1,0, 0,1,0,0, 0,0,0,0],  // 180°
        [0,0,1,0, 0,0,1,1, 0,0,1,0, 0,0,0,0],  // 270°
    ],
    // L piece
    [
        [0,0,0,0, 0,0,0,1, 0,1,1,1, 0,0,0,0],  // 0°
        [0,0,1,0, 0,0,1,0, 0,0,1,1, 0,0,0,0],  // 90°
        [0,0,0,0, 0,1,1,1, 0,1,0,0, 0,0,0,0],  // 180°
        [0,1,1,0, 0,0,1,0, 0,0,1,0, 0,0,0,0],  // 270°
    ],
    // J piece
    [
        [0,0,0,0, 0,1,0,0, 0,1,1,1, 0,0,0,0],  // 0°
        [0,0,1,1, 0,0,1,0, 0,0,1,0, 0,0,0,0],  // 90°
        [0,0,0,0, 0,1,1,1, 0,0,0,1, 0,0,0,0],  // 180°
        [0,0,1,0, 0,0,1,0, 0,1,1,0, 0,0,0,0],  // 270°
    ],
];

pub struct ZeugtrisPage {
    arena: [[u8; ARENA_WIDTH]; ARENA_HEIGHT],
    score: u32,
    lines_cleared: u32,
    level: u32,
    game_over: bool,
    paused: bool,
    current_piece: usize,
    current_rotation: usize,
    current_x: i32,
    current_y: i32,
    last_update: Instant,
    drop_interval: Duration,
    next_piece: usize,
    hold_piece: Option<usize>,
    can_hold: bool,
    needs_redraw: bool,
    last_frame_time: Instant,
    frame_count: u32,
}

impl ZeugtrisPage {
    pub fn new() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let mut game = Self {
            arena: [[0; ARENA_WIDTH]; ARENA_HEIGHT],
            score: 0,
            lines_cleared: 0,
            level: 1,
            game_over: false,
            paused: false,
            current_piece: rng.gen_range(0..7),
            current_rotation: 0,
            current_x: ARENA_WIDTH as i32 / 2 - 2,
            current_y: 0,
            last_update: Instant::now(),
            drop_interval: Duration::from_millis(1000),
            next_piece: rng.gen_range(0..7),
            hold_piece: None,
            can_hold: true,
            needs_redraw: true,
            last_frame_time: Instant::now(),
            frame_count: 0,
        };
        
        // Generate new next piece
        game.next_piece = rng.gen_range(0..7);
        
        Ok(game)
    }
    
    fn new_piece(&mut self) {
        let mut rng = rand::thread_rng();
        
        self.current_piece = self.next_piece;
        self.next_piece = rng.gen_range(0..7);
        self.current_rotation = 0;
        self.current_x = ARENA_WIDTH as i32 / 2 - 2;
        self.current_y = 0;
        self.can_hold = true;
        
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
    
    fn try_wall_kick(&self, piece: usize, from_rot: usize, to_rot: usize, x: i32, y: i32) -> Option<(i32, i32)> {
        let kick_table = if piece == 0 { &WALL_KICKS_I } else { &WALL_KICKS };
        
        // Determine which kick table to use
        let kick_index = match (from_rot, to_rot) {
            (0, 1) => 0,
            (1, 0) => 1,
            (1, 2) => 2,
            (2, 1) => 3,
            (2, 3) => 4,
            (3, 2) => 5,
            (3, 0) => 6,
            (0, 3) => 7,
            _ => return None,
        };
        
        // Try all kick offsets
        for &(kx, ky) in &kick_table[kick_index] {
            let new_x = x + kx;
            let new_y = y + ky;
            if self.valid_position(piece, to_rot, new_x, new_y) {
                return Some((new_x, new_y));
            }
        }
        
        None
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
        let mut new_arena = [[0u8; ARENA_WIDTH]; ARENA_HEIGHT];
        let mut new_row = ARENA_HEIGHT - 1;
        
        // Scan from bottom to top
        for row in (0..ARENA_HEIGHT).rev() {
            let mut line_full = true;
            for x in 0..ARENA_WIDTH {
                if self.arena[row][x] == 0 {
                    line_full = false;
                    break;
                }
            }
            
            if !line_full {
                // Copy non-full lines to new arena
                new_arena[new_row].copy_from_slice(&self.arena[row]);
                new_row = new_row.saturating_sub(1);
            } else {
                lines_cleared += 1;
            }
        }
        
        // Replace old arena with new one
        self.arena = new_arena;
        
        if lines_cleared > 0 {
            self.lines_cleared += lines_cleared as u32;
            
            // Update score using standard Tetris scoring
            let line_points = match lines_cleared {
                1 => 40,
                2 => 100,
                3 => 300,
                4 => 1200,  // Tetris!
                _ => 0,
            };
            
            self.score += line_points * (self.level + 1);
            
            // Level up every 10 lines
            self.level = (self.lines_cleared / 10) + 1;
            
            // Increase speed with level (cap at 50ms drop interval)
            let drop_ms = (1000.0 * (0.8_f32).powf((self.level - 1) as f32)).max(50.0) as u64;
            self.drop_interval = Duration::from_millis(drop_ms);
        }
        
        self.needs_redraw = true;
    }
    
    fn ghost_y(&self) -> i32 {
        let mut ghost_y = self.current_y;
        while self.valid_position(
            self.current_piece, 
            self.current_rotation, 
            self.current_x, 
            ghost_y + 1
        ) {
            ghost_y += 1;
        }
        ghost_y
    }
    
    fn hold_current_piece(&mut self) {
        if !self.can_hold || self.game_over || self.paused {
            return;
        }
        
        if let Some(hold) = self.hold_piece {
            // Swap current piece with held piece
            let temp = self.current_piece;
            self.current_piece = hold;
            self.current_rotation = 0;
            self.current_x = ARENA_WIDTH as i32 / 2 - 2;
            self.current_y = 0;
            self.hold_piece = Some(temp);
        } else {
            // First hold - just store current piece and get new one
            self.hold_piece = Some(self.current_piece);
            self.current_piece = self.next_piece;
            let mut rng = rand::thread_rng();
            self.next_piece = rng.gen_range(0..7);
            self.current_rotation = 0;
            self.current_x = ARENA_WIDTH as i32 / 2 - 2;
            self.current_y = 0;
        }
        
        self.can_hold = false;
        
        // Check if new position is valid
        if !self.valid_position(self.current_piece, self.current_rotation, self.current_x, self.current_y) {
            self.game_over = true;
        }
        
        self.needs_redraw = true;
    }
    
    fn update_game(&mut self) {
        if self.game_over || self.paused {
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
    
    fn draw_block(&self, display: &mut SharpDisplay, x: usize, y: usize, is_ghost: bool) {
        let block_x = ARENA_X + x * BLOCK_SIZE;
        let block_y = ARENA_Y + y * BLOCK_SIZE;
        
        if is_ghost {
            // Draw ghost piece (dotted outline)
            for by in 0..BLOCK_SIZE {
                for bx in 0..BLOCK_SIZE {
                    if (bx + by) % 3 == 0 {  // Create dotted pattern
                        if block_x + bx < 400 && block_y + by < 240 {
                            display.draw_pixel(block_x + bx, block_y + by, Pixel::Black);
                        }
                    }
                }
            }
        } else {
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
                    // Top and bottom borders
                    display.draw_pixel(block_x + bx, block_y, Pixel::Black);
                    display.draw_pixel(block_x + bx, block_y + BLOCK_SIZE - 1, Pixel::Black);
                }
            }
            for by in 0..BLOCK_SIZE {
                if block_y + by < 240 {
                    // Left and right borders
                    display.draw_pixel(block_x, block_y + by, Pixel::Black);
                    display.draw_pixel(block_x + BLOCK_SIZE - 1, block_y + by, Pixel::Black);
                }
            }
        }
    }
    
    fn draw_piece(&self, display: &mut SharpDisplay, x: i32, y: i32, piece: usize, rotation: usize, is_ghost: bool) {
        for py in 0..4 {
            for px in 0..4 {
                let index = py * 4 + px;
                if TETROMINOES[piece][rotation][index] == 0 {
                    continue;
                }
                
                let block_x = (x + px as i32) as usize;
                let block_y = (y + py as i32) as usize;
                
                if block_x < ARENA_WIDTH && block_y < ARENA_HEIGHT {
                    self.draw_block(display, block_x, block_y, is_ghost);
                }
            }
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
                    self.draw_block(display, x, y, false);
                }
            }
        }
        
        // Draw ghost piece
        if !self.game_over && !self.paused {
            let ghost_y = self.ghost_y();
            self.draw_piece(display, self.current_x, ghost_y, self.current_piece, self.current_rotation, true);
        }
        
        // Draw current piece (if game not over and not paused)
        if !self.game_over && !self.paused {
            self.draw_piece(display, self.current_x, self.current_y, self.current_piece, self.current_rotation, false);
        }
    }
    
    fn draw_preview(&self, display: &mut SharpDisplay, x: usize, y: usize, piece: usize, title: &str) {
        let preview_size = 6;
        
        // Draw title
        self.draw_simple_text(display, x, y - 15, title);
        
        // Draw preview piece
        for py in 0..4 {
            for px in 0..4 {
                if TETROMINOES[piece][0][py * 4 + px] == 0 {
                    continue;
                }
                
                let screen_x = x + px * (preview_size + 1);
                let screen_y = y + py * (preview_size + 1);
                
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
    
    fn draw_simple_text(&self, display: &mut SharpDisplay, x: usize, y: usize, text: &str) {
        // Simple 5x7 font drawing
        let font_width = 5;
        let font_height = 7;
        let char_spacing = 1;
        
        for (i, ch) in text.chars().enumerate() {
            let char_x = x + i * (font_width + char_spacing);
            
            // Draw character pixels
            for cy in 0..font_height {
                for cx in 0..font_width {
                    let pixel_on = self.get_font_pixel(ch, cx, cy);
                    
                    if pixel_on && char_x + cx < 400 && y + cy < 240 {
                        display.draw_pixel(char_x + cx, y + cy, Pixel::Black);
                    }
                }
            }
        }
    }
    
    fn get_font_pixel(&self, ch: char, x: usize, y: usize) -> bool {
        // Very basic 5x7 font
        match ch {
            // Numbers
            '0' => y == 0 || y == 6 || x == 0 || x == 4 || (y == 3 && x == 2),
            '1' => x == 2 || (y == 6 && (x == 1 || x == 2 || x == 3)) || (y == 1 && x == 1),
            '2' => y == 0 || y == 3 || y == 6 || (y < 3 && x == 4) || (y > 3 && x == 0),
            '3' => y == 0 || y == 3 || y == 6 || x == 4,
            '4' => x == 0 && y < 4 || x == 4 || y == 3,
            '5' => y == 0 || y == 3 || y == 6 || (y < 3 && x == 0) || (y > 3 && x == 4),
            '6' => y == 0 || y == 3 || y == 6 || x == 0 || (y > 3 && x == 4),
            '7' => y == 0 || (x == 4 && y > 0),
            '8' => y == 0 || y == 3 || y == 6 || x == 0 || x == 4,
            '9' => y == 0 || y == 3 || x == 4 || (y < 3 && x == 0),
            
            // Uppercase letters
            'A' => y == 0 || x == 0 || x == 4 || y == 3,
            'B' => y == 0 || y == 3 || y == 6 || x == 0 || (x == 4 && (y < 3 || y > 3)),
            'C' => y == 0 || y == 6 || x == 0,
            'D' => y == 0 || y == 6 || x == 0 || (x == 4 && y > 0 && y < 6),
            'E' => y == 0 || y == 3 || y == 6 || x == 0,
            'F' => y == 0 || y == 3 || x == 0,
            'G' => y == 0 || y == 6 || x == 0 || (x == 4 && y > 3) || (y == 3 && x > 1),
            'H' => x == 0 || x == 4 || y == 3,
            'I' => y == 0 || y == 6 || x == 2,
            'J' => y == 0 || x == 2 || (y == 6 && x < 3) || (x == 0 && y > 4),
            'K' => x == 0 || (x + y == 5) || (y == x + 1),
            'L' => x == 0 || y == 6,
            'M' => x == 0 || x == 4 || (y == 1 && (x == 1 || x == 3)) || (y == 2 && x == 2),
            'N' => x == 0 || x == 4 || (x == y),
            'O' => y == 0 || y == 6 || x == 0 || x == 4,
            'P' => y == 0 || y == 3 || x == 0 || (x == 4 && y < 3),
            'Q' => y == 0 || y == 6 || x == 0 || x == 4 || (x == 3 && y == 5) || (x == 2 && y == 4),
            'R' => y == 0 || y == 3 || x == 0 || (x == 4 && y < 3) || (x == y - 2 && y > 3),
            'S' => y == 0 || y == 3 || y == 6 || (y < 3 && x == 0) || (y > 3 && x == 4),
            'T' => y == 0 || x == 2,
            'U' => x == 0 || x == 4 || y == 6,
            'V' => (x == 0 && y < 6) || (x == 4 && y < 6) || (x == 2 && y == 6),
            'W' => x == 0 || x == 4 || (y == 5 && x == 2) || (y == 4 && x == 1) || (y == 4 && x == 3),
            'X' => (x == 0 && y == 0) || (x == 4 && y == 0) || (x == 0 && y == 6) || (x == 4 && y == 6) || (x == y) || (x + y == 4),
            'Y' => (x == 0 && y < 3) || (x == 4 && y < 3) || (x == 2 && y >= 3),
            'Z' => y == 0 || y == 6 || (x + y == 4),
            
            // Punctuation
            ':' => (y == 2 || y == 4) && x == 2,
            ' ' => false,
            _ => cy == 0 || cy == 6 || cx == 0 || cx == 4,
        }
    }
    
    fn draw_score(&self, display: &mut SharpDisplay) {
        let score_x = 300;
        let mut score_y = 100;
        
        self.draw_simple_text(display, score_x, score_y, &format!("SCORE: {}", self.score));
        score_y += 20;
        self.draw_simple_text(display, score_x, score_y, &format!("LEVEL: {}", self.level));
        score_y += 20;
        self.draw_simple_text(display, score_x, score_y, &format!("LINES: {}", self.lines_cleared));
        score_y += 40;
        
        if self.game_over {
            self.draw_simple_text(display, score_x, score_y, "GAME OVER");
            score_y += 20;
        } else if self.paused {
            self.draw_simple_text(display, score_x, score_y, "PAUSED");
            score_y += 20;
        }
        
        score_y += 20;
        self.draw_simple_text(display, score_x, score_y, "CONTROLS:");
        score_y += 15;
        self.draw_simple_text(display, score_x, score_y, "Z X ROTATE");
        score_y += 15;
        self.draw_simple_text(display, score_x, score_y, "ARROWS MOVE");
        score_y += 15;
        self.draw_simple_text(display, score_x, score_y, "SPACE DROP");
        score_y += 15;
        self.draw_simple_text(display, score_x, score_y, "C HOLD");
        score_y += 15;
        self.draw_simple_text(display, score_x, score_y, "P PAUSE");
        score_y += 15;
        self.draw_simple_text(display, score_x, score_y, "ESC MENU");
        score_y += 15;
        self.draw_simple_text(display, score_x, score_y, "R RESTART");
    }
}

impl Page for ZeugtrisPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        // Update game state
        self.update_game();
        
        // Only redraw if needed or enough time has passed for smooth animation
        let now = Instant::now();
        let time_since_last_frame = now.duration_since(self.last_frame_time);
        self.frame_count += 1;
        
        // Force redraw every 8 frames for blinking effects
        let force_redraw = self.frame_count % 8 < 4;
        
        if self.needs_redraw || force_redraw || time_since_last_frame >= Duration::from_millis(16) {
            display.clear()?;
            
            self.draw_arena(display);
            self.draw_preview(display, NEXT_X, NEXT_Y, self.next_piece, "NEXT:");
            
            if let Some(hold_piece) = self.hold_piece {
                self.draw_preview(display, HOLD_X, HOLD_Y, hold_piece, "HOLD:");
            } else {
                self.draw_simple_text(display, HOLD_X, HOLD_Y - 15, "HOLD:");
            }
            
            self.draw_score(display);
            
            display.update()?;
            
            self.needs_redraw = false;
            self.last_frame_time = now;
        }
        
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        // Handle game over state
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
        
        // Handle pause
        if key == Key::Char('p') || key == Key::Char('P') {
            self.paused = !self.paused;
            self.needs_redraw = true;
            return Ok(None);
        }
        
        if self.paused {
            match key {
                Key::Esc => return Ok(Some(PageId::ZeugtrisMenu)),
                Key::Char('p') | Key::Char('P') => {
                    self.paused = false;
                    self.needs_redraw = true;
                }
                _ => {}
            }
            return Ok(None);
        }
        
        match key {
            // Rotate clockwise (Z key)
            Key::Char('z') | Key::Char('Z') | Key::Char('x') | Key::Char('X') => {
                let next_rotation = (self.current_rotation + 1) % 4;
                if self.valid_position(self.current_piece, next_rotation, self.current_x, self.current_y) {
                    self.current_rotation = next_rotation;
                    self.needs_redraw = true;
                } else {
                    // Try wall kicks
                    if let Some((new_x, new_y)) = self.try_wall_kick(
                        self.current_piece,
                        self.current_rotation,
                        next_rotation,
                        self.current_x,
                        self.current_y
                    ) {
                        self.current_rotation = next_rotation;
                        self.current_x = new_x;
                        self.current_y = new_y;
                        self.needs_redraw = true;
                    }
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
                    self.last_update = Instant::now(); // Reset drop timer
                    self.needs_redraw = true;
                } else {
                    self.lock_piece();
                }
            }
            Key::Up | Key::Char(' ') => {
                // Hard drop
                while self.valid_position(self.current_piece, self.current_rotation, self.current_x, self.current_y + 1) {
                    self.current_y += 1;
                }
                self.lock_piece();
            }
            Key::Char('c') | Key::Char('C') => {
                self.hold_current_piece();
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