use super::{board::Board, score::Score, tetrimino::{Tetrimino, TetriminoType}, sprites::BlockSprites};
use crate::display::SharpDisplay;
use anyhow::Result;
use rpi_memory_display::Pixel;
use std::time::Instant;
use std::fs;

// Game constants
const BLOCK_SIZE: usize = 12;
const ARENA_X: usize = 150;
const ARENA_Y: usize = 12;
const NEXT_X: usize = 300;
const NEXT_Y: usize = 30;
const HOLD_X: usize = 300;
const HOLD_Y: usize = 100;
const SCORE_X: usize = 300;
const SCORE_Y: usize = 160;
const OVERLAY_X: usize = 0;
const OVERLAY_Y: usize = 0;

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
    overlay_data: Option<Vec<Pixel>>,
    overlay_width: usize,
    overlay_height: usize,
    sprites: BlockSprites,
}

impl TetrisGame {
    pub fn new() -> Result<Self> {
        // Load block sprites
        let sprites = match BlockSprites::new() {
            Ok(sprites) => {
                if sprites.has_sprites() {
                    println!("Successfully loaded some Tetris sprites");
                } else {
                    println!("Warning: No Tetris sprites were loaded");
                }
                sprites
            }
            Err(e) => {
                println!("Failed to load sprites: {}", e);
                BlockSprites {
                    i_sprite: None,
                    o_sprite: None,
                    s_sprite: None,
                    z_sprite: None,
                    t_sprite: None,
                    l_sprite: None,
                    j_sprite: None,
                    sprite_width: 12,
                    sprite_height: 12,
                }
            }
        };
        
        // Load overlay bitmap
        let overlay_path = "/home/kramwriter/KramWriter/assets/zeugtris/zeugtris_overlay.bmp";
        println!("Loading overlay from: {}", overlay_path);
        
        let (overlay_data, overlay_width, overlay_height) = match fs::read(overlay_path) {
            Ok(data) => {
                println!("Loaded overlay: {} bytes", data.len());
                match Self::parse_bmp(&data) {
                    Some((pixels, width, height)) => {
                        println!("Parsed overlay BMP: {}x{}, {} pixels", width, height, pixels.len());
                        (Some(pixels), width, height)
                    }
                    None => {
                        println!("Failed to parse overlay BMP");
                        (None, 0, 0)
                    }
                }
            }
            Err(e) => {
                println!("Failed to read overlay: {}", e);
                (None, 0, 0)
            }
        };
        
        let mut game = Self {
            board: Board::new(),
            score: Score::new(),
            current_tetrimino: Tetrimino::random(),
            next_tetrimino: Tetrimino::random(),
            hold_tetrimino: None,
            can_hold: true,
            position: (4, 0),
            game_over: false,
            paused: false,
            last_update: Instant::now(),
            needs_redraw: true,
            overlay_data,
            overlay_width,
            overlay_height,
            sprites,
        };
        
        // Check initial position
        if game.check_collision(game.position.0, game.position.1, None) {
            game.game_over = true;
        }
        
        Ok(game)
    }
    
    fn parse_bmp(data: &[u8]) -> Option<(Vec<Pixel>, usize, usize)> {
        if data.len() < 54 { return None; }
        if data[0] != 0x42 || data[1] != 0x4D { return None; }
        
        let width = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        let height = u32::from_le_bytes([data[22], data[23], data[24], data[25]]) as usize;
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]) as usize;
        let data_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        
        println!("Overlay BMP: {}x{}, {} bpp, offset: {}", width, height, bits_per_pixel, data_offset);
        
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
        // Draw overlay first (as background)
        self.draw_overlay(display);
        
        // Then draw game elements on top
        self.draw_arena(display);
        self.draw_game_info(display);
        
        if self.game_over {
            self.draw_game_over(display);
        }
        
        if self.paused {
            self.draw_pause(display);
        }
    }
    
    fn draw_overlay(&self, display: &mut SharpDisplay) {
        if let Some(overlay_pixels) = &self.overlay_data {
            // Draw overlay at specified position
            for y in 0..self.overlay_height.min(240 - OVERLAY_Y) {
                for x in 0..self.overlay_width.min(400 - OVERLAY_X) {
                    let pixel = overlay_pixels[y * self.overlay_width + x];
                    // Only draw black pixels (skip white/transparent)
                    if pixel == Pixel::Black {
                        let screen_x = OVERLAY_X + x;
                        let screen_y = OVERLAY_Y + y;
                        if screen_x < 400 && screen_y < 240 {
                            display.draw_pixel(screen_x, screen_y, pixel);
                        }
                    }
                }
            }
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
        
        // Draw placed blocks using sprites
        for y in 0..self.board.height() {
            for x in 0..self.board.width() {
                if let Some(color_index) = self.board.get_cell(x, y) {
                    // color_index is 1-7, convert to 0-6 for sprite lookup
                    let piece_type = (color_index as usize).saturating_sub(1);
                    self.draw_block(display, x, y, piece_type, false);
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
        // Draw next piece using sprite
        self.draw_preview(display, NEXT_X, NEXT_Y, &self.next_tetrimino);
        
        // Draw hold piece using sprite
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
    
    fn draw_block(&self, display: &mut SharpDisplay, x: usize, y: usize, piece_type: usize, is_ghost: bool) {
        let block_x = ARENA_X + x * BLOCK_SIZE;
        let block_y = ARENA_Y + y * BLOCK_SIZE;
        
        if is_ghost {
            // Draw ghost piece (checkerboard pattern)
            for by in 0..BLOCK_SIZE {
                for bx in 0..BLOCK_SIZE {
                    // Create a visible checkerboard pattern
                    let check_size = 3;
                    let check_x = bx / check_size;
                    let check_y = by / check_size;
                    
                    if (check_x + check_y) % 2 == 0 && block_x + bx < 400 && block_y + by < 240 {
                        display.draw_pixel(block_x + bx, block_y + by, Pixel::Black);
                    }
                }
            }
        } else {
            // Try to draw sprite if available
            if let Some(sprite_pixels) = self.sprites.get_sprite(piece_type) {
                // Draw the sprite
                for sy in 0..self.sprites.sprite_height {
                    for sx in 0..self.sprites.sprite_width {
                        let pixel = sprite_pixels[sy * self.sprites.sprite_width + sx];
                        if pixel == Pixel::Black {
                            let screen_x = block_x + sx;
                            let screen_y = block_y + sy;
                            if screen_x < 400 && screen_y < 240 {
                                display.draw_pixel(screen_x, screen_y, Pixel::Black);
                            }
                        }
                    }
                }
            } else {
                // Fallback: draw solid block if sprite not available
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
    
    fn draw_tetrimino(&self, display: &mut SharpDisplay, x: i32, y: i32, tetrimino: &Tetrimino, is_ghost: bool) {
        let piece_type = tetrimino.tetrimino_type.as_index();
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
                    self.draw_block(display, block_x, block_y, piece_type, is_ghost);
                }
            }
        }
    }
    
    fn draw_preview(&self, display: &mut SharpDisplay, x: usize, y: usize, tetrimino: &Tetrimino) {
        let piece_type = tetrimino.tetrimino_type.as_index();
        let preview_size = 10;
        let matrix = tetrimino.matrix();
        
        // Try to draw sprite if available
        if let Some(sprite_pixels) = self.sprites.get_sprite(piece_type) {
            for py in 0..4 {
                for px in 0..4 {
                    if matrix[py * 4 + px] == 0 {
                        continue;
                    }
                    
                    let screen_x = x + px * (preview_size + 2);
                    let screen_y = y + py * (preview_size + 2);
                    
                    // Draw scaled down sprite (10x10 instead of 12x12)
                    for sy in 0..preview_size {
                        for sx in 0..preview_size {
                            // Map preview coordinates to sprite coordinates (simple scaling)
                            let sprite_x = (sx * self.sprites.sprite_width) / preview_size;
                            let sprite_y = (sy * self.sprites.sprite_height) / preview_size;
                            
                            if sprite_x < self.sprites.sprite_width && sprite_y < self.sprites.sprite_height {
                                let pixel = sprite_pixels[sprite_y * self.sprites.sprite_width + sprite_x];
                                if pixel == Pixel::Black {
                                    let draw_x = screen_x + sx;
                                    let draw_y = screen_y + sy;
                                    if draw_x < 400 && draw_y < 240 {
                                        display.draw_pixel(draw_x, draw_y, Pixel::Black);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Fallback: draw solid preview
            for py in 0..4 {
                for px in 0..4 {
                    if matrix[py * 4 + px] == 0 {
                        continue;
                    }
                    
                    let screen_x = x + px * (preview_size + 2);
                    let screen_y = y + py * (preview_size + 2);
                    
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