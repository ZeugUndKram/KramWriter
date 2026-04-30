use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use termion::event::Key;
use rpi_memory_display::Pixel;
use rand::seq::SliceRandom;

// Constants for the 400x240 Sharp Display
const GRID_SIZE: usize = 10;
const GRID_HEIGHT: usize = 20;
const CELL_DIM: usize = 11;    // Size of block + 1px border for grid effect
const OFFSET_X: usize = 145;   // Centering the 110px wide board
const OFFSET_Y: usize = 10;

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
}

impl ZeugtrisPage {
    pub fn new() -> Self {
        Self {
            playfield: [[None; GRID_SIZE]; GRID_HEIGHT],
            active_piece: Self::spawn_piece(),
            tick_count: 0,
            game_over: false,
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

        Piece {
            kind,
            matrix,
            row: -2, 
            col: 3,
        }
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
                    if r < 0 {
                        self.game_over = true;
                        return;
                    }
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
                self.clear_lines(); // Check same row again after shift
                break;
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
            Key::Left => {
                if self.is_valid_move(&self.active_piece.matrix, self.active_piece.row, self.active_piece.col - 1) {
                    self.active_piece.col -= 1;
                }
            }
            Key::Right => {
                if self.is_valid_move(&self.active_piece.matrix, self.active_piece.row, self.active_piece.col + 1) {
                    self.active_piece.col += 1;
                }
            }
            Key::Up => {
                let rotated = Self::rotate(&self.active_piece.matrix);
                if self.is_valid_move(&rotated, self.active_piece.row, self.active_piece.col) {
                    self.active_piece.matrix = rotated;
                }
            }
            Key::Down => {
                if self.is_valid_move(&self.active_piece.matrix, self.active_piece.row + 1, self.active_piece.col) {
                    self.active_piece.row += 1;
                }
            }
            Key::Esc => return Action::Pop,
            _ => {}
        }
        Action::Redraw
    }

    fn tick(&mut self, _ctx: &mut Context) -> Action {
        if self.game_over {
            return Action::None;
        }

        self.tick_count += 1;
        // Adjust this value to change falling speed (e.g., 10 for faster, 30 for slower)
        if self.tick_count > 20 { 
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

        // Draw side borders
        for y in 0..(GRID_HEIGHT * CELL_DIM) {
            display.draw_pixel(OFFSET_X - 1, OFFSET_Y + y, Pixel::Black, ctx);
            display.draw_pixel(OFFSET_X + (GRID_SIZE * CELL_DIM), OFFSET_Y + y, Pixel::Black, ctx);
        }

        // Draw settled blocks
        for r in 0..GRID_HEIGHT {
            for c in 0..GRID_SIZE {
                if self.playfield[r][c].is_some() {
                    for py in 0..CELL_DIM-1 {
                        for px in 0..CELL_DIM-1 {
                            display.draw_pixel(OFFSET_X + c * CELL_DIM + px, OFFSET_Y + r * CELL_DIM + py, Pixel::Black, ctx);
                        }
                    }
                }
            }
        }

        // Draw active piece
        for (y, row) in self.active_piece.matrix.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell != 0 {
                    let r = self.active_piece.row + y as i32;
                    let c = self.active_piece.col + x as i32;
                    if r >= 0 && r < GRID_HEIGHT as i32 {
                        for py in 0..CELL_DIM-1 {
                            for px in 0..CELL_DIM-1 {
                                display.draw_pixel(OFFSET_X + c as usize * CELL_DIM + px, OFFSET_Y + r as usize * CELL_DIM + py, Pixel::Black, ctx);
                            }
                        }
                    }
                }
            }
        }

        if self.game_over {
            // "Game Over" logic could be added here (e.g., drawing a Bitmap)
        }
    }
}