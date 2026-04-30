use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;
use rand::seq::SliceRandom;

// Constants updated for 12x12 blocks
const GRID_SIZE: usize = 10;
const GRID_HEIGHT: usize = 20;
const CELL_DIM: usize = 12;    
const OFFSET_X: usize = 140;   // Adjusted centering for 120px board
const OFFSET_Y: usize = 0;

#[derive(Clone, Copy, PartialEq)]
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
    tick_count: u32,
    game_over: bool,
    // Graphics assets
    backdrop: Option<Bitmap>,
    sprites: std::collections::HashMap<TetrominoType, Bitmap>,
}

impl ZeugtrisPage {
    pub fn new() -> Self {
        let asset_path = "/home/kramwriter/KramWriter/assets/zeugtris/game";
        
        // Load block sprites for each tetromino type
        let mut sprites = std::collections::HashMap::new();
        let types = [
            (TetrominoType::I, "zeugtris_i.bmp"),
            (TetrominoType::J, "zeugtris_j.bmp"), // Fixed typo from 'zeutris_j'
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
        }

        Self {
            playfield: [[None; GRID_SIZE]; GRID_HEIGHT],
            active_piece: Self::spawn_piece(),
            tick_count: 0,
            game_over: false,
            backdrop: Bitmap::load(&format!("{}/backdrop.bmp", asset_path)).ok(),
            sprites,
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
        self.active_piece = Self::spawn_piece();
    }

    fn clear_lines(&mut self) {
        for y in (0..GRID_HEIGHT).rev() {
            if self.playfield[y].iter().all(|cell| cell.is_some()) {
                for move_y in (1..=y).rev() {
                    self.playfield[move_y] = self.playfield[move_y - 1];
                }
                self.playfield[0] = [None; GRID_SIZE];
                self.clear_lines();
                break;
            }
        }
    }

    fn draw_block(&self, display: &mut SharpDisplay, kind: TetrominoType, grid_x: usize, grid_y: usize, ctx: &Context) {
        if let Some(bmp) = self.sprites.get(&kind) {
            let screen_x = OFFSET_X + (grid_x * CELL_DIM);
            let screen_y = OFFSET_Y + (grid_y * CELL_DIM);
            
            for y in 0..bmp.height.min(CELL_DIM as u32) {
                for x in 0..bmp.width.min(CELL_DIM as u32) {
                    if bmp.pixels[(y * bmp.width + x) as usize] == Pixel::Black {
                        display.draw_pixel(screen_x + x as usize, screen_y + y as usize, Pixel::Black, ctx);
                    }
                }
            }
        }
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
            Key::Down => if self.is_valid_move(&self.active_piece.matrix, self.active_piece.row + 1, self.active_piece.col) { self.active_piece.row += 1; }
            Key::Esc => return Action::Pop,
            _ => {}
        }
        Action::Redraw
    }

    fn tick(&mut self, _ctx: &mut Context) -> Action {
        if self.game_over { return Action::None; }
        self.tick_count += 1;
        if self.tick_count > 5 { // Set to your preferred 5-tick speed
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

        // 1. Always draw Backdrop first
        if let Some(bmp) = &self.backdrop {
            for y in 0..bmp.height.min(240) {
                for x in 0..bmp.width.min(400) {
                    if bmp.pixels[(y * bmp.width + x) as usize] == Pixel::Black {
                        display.draw_pixel(x as usize, y as usize, Pixel::Black, ctx);
                    }
                }
            }
        }

        // 2. Draw settled blocks using sprites
        for r in 0..GRID_HEIGHT {
            for c in 0..GRID_SIZE {
                if let Some(kind) = self.playfield[r][c] {
                    self.draw_block(display, kind, c, r, ctx);
                }
            }
        }

        // 3. Draw active piece using sprites
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
    }
}