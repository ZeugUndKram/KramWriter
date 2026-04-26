use std::time::Duration;

pub struct Score {
    score: u32,
    lines_cleared: u32,
    level: u32,
    drop_interval: Duration,
}

impl Score {
    pub fn new() -> Self {
        Self {
            score: 0,
            lines_cleared: 0,
            level: 1,
            drop_interval: Duration::from_millis(1000),
        }
    }
    
    pub fn add_lines(&mut self, lines: usize) {
        let line_points = match lines {
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200,
            _ => 0,
        };
        
        self.lines_cleared += lines as u32;
        self.score += line_points * (self.level + 1);
        self.level = (self.lines_cleared / 10) + 1;
        
        // Update drop speed based on level
        let drop_ms = (1000.0 * (0.8_f32).powf((self.level - 1) as f32)).max(50.0) as u64;
        self.drop_interval = Duration::from_millis(drop_ms);
    }
    
    pub fn add_soft_drop_points(&mut self, lines: u32) {
        self.score += lines;
    }
    
    pub fn add_hard_drop_points(&mut self, lines: u32) {
        self.score += lines * 2;
    }
    
    pub fn score(&self) -> u32 {
        self.score
    }
    
    pub fn level(&self) -> u32 {
        self.level
    }
    
    pub fn lines_cleared(&self) -> u32 {
        self.lines_cleared
    }
    
    pub fn drop_interval(&self) -> Duration {
        self.drop_interval
    }
    
    pub fn reset(&mut self) {
        self.score = 0;
        self.lines_cleared = 0;
        self.level = 1;
        self.drop_interval = Duration::from_millis(1000);
    }
}