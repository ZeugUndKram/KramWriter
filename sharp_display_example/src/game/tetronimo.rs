use rand::Rng;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TetriminoType {
    I, O, S, Z, T, L, J
}

impl TetriminoType {
    pub fn as_index(&self) -> usize {
        match self {
            Self::I => 0,
            Self::O => 1,
            Self::S => 2,
            Self::Z => 3,
            Self::T => 4,
            Self::L => 5,
            Self::J => 6,
        }
    }
    
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..7) {
            0 => Self::I,
            1 => Self::O,
            2 => Self::S,
            3 => Self::Z,
            4 => Self::T,
            5 => Self::L,
            6 => Self::J,
            _ => Self::I,
        }
    }
}

impl fmt::Display for TetriminoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::I => "I",
            Self::O => "O",
            Self::S => "S",
            Self::Z => "Z",
            Self::T => "T",
            Self::L => "L",
            Self::J => "J",
        })
    }
}

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

#[derive(Clone, Copy)]
pub struct Tetrimino {
    pub tetrimino_type: TetriminoType,
    pub rotation: usize,
}

impl Tetrimino {
    pub fn new(tetrimino_type: TetriminoType) -> Self {
        Self {
            tetrimino_type,
            rotation: 0,
        }
    }
    
    pub fn random() -> Self {
        Self::new(TetriminoType::random())
    }
    
    pub fn matrix(&self) -> &[u8; 16] {
        &TETROMINOES[self.tetrimino_type.as_index()][self.rotation]
    }
    
    pub fn rotate_right(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }
    
    pub fn rotate_left(&mut self) {
        self.rotation = (self.rotation + 3) % 4; // +3 ≡ -1 mod 4
    }
    
    pub fn set_rotation(&mut self, rotation: usize) {
        self.rotation = rotation % 4;
    }
    
    pub fn get_matrix(&self, rotation: Option<usize>) -> &[u8; 16] {
        let rotation = rotation.unwrap_or(self.rotation);
        &TETROMINOES[self.tetrimino_type.as_index()][rotation]
    }
}