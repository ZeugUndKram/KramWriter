use super::tetrimino::Tetrimino;

const ARENA_WIDTH: usize = 10;
const ARENA_HEIGHT: usize = 18;

pub struct Board {
    grid: [[Option<u8>; ARENA_WIDTH]; ARENA_HEIGHT],
    lines_to_clear: Vec<usize>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            grid: [[None; ARENA_WIDTH]; ARENA_HEIGHT],
            lines_to_clear: Vec::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.grid = [[None; ARENA_WIDTH]; ARENA_HEIGHT];
        self.lines_to_clear.clear();
    }
    
    pub fn check_collision(
        &self,
        tetrimino: &Tetrimino,
        x: i32,
        y: i32,
        rotation: Option<usize>
    ) -> bool {
        let matrix = tetrimino.get_matrix(rotation);
        
        for py in 0..4 {
            for px in 0..4 {
                let index = py * 4 + px;
                if matrix[index] == 0 {
                    continue;
                }
                
                let arena_x = x + px as i32;
                let arena_y = y + py as i32;
                
                if arena_x < 0 || arena_x >= ARENA_WIDTH as i32 || arena_y >= ARENA_HEIGHT as i32 {
                    return true;
                }
                
                if arena_y >= 0 && self.grid[arena_y as usize][arena_x as usize].is_some() {
                    return true;
                }
            }
        }
        false
    }
    
    pub fn lock_tetrimino(&mut self, tetrimino: &Tetrimino, x: i32, y: i32) -> usize {
        let color_index = tetrimino.tetrimino_type.as_index() as u8 + 1;
        let matrix = tetrimino.matrix();
        
        for py in 0..4 {
            for px in 0..4 {
                let index = py * 4 + px;
                if matrix[index] == 0 {
                    continue;
                }
                
                let arena_x = (x + px as i32) as usize;
                let arena_y = (y + py as i32) as usize;
                
                if arena_x < ARENA_WIDTH && arena_y < ARENA_HEIGHT {
                    self.grid[arena_y][arena_x] = Some(color_index);
                }
            }
        }
        
        self.check_lines()
    }
    
    fn check_lines(&mut self) -> usize {
        let mut lines_cleared = 0;
        let mut new_grid = [[None; ARENA_WIDTH]; ARENA_HEIGHT];
        let mut new_row = ARENA_HEIGHT - 1;
        
        self.lines_to_clear.clear();
        
        for row in (0..ARENA_HEIGHT).rev() {
            let mut line_full = true;
            for x in 0..ARENA_WIDTH {
                if self.grid[row][x].is_none() {
                    line_full = false;
                    break;
                }
            }
            
            if !line_full {
                new_grid[new_row] = self.grid[row];
                new_row -= 1;
            } else {
                lines_cleared += 1;
                self.lines_to_clear.push(row);
            }
        }
        
        self.grid = new_grid;
        lines_cleared
    }
    
    pub fn get_cell(&self, x: usize, y: usize) -> Option<u8> {
        if x < ARENA_WIDTH && y < ARENA_HEIGHT {
            self.grid[y][x]
        } else {
            None
        }
    }
    
    pub fn width(&self) -> usize {
        ARENA_WIDTH
    }
    
    pub fn height(&self) -> usize {
        ARENA_HEIGHT
    }
    
    pub fn lines_to_clear(&self) -> &Vec<usize> {
        &self.lines_to_clear
    }
    
    pub fn is_line_clearing(&self) -> bool {
        !self.lines_to_clear.is_empty()
    }
}