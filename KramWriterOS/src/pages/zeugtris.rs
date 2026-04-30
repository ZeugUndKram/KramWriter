use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use rand::seq::SliceRandom;
use std::collections::HashMap;

// --- Positioning Constants ---
const GRID_SIZE: usize = 10;
const GRID_HEIGHT: usize = 18;
const CELL_DIM: usize = 12;    
const OFFSET_X: usize = 140;   
const OFFSET_Y: usize = 12;    

const NEXT_X: usize = 286;     
const NEXT_Y: usize = 16;      
const NEXT_CELL_DIM: usize = 12; 

// --- Statistics Positioning (Aligned with grafik.png) ---
const STATS_X: i32 = 84;         
const STATS_START_Y: i32 = 35;   
const STATS_SPACING: i32 = 30;   
const STATS_FONT_SIZE: f32 = 24.0;

// --- Right Panel Info Positioning ---
const INFO_X: i32 = 286;
const INFO_SCORE_LBL_Y: i32 = 95;
const INFO_SCORE_VAL_Y: i32 = 120;
const INFO_LEVEL_Y: i32 = 175;
const INFO_LINES_Y: i32 = 205;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum TetrominoType { I, J, L, O, S, T, Z }

struct Piece {
    kind: TetrominoType,
    matrix: Vec<Vec<u8>>,
    row: i32,
    col: i32,
}

pub struct ZeugtrisPage {
    playfield: [[Option<TetrominoType>; GRID_SIZE]; GRID_HEIGHT],
    active_piece: Piece,
    next_piece: Piece,
    tick_count: u32,
    game_over: bool,
    backdrop: Option<Bitmap>,
    sprites: HashMap<TetrominoType, Bitmap>,
    renderer: FontRenderer,
    stats: HashMap<TetrominoType, u32>,
    
    // Marathon Mode variables
    score: u32,
    level: u32,
    lines: u32,
}

impl ZeugtrisPage {
    pub fn new() -> Self {
        let asset_path = "/home/kramwriter/KramWriter/assets/zeugtris/game";
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        
        let mut sprites = HashMap::new();
        let mut stats = HashMap::new();
        
        let types = [
            (TetrominoType::I, "zeugtris_i.bmp"),
            (TetrominoType::J, "zeugtris_j.bmp"), 
            (TetrominoType::L, "zeugtris_l.bmp"),
            (TetrominoType::O, "zeugtris_o.bmp"),
            (TetrominoType::S, "zeugtris_s.bmp"),
            (TetrominoType::T, "zeugtris_t.bmp"),
            (TetrominoType::Z, "zeugtris_z.bmp"),
        ];

        for (kind, filename) in types {
            if let Ok(bmp) = Bitmap::load(&format!("{}/{}", asset_path, filename)) {
                sprites.insert(kind, bmp);
            }
            stats.insert(kind, 0);
        }

        let first_piece = Self::spawn_piece();
        let next_piece = Self::spawn_piece();
        
        if let Some(count) = stats.get_mut(&first_piece.kind) {
            *count += 1;
        }

        Self {
            playfield: [[None; GRID_SIZE]; GRID_HEIGHT],
            active_piece: first_piece,
            next_piece,
            tick_count: 0,
            game_over: false,
            backdrop: Bitmap::load(&format!("{}/backdrop.bmp", asset_path)).ok(),
            sprites,
            renderer,
            stats,
            score: 0,
            level: 1,
            lines: 0,
        }
    }

    fn spawn_piece() -> Piece {
        let kinds = [
            TetrominoType::I, TetrominoType::J, TetrominoType::L, 
            TetrominoType::O, TetrominoType::S, TetrominoType::T, TetrominoType::Z
        ];
        let kind = *kinds.choose(&mut rand::thread_rng()).unwrap();
        
        let matrix = match kind {
            TetrominoType::I => vec![vec![0,0,0,0], vec![1,1,1,1], vec![0,0,0,0], vec![0,0,0,0]],
            TetrominoType::J => vec![vec![1,0,0], vec![1,1,1], vec![0,0,0]],
            TetrominoType::L => vec![vec![0,0,1], vec![1,1,1], vec![0,0,0]],
            TetrominoType::O => vec![vec![1,1], vec![1,1]],
            TetrominoType::S => vec![vec![0,1,1], vec![1,1,0], vec![0,0,0]],
            TetrominoType::Z => vec![vec![1,1,0], vec![0,1,1], vec![0,0,0]],
            TetrominoType::T => vec![vec![0,1,0], vec![1,1,1], vec![0,0,0]],
        };

        Piece { kind, matrix, row: -2, col: 3 }
    }

    fn rotate(matrix: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        let n = matrix.len();
        let mut result = vec![vec![0; n]; n];
        for i in 0..n {
            for j in 0..n {
                result[j][n - 1 - i] = matrix[i][j];
            }
        }
        result
    }

    fn is_valid_move(&self, matrix: &Vec<Vec<u8>>, r: i32, c: i32) -> bool {
        for (y, row) in matrix.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell != 0 {
                    let new_r = r + y as i32;
                    let new_c = c + x as i32;
                    if new_c < 0 || new_c >= GRID_SIZE as i32 || new_r >= GRID_HEIGHT as i32 {
                        return false;
                    }
                    if new_r >= 0 && self.playfield[new_r as usize][new_c as usize].is_some() {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn place_piece(&mut self) {
        for (y, row) in self.active_piece.matrix.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell != 0 {
                    let r = self.active_piece.row + y as i32;
                    let c = self.active_piece.col + x as i32;
                    if r < 0 { self.game_over = true; return; }
                    self.playfield[r as usize][c as usize] = Some(self.active_piece.kind);
                }
            }
        }
        
        self.clear_lines();
        self.active_piece = std::mem::replace(&mut self.next_piece, Self::spawn_piece());
        
        if let Some(count) = self.stats.get_mut(&self.active_piece.kind) {
            *count += 1;
        }
    }

    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;
        let mut y = GRID_HEIGHT as i32 - 1;

        while y >= 0 {
            let uy = y as usize;
            if self.playfield[uy].iter().all(|cell| cell.is_some()) {
                // Clear and shift down
                for move_y in (1..=uy).rev() {
                    self.playfield[move_y] = self.playfield[move_y - 1];
                }
                self.playfield[0] = [None; GRID_SIZE];
                lines_cleared += 1;
                // DO NOT decrement y here, we need to check the newly shifted row
            } else {
                y -= 1;
            }
        }

        // Apply scoring and level rules based on Guideline
        if lines_cleared > 0 {
            let base_points = match lines_cleared {
                1 => 100,
                2 => 300,
                3 => 500,
                4 => 800,
                _ => 0,
            };
            self.score += base_points * self.level;
            self.lines += lines_cleared;
            
            // Fixed-goal level advancement (10 lines per level)
            self.level = (self.lines / 10) + 1;
        }
    }

    // Convert Guideline G-values (gravity) into frame delays for 60fps tick rate
    fn get_drop_delay(&self) -> u32 {
        let delays = [
            60, // Lvl 1 (0.01667G)
            48, // Lvl 2 (0.021G)
            37, // Lvl 3 (0.026G)
            28, // Lvl 4 (0.035G)
            21, // Lvl 5 (0.046G)
            16, // Lvl 6 (0.063G)
            11, // Lvl 7 (0.087G)
             8, // Lvl 8 (0.123G)
             6, // Lvl 9 (0.177G)
             4, // Lvl 10 (0.259G)
             3, // Lvl 11 (0.388G)
             2, // Lvl 12 (0.591G)
             1, // Lvl 13 (0.92G)
             1, // Lvl 14 (1.46G)
             0  // Lvl 15+ (2.36G+, instant drop/20G)
        ];
        
        // Cap index at 14 (Level 15 rules)
        let idx = (self.level.saturating_sub(1) as usize).min(14);
        delays[idx]
    }

    fn draw_block(&self, display: &mut SharpDisplay, kind: TetrominoType, grid_x: usize, grid_y: usize, ctx: &Context) {
        if let Some(bmp) = self.sprites.get(&kind) {
            let screen_x = OFFSET_X + (grid_x * CELL_DIM);
            let screen_y = OFFSET_Y + (grid_y * CELL_DIM);
            for y in 0..(bmp.height as usize).min(CELL_DIM) {
                for x in 0..(bmp.width as usize).min(CELL_DIM) {
                    let pixel = bmp.pixels[y * bmp.width as usize + x];
                    if pixel == Pixel::Black {
                        display.draw_pixel(screen_x + x, screen_y + y, Pixel::Black, ctx);
                    }
                }
            }
        }
    }

    fn draw_statistics(&self, display: &mut SharpDisplay, ctx: &Context) {
        let order = [
            TetrominoType::J, 
            TetrominoType::L, 
            TetrominoType::Z, 
            TetrominoType::S, 
            TetrominoType::T,
            TetrominoType::I,
            TetrominoType::O,
        ];

        for (i, kind) in order.iter().enumerate() {
            let count = self.stats.get(kind).unwrap_or(&0);
            let text = format!("{:03}", count); 
            let y = STATS_START_Y + (i as i32 * STATS_SPACING);
            
            self.renderer.draw_text(display, &text, STATS_X, y, STATS_FONT_SIZE, ctx);
        }
    }

    fn draw_game_info(&self, display: &mut SharpDisplay, ctx: &Context) {
        // Draw SCORE
        self.renderer.draw_text(display, "SCORE:", INFO_X, INFO_SCORE_LBL_Y, STATS_FONT_SIZE, ctx);
        let score_text = format!("{}", self.score);
        self.renderer.draw_text(display, &score_text, INFO_X, INFO_SCORE_VAL_Y, STATS_FONT_SIZE, ctx);

        // Draw LEVEL
        let level_text = format!("LEVEL: {}", self.level);
        self.renderer.draw_text(display, &level_text, INFO_X, INFO_LEVEL_Y, STATS_FONT_SIZE, ctx);

        // Draw LINES
        let lines_text = format!("LINES: {}", self.lines);
        self.renderer.draw_text(display, &lines_text, INFO_X, INFO_LINES_Y, STATS_FONT_SIZE, ctx);
    }
}

impl Page for ZeugtrisPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        if self.game_over {
            return if key == Key::Esc { Action::Pop } else { Action::None };
        }
        match key {
            Key::Left => if self.is_valid_move(&self.active_piece.matrix, self.active_piece.row, self.active_piece.col - 1) { self.active_piece.col -= 1; }
            Key::Right => if self.is_valid_move(&self.active_piece.matrix, self.active_piece.row, self.active_piece.col + 1) { self.active_piece.col += 1; }
            Key::Up => {
                let rotated = Self::rotate(&self.active_piece.matrix);
                if self.is_valid_move(&rotated, self.active_piece.row, self.active_piece.col) { self.active_piece.matrix = rotated; }
            }
            Key::Down => {
                if self.is_valid_move(&self.active_piece.matrix, self.active_piece.row + 1, self.active_piece.col) { 
                    self.active_piece.row += 1; 
                    self.score += 1; // Soft drop gives 1 point per cell
                }
            }
            Key::Esc => return Action::Pop,
            _ => {}
        }
        Action::Redraw
    }

    fn tick(&mut self, _ctx: &mut Context) -> Action {
        if self.game_over { return Action::None; }
        
        self.tick_count += 1;
        
        let drop_delay = self.get_drop_delay();
        
        if self.tick_count >= drop_delay { 
            self.tick_count = 0;
            if self.is_valid_move(&self.active_piece.matrix, self.active_piece.row + 1, self.active_piece.col) {
                self.active_piece.row += 1;
                return Action::Redraw;
            } else {
                self.place_piece();
                return Action::Redraw;
            }
        }
        Action::None
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        display.clear(ctx);

        if let Some(bmp) = &self.backdrop {
            for y in 0..(bmp.height as usize).min(240) {
                for x in 0..(bmp.width as usize).min(400) {
                    if bmp.pixels[y * bmp.width as usize + x] == Pixel::Black {
                        display.draw_pixel(x, y, Pixel::Black, ctx);
                    }
                }
            }
        }

        for r in 0..GRID_HEIGHT {
            for c in 0..GRID_SIZE {
                if let Some(kind) = self.playfield[r][c] {
                    self.draw_block(display, kind, c, r, ctx);
                }
            }
        }

        for (y, row) in self.active_piece.matrix.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell != 0 {
                    let r = self.active_piece.row + y as i32;
                    let c = self.active_piece.col + x as i32;
                    if r >= 0 && r < GRID_HEIGHT as i32 {
                        self.draw_block(display, self.active_piece.kind, c as usize, r as usize, ctx);
                    }
                }
            }
        }

        if let Some(bmp) = self.sprites.get(&self.next_piece.kind) {
            for (y, row) in self.next_piece.matrix.iter().enumerate() {
                for (x, &cell) in row.iter().enumerate() {
                    if cell != 0 {
                        let screen_x = NEXT_X + (x * NEXT_CELL_DIM);
                        let screen_y = NEXT_Y + (y * NEXT_CELL_DIM);
                        for py in 0..(bmp.height as usize).min(NEXT_CELL_DIM) {
                            for px in 0..(bmp.width as usize).min(NEXT_CELL_DIM) {
                                let pixel = bmp.pixels[py * bmp.width as usize + px];
                                if pixel == Pixel::Black {
                                    display.draw_pixel(screen_x + px, screen_y + py, Pixel::Black, ctx);
                                }
                            }
                        }
                    }
                }
            }
        }

        self.draw_statistics(display, ctx);
        self.draw_game_info(display, ctx); // Draws Score, Level, and Lines
    }
}