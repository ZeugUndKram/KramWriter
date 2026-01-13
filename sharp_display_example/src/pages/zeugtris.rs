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
const HOLD_Y: usize = 100;     // Y position of hold piece preview
const SCORE_X: usize = 300;    // X position for score display
const SCORE_Y: usize = 160;    // Y position for score display

// SRS Wall kick data
const WALL_KICKS: [[(i32, i32); 5]; 8] = [
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
];

// I piece special wall kicks
const WALL_KICKS_I: [[(i32, i32); 5]; 8] = [
    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
];

// Tetromino definitions
const TETROMINOES: [[[u8; 16]; 4]; 7] = [
    // I
    [
        [0,0,0,0, 1,1,1,1, 0,0,0,0, 0,0,0,0],
        [0,0,1,0, 0,0,1,0, 0,0,1,0, 0,0,1,0],
        [0,0,0,0, 0,0,0,0, 1,1,1,1, 0,0,0,0],
        [0,1,0,0, 0,1,0,0, 0,1,0,0, 0,1,0,0],
    ],
    // O
    [
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
        [0,0,0,0, 0,1,1,0, 0,1,1,0, 0,0,0,0],
    ],
    // S
    [
        [0,0,0,0, 0,0,1,1, 0,1,1,0, 0,0,0,0],
        [0,0,1,0, 0,0,1,1, 0,0,0,1, 0,0,0,0],
        [0,0,0,0, 0,1,1,0, 1,1,0,0, 0,0,0,0],
        [0,1,0,0, 0,1,1,0, 0,0,1,0, 0,0,0,0],
    ],
    // Z
    [
        [0,0,0,0, 1,1,0,0, 0,1,1,0, 0,0,0,0],
        [0,0,0,1, 0,0,1,1, 0,0,1,0, 0,0,0,0],
        [0,0,0,0, 0,1,1,0, 0,0,1,1, 0,0,0,0],
        [0,0,1,0, 0,1,1,0, 0,1,0,0, 0,0,0,0],
    ],
    // T
    [
        [0,0,0,0, 0,1,0,0, 1,1,1,0, 0,0,0,0],
        [0,0,1,0, 0,1,1,0, 0,0,1,0, 0,0,0,0],
        [0,0,0,0, 1,1,1,0, 0,1,0,0, 0,0,0,0],
        [0,0,1,0, 0,0,1,1, 0,0,1,0, 0,0,0,0],
    ],
    // L
    [
        [0,0,0,0, 0,0,0,1, 0,1,1,1, 0,0,0,0],
        [0,0,1,0, 0,0,1,0, 0,0,1,1, 0,0,0,0],
        [0,0,0,0, 0,1,1,1, 0,1,0,0, 0,0,0,0],
        [0,1,1,0, 0,0,1,0, 0,0,1,0, 0,0,0,0],
    ],
    // J
    [
        [0,0,0,0, 0,1,0,0, 0,1,1,1, 0,0,0,0],
        [0,0,1,1, 0,0,1,0, 0,0,1,0, 0,0,0,0],
        [0,0,0,0, 0,1,1,1, 0,0,0,1, 0,0,0,0],
        [0,0,1,0, 0,0,1,0, 0,1,1,0, 0,0,0,0],
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
    
    fn try_wall_kick(&self, piece: usize, from_rot: usize, to_rot: usize, x: i32, y: i32) -> Option<(i32, i32)> {
        let kick_table = if piece == 0 { &WALL_KICKS_I } else { &WALL_KICKS };
        
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
        
        for row in (0..ARENA_HEIGHT).rev() {
            let mut line_full = true;
            for x in 0..ARENA_WIDTH {
                if self.arena[row][x] == 0 {
                    line_full = false;
                    break;
                }
            }
            
            if !line_full {
                new_arena[new_row].copy_from_slice(&self.arena[row]);
                new_row = new_row.saturating_sub(1);
            } else {
                lines_cleared += 1;
            }
        }
        
        self.arena = new_arena;
        
        if lines_cleared > 0 {
            self.lines_cleared += lines_cleared as u32;
            
            let line_points = match lines_cleared {
                1 => 40,
                2 => 100,
                3 => 300,
                4 => 1200,
                _ => 0,
            };
            
            self.score += line_points * (self.level + 1);
            self.level = (self.lines_cleared / 10) + 1;
            
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
            let temp = self.current_piece;
            self.current_piece = hold;
            self.current_rotation = 0;
            self.current_x = ARENA_WIDTH as i32 / 2 - 2;
            self.current_y = 0;
            self.hold_piece = Some(temp);
        } else {
            self.hold_piece = Some(self.current_piece);
            self.current_piece = self.next_piece;
            let mut rng = rand::thread_rng();
            self.next_piece = rng.gen_range(0..7);
            self.current_rotation = 0;
            self.current_x = ARENA_WIDTH as i32 / 2 - 2;
            self.current_y = 0;
        }
        
        self.can_hold = false;
        
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
            // Draw ghost piece (checkerboard pattern)
            for by in 0..BLOCK_SIZE {
                for bx in 0..BLOCK_SIZE {
                    if (bx + by) % 2 == 0 && block_x + bx < 400 && block_y + by < 240 {
                        display.draw_pixel(block_x + bx, block_y + by, Pixel::Black);
                    }
                }
            }
        } else {
            // Draw solid block
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
    
    fn draw_preview(&self, display: &mut SharpDisplay, x: usize, y: usize, piece: usize) {
        let preview_size = 8;
        
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
    
    fn draw_score_bar(&self, display: &mut SharpDisplay, x: usize, y: usize, value: u32, max: u32, height: usize) {
        let bar_width = 80;
        let filled_width = (value as f32 / max as f32 * bar_width as f32) as usize;
        
        // Draw bar background
        for by in 0..height {
            for bx in 0..bar_width {
                if x + bx < 400 && y + by < 240 {
                    display.draw_pixel(x + bx, y + by, Pixel::Black);
                }
            }
        }
        
        // Draw filled portion
        for by in 1..height - 1 {
            for bx in 1..filled_width.min(bar_width - 2) {
                if x + bx < 400 && y + by < 240 {
                    display.draw_pixel(x + bx, y + by, Pixel::Black);
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
        
        // Draw current piece
        if !self.game_over && !self.paused {
            self.draw_piece(display, self.current_x, self.current_y, self.current_piece, self.current_rotation, false);
        }
    }
    
    fn draw_game_info(&self, display: &mut SharpDisplay) {
        // Draw next piece preview with label box
        let next_box_x = NEXT_X - 5;
        let next_box_y = NEXT_Y - 15;
        let next_box_width = 50;
        let next_box_height = 70;
        
        // Draw box
        for x in next_box_x..next_box_x + next_box_width {
            if x < 400 {
                display.draw_pixel(x, next_box_y, Pixel::Black);
                display.draw_pixel(x, next_box_y + next_box_height, Pixel::Black);
            }
        }
        for y in next_box_y..next_box_y + next_box_height {
            if y < 240 {
                display.draw_pixel(next_box_x, y, Pixel::Black);
                display.draw_pixel(next_box_x + next_box_width, y, Pixel::Black);
            }
        }
        
        // Draw next piece
        self.draw_preview(display, NEXT_X, NEXT_Y, self.next_piece);
        
        // Draw hold piece preview with label box
        let hold_box_x = HOLD_X - 5;
        let hold_box_y = HOLD_Y - 15;
        
        // Draw box
        for x in hold_box_x..hold_box_x + next_box_width {
            if x < 400 {
                display.draw_pixel(x, hold_box_y, Pixel::Black);
                display.draw_pixel(x, hold_box_y + next_box_height, Pixel::Black);
            }
        }
        for y in hold_box_y..hold_box_y + next_box_height {
            if y < 240 {
                display.draw_pixel(hold_box_x, y, Pixel::Black);
                display.draw_pixel(hold_box_x + next_box_width, y, Pixel::Black);
            }
        }
        
        // Draw hold piece or empty indicator
        if let Some(hold_piece) = self.hold_piece {
            self.draw_preview(display, HOLD_X, HOLD_Y, hold_piece);
        } else {
            // Draw empty indicator (cross)
            let cross_size = 20;
            let center_x = HOLD_X + 15;
            let center_y = HOLD_Y + 25;
            
            for i in 0..cross_size {
                if center_x + i < 400 && center_y < 240 {
                    display.draw_pixel(center_x + i, center_y, Pixel::Black);
                    display.draw_pixel(center_x, center_y + i, Pixel::Black);
                }
                if center_x >= i && center_y < 240 {
                    display.draw_pixel(center_x - i, center_y, Pixel::Black);
                }
                if center_x < 400 && center_y + i < 240 {
                    display.draw_pixel(center_x, center_y + i, Pixel::Black);
                }
            }
        }
        
        // Draw score/level info with visual indicators
        let info_x = SCORE_X;
        let mut info_y = SCORE_Y;
        
        // Draw level indicator (bar)
        self.draw_score_bar(display, info_x, info_y, self.level.min(20), 20, 10);
        info_y += 20;
        
        // Draw lines indicator (bar)
        self.draw_score_bar(display, info_x, info_y, (self.lines_cleared % 10) as u32, 10, 10);
        
        // Draw game over indicator
        if self.game_over {
            let center_x = ARENA_X + (ARENA_WIDTH * BLOCK_SIZE) / 2;
            let center_y = ARENA_Y + (ARENA_HEIGHT * BLOCK_SIZE) / 2;
            
            // Draw X mark
            for i in 0..30 {
                if center_x + i < 400 && center_y + i < 240 {
                    display.draw_pixel(center_x + i, center_y + i, Pixel::Black);
                }
                if center_x + i < 400 && center_y >= i && center_y - i < 240 {
                    display.draw_pixel(center_x + i, center_y - i, Pixel::Black);
                }
                if center_x >= i && center_y + i < 240 {
                    display.draw_pixel(center_x - i, center_y + i, Pixel::Black);
                }
                if center_x >= i && center_y >= i {
                    display.draw_pixel(center_x - i, center_y - i, Pixel::Black);
                }
            }
        }
        
        // Draw pause indicator
        if self.paused {
            let center_x = ARENA_X + (ARENA_WIDTH * BLOCK_SIZE) / 2;
            let center_y = ARENA_Y + (ARENA_HEIGHT * BLOCK_SIZE) / 2;
            
            // Draw pause symbol (two vertical bars)
            // Left bar
            for y in 0..30 {
                let draw_y = center_y as i32 - 15 + y as i32;
                if draw_y >= 0 && draw_y < 240 {
                    for x in 0..3 {
                        let draw_x = center_x as i32 - 6 + x as i32;
                        if draw_x >= 0 && draw_x < 400 {
                            display.draw_pixel(draw_x as usize, draw_y as usize, Pixel::Black);
                        }
                    }
                }
            }
            
            // Right bar
            for y in 0..30 {
                let draw_y = center_y as i32 - 15 + y as i32;
                if draw_y >= 0 && draw_y < 240 {
                    for x in 0..3 {
                        let draw_x = center_x as i32 + 4 + x as i32;
                        if draw_x >= 0 && draw_x < 400 {
                            display.draw_pixel(draw_x as usize, draw_y as usize, Pixel::Black);
                        }
                    }
                }
            }
        }
    }
}

impl Page for ZeugtrisPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        self.update_game();
        
        let now = Instant::now();
        let time_since_last_frame = now.duration_since(self.last_frame_time);
        self.frame_count += 1;
        
        let force_redraw = self.frame_count % 8 < 4;
        
        if self.needs_redraw || force_redraw || time_since_last_frame >= Duration::from_millis(16) {
            display.clear()?;
            
            self.draw_arena(display);
            self.draw_game_info(display);
            
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
                    *self = ZeugtrisPage::new()?;
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }
        
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
            Key::Char('z') | Key::Char('Z') | Key::Char('x') | Key::Char('X') => {
                let next_rotation = (self.current_rotation + 1) % 4;
                if self.valid_position(self.current_piece, next_rotation, self.current_x, self.current_y) {
                    self.current_rotation = next_rotation;
                    self.needs_redraw = true;
                } else if let Some((new_x, new_y)) = self.try_wall_kick(
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
                    self.last_update = Instant::now();
                    self.needs_redraw = true;
                } else {
                    self.lock_piece();
                }
            }
            Key::Up | Key::Char(' ') => {
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
                *self = ZeugtrisPage::new()?;
            }
            _ => {}
        }
        
        Ok(None)
    }
}