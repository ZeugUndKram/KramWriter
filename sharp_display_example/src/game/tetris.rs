use super::{board::Board, score::Score, tetrimino::{Tetrimino, TetriminoType}};
use crate::display::SharpDisplay;
use anyhow::Result;
use rand::Rng;
use rpi_memory_display::Pixel;
use std::time::{Duration, Instant};

// Game constants
const BLOCK_SIZE: usize = 10;
const ARENA_X: usize = 50;
const ARENA_Y: usize = 10;
const NEXT_X: usize = 300;
const NEXT_Y: usize = 30;
const HOLD_X: usize = 300;
const HOLD_Y: usize = 100;
const SCORE_X: usize = 300;
const SCORE_Y: usize = 160;

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

pub struct TetrisGame {
    board: Board,
    score: Score,
    current_tetrimino: Tetrimino,
    next_tetrimino: Tetrimino,
    hold_tetrimino: Option<Tetrimino>,
    can_hold: bool,
    position: (i32, i32),
    game_over: bool,
    paused: bool,
    last_update: Instant,
    needs_redraw: bool,
}

impl TetrisGame {
    pub fn new() -> Result<Self> {
        let mut game = Self {
            board: Board::new(),
            score: Score::new(),
            current_tetrimino: Tetrimino::random(),
            next_tetrimino: Tetrimino::random(),
            hold_tetrimino: None,
            can_hold: true,
            position: (4, 0), // Center position
            game_over: false,
            paused: false,
            last_update: Instant::now(),
            needs_redraw: true,
        };
        
        // Check initial position
        if game.check_collision(game.position.0, game.position.1, None) {
            game.game_over = true;
        }
        
        Ok(game)
    }
    
    fn check_collision(&self, x: i32, y: i32, rotation: Option<usize>) -> bool {
        self.board.check_collision(&self.current_tetrimino, x, y, rotation)
    }
    
    fn valid_position(&self, x: i32, y: i32, rotation: Option<usize>) -> bool {
        !self.check_collision(x, y, rotation)
    }
    
    fn try_wall_kick(&self, from_rotation: usize, to_rotation: usize, x: i32, y: i32) -> Option<(i32, i32)> {
        let is_i_piece = matches!(self.current_tetrimino.tetrimino_type, TetriminoType::I);
        let kick_table = if is_i_piece { &WALL_KICKS_I } else { &WALL_KICKS };
        
        let kick_index = match (from_rotation, to_rotation) {
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
            if self.valid_position(new_x, new_y, Some(to_rotation)) {
                return Some((new_x, new_y));
            }
        }
        
        None
    }
    
    pub fn move_left(&mut self) -> bool {
        if !self.game_over && !self.paused {
            let new_x = self.position.0 - 1;
            if self.valid_position(new_x, self.position.1, None) {
                self.position.0 = new_x;
                self.needs_redraw = true;
                return true;
            }
        }
        false
    }
    
    pub fn move_right(&mut self) -> bool {
        if !self.game_over && !self.paused {
            let new_x = self.position.0 + 1;
            if self.valid_position(new_x, self.position.1, None) {
                self.position.0 = new_x;
                self.needs_redraw = true;
                return true;
            }
        }
        false
    }
    
    pub fn soft_drop(&mut self) -> bool {
        if !self.game_over && !self.paused {
            let new_y = self.position.1 + 1;
            if self.valid_position(self.position.0, new_y, None) {
                self.position.1 = new_y;
                self.score.add_soft_drop_points(1);
                self.needs_redraw = true;
                return true;
            } else {
                return self.lock_current_piece();
            }
        }
        false
    }
    
    pub fn hard_drop(&mut self) -> bool {
        if self.game_over || self.paused {
            return false;
        }
        
        let mut drop_distance = 0;
        while self.valid_position(self.position.0, self.position.1 + drop_distance + 1, None) {
            drop_distance += 1;
        }
        
        self.position.1 += drop_distance;
        self.score.add_hard_drop_points(drop_distance as u32);
        self.lock_current_piece()
    }
    
    pub fn rotate_left(&mut self) -> bool {
        if self.game_over || self.paused {
            return false;
        }
        
        let from_rotation = self.current_tetrimino.rotation;
        let to_rotation = (from_rotation + 3) % 4; // +3 ≡ -1 mod 4
        
        // Try normal rotation first
        if self.valid_position(self.position.0, self.position.1, Some(to_rotation)) {
            self.current_tetrimino.rotate_left();
            self.needs_redraw = true;
            return true;
        }
        
        // Try wall kicks
        if let Some((new_x, new_y)) = self.try_wall_kick(from_rotation, to_rotation, self.position.0, self.position.1) {
            self.current_tetrimino.rotate_left();
            self.position = (new_x, new_y);
            self.needs_redraw = true;
            return true;
        }
        
        false
    }
    
    pub fn rotate_right(&mut self) -> bool {
        if self.game_over || self.paused {
            return false;
        }
        
        let from_rotation = self.current_tetrimino.rotation;
        let to_rotation = (from_rotation + 1) % 4;
        
        // Try normal rotation first
        if self.valid_position(self.position.0, self.position.1, Some(to_rotation)) {
            self.current_tetrimino.rotate_right();
            self.needs_redraw = true;
            return true;
        }
        
        // Try wall kicks
        if let Some((new_x, new_y)) = self.try_wall_kick(from_rotation, to_rotation, self.position.0, self.position.1) {
            self.current_tetrimino.rotate_right();
            self.position = (new_x, new_y);
            self.needs_redraw = true;
            return true;
        }
        
        false
    }
    
    pub fn hold_current_piece(&mut self) -> bool {
        if self.game_over || self.paused || !self.can_hold {
            return false;
        }
        
        if let Some(hold_piece) = self.hold_tetrimino.take() {
            let temp = std::mem::replace(&mut self.current_tetrimino, hold_piece);
            self.hold_tetrimino = Some(temp);
        } else {
            self.hold_tetrimino = Some(self.current_tetrimino);
            self.current_tetrimino = std::mem::replace(&mut self.next_tetrimino, Tetrimino::random());
        }
        
        self.position = (4, 0);
        self.current_tetrimino.rotation = 0;
        self.can_hold = false;
        
        // Check if game over after hold
        if self.check_collision(self.position.0, self.position.1, None) {
            self.game_over = true;
        }
        
        self.needs_redraw = true;
        true
    }
    
    fn lock_current_piece(&mut self) -> bool {
        if self.game_over || self.paused {
            return false;
        }
        
        let lines_cleared = self.board.lock_tetrimino(&self.current_tetrimino, self.position.0, self.position.1);
        if lines_cleared > 0 {
            self.score.add_lines(lines_cleared);
        }
        
        self.spawn_new_piece();
        true
    }
    
    fn spawn_new_piece(&mut self) {
        self.current_tetrimino = std::mem::replace(&mut self.next_tetrimino, Tetrimino::random());
        self.position = (4, 0);
        self.current_tetrimino.rotation = 0;
        self.can_hold = true;
        self.last_update = Instant::now();
        
        // Check for game over
        if self.check_collision(self.position.0, self.position.1, None) {
            self.game_over = true;
        }
        
        self.needs_redraw = true;
    }
    
    pub fn update(&mut self) {
        if self.game_over || self.paused || self.board.is_line_clearing() {
            return;
        }
        
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.score.drop_interval() {
            if !self.soft_drop() {
                self.last_update = now;
            }
        }
    }
    
    pub fn reset(&mut self) -> Result<()> {
        *self = Self::new()?;
        Ok(())
    }
    
    pub fn toggle_pause(&mut self) {
        if !self.game_over {
            self.paused = !self.paused;
            self.needs_redraw = true;
        }
    }
    
    pub fn is_game_over(&self) -> bool {
        self.game_over
    }
    
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
    
    pub fn clear_redraw_flag(&mut self) {
        self.needs_redraw = false;
    }
    
    // Drawing methods
    pub fn draw(&self, display: &mut SharpDisplay) {
        self.draw_arena(display);
        self.draw_game_info(display);
        
        if self.game_over {
            self.draw_game_over(display);
        }
        
        if self.paused {
            self.draw_pause(display);
        }
    }
    
    fn draw_arena(&self, display: &mut SharpDisplay) {
        let border_left = ARENA_X.saturating_sub(2);
        let border_top = ARENA_Y.saturating_sub(2);
        let border_right = (ARENA_X + self.board.width() * BLOCK_SIZE + 1).min(399);
        let border_bottom = (ARENA_Y + self.board.height() * BLOCK_SIZE + 1).min(239);
        
        // Draw border
        for x in border_left..=border_right {
            display.draw_pixel(x, border_top, Pixel::Black);
            display.draw_pixel(x, border_bottom, Pixel::Black);
        }
        for y in border_top..=border_bottom {
            display.draw_pixel(border_left, y, Pixel::Black);
            display.draw_pixel(border_right, y, Pixel::Black);
        }
        
        // Draw placed blocks
        for y in 0..self.board.height() {
            for x in 0..self.board.width() {
                if let Some(_color) = self.board.get_cell(x, y) {
                    self.draw_block(display, x, y, false);
                }
            }
        }
        
        // Draw ghost piece
        if !self.game_over && !self.paused {
            let ghost_y = self.ghost_position();
            self.draw_tetrimino(display, self.position.0, ghost_y, &self.current_tetrimino, true);
        }
        
        // Draw current piece
        if !self.game_over && !self.paused {
            self.draw_tetrimino(display, self.position.0, self.position.1, &self.current_tetrimino, false);
        }
    }
    
    fn draw_game_info(&self, display: &mut SharpDisplay) {
        // Draw next piece
        self.draw_preview(display, NEXT_X, NEXT_Y, &self.next_tetrimino);
        
        // Draw hold piece
        if let Some(hold_piece) = &self.hold_tetrimino {
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
        
        // Draw score/level info
        let info_x = SCORE_X;
        let mut info_y = SCORE_Y;
        
        // Draw level indicator (bar)
        self.draw_score_bar(display, info_x, info_y, self.score.level().min(20), 20, 10);
        info_y += 20;
        
        // Draw lines indicator (bar)
        self.draw_score_bar(display, info_x, info_y, (self.score.lines_cleared() % 10) as u32, 10, 10);
    }
    
    fn ghost_position(&self) -> i32 {
        let mut ghost_y = self.position.1;
        while self.valid_position(self.position.0, ghost_y + 1, None) {
            ghost_y += 1;
        }
        ghost_y
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
    
    fn draw_tetrimino(&self, display: &mut SharpDisplay, x: i32, y: i32, tetrimino: &Tetrimino, is_ghost: bool) {
        let matrix = tetrimino.matrix();
        
        for py in 0..4 {
            for px in 0..4 {
                let index = py * 4 + px;
                if matrix[index] == 0 {
                    continue;
                }
                
                let block_x = (x + px as i32) as usize;
                let block_y = (y + py as i32) as usize;
                
                if block_x < self.board.width() && block_y < self.board.height() {
                    self.draw_block(display, block_x, block_y, is_ghost);
                }
            }
        }
    }
    
    fn draw_preview(&self, display: &mut SharpDisplay, x: usize, y: usize, tetrimino: &Tetrimino) {
        let preview_size = 8;
        let matrix = tetrimino.matrix();
        
        for py in 0..4 {
            for px in 0..4 {
                if matrix[py * 4 + px] == 0 {
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
    
    fn draw_game_over(&self, display: &mut SharpDisplay) {
        let center_x = ARENA_X + (self.board.width() * BLOCK_SIZE) / 2;
        let center_y = ARENA_Y + (self.board.height() * BLOCK_SIZE) / 2;
        
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
    
    fn draw_pause(&self, display: &mut SharpDisplay) {
        let center_x = ARENA_X + (self.board.width() * BLOCK_SIZE) / 2;
        let center_y = ARENA_Y + (self.board.height() * BLOCK_SIZE) / 2;
        
        // Draw pause symbol (two vertical bars)
        for y in 0..30 {
            let draw_y = center_y as i32 - 15 + y as i32;
            if draw_y >= 0 && draw_y < 240 {
                // Left bar
                for x in 0..3 {
                    let draw_x = center_x as i32 - 6 + x as i32;
                    if draw_x >= 0 && draw_x < 400 {
                        display.draw_pixel(draw_x as usize, draw_y as usize, Pixel::Black);
                    }
                }
                
                // Right bar
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