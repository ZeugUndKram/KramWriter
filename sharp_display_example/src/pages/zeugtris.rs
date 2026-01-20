use super::{Page, PageId};
use crate::display::SharpDisplay;
use crate::game::TetrisGame;
use anyhow::Result;
use termion::event::Key;

pub struct ZeugtrisPage {
    game: TetrisGame,
    last_frame_time: std::time::Instant,
    frame_count: u32,
}

impl ZeugtrisPage {
    pub fn new() -> Result<Self> {
        Ok(Self {
            game: TetrisGame::new()?,
            last_frame_time: std::time::Instant::now(),
            frame_count: 0,
        })
    }
}

impl Page for ZeugtrisPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        self.game.update();
        
        let now = std::time::Instant::now();
        let time_since_last_frame = now.duration_since(self.last_frame_time);
        self.frame_count += 1;
        
        let force_redraw = self.frame_count % 8 < 4;
        
        if self.game.needs_redraw() || force_redraw || time_since_last_frame >= std::time::Duration::from_millis(16) {
            display.clear()?;
            self.game.draw(display);
            display.update()?;
            
            self.game.clear_redraw_flag();
            self.last_frame_time = now;
        }
        
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        if self.game.is_game_over() {
            match key {
                Key::Esc => return Ok(Some(PageId::ZeugtrisMenu)),
                Key::Char('r') | Key::Char('R') => {
                    self.game.reset()?;
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }
        
        if key == Key::Char('p') || key == Key::Char('P') {
            self.game.toggle_pause();
            return Ok(None);
        }
        
        if self.game.is_paused() {
            match key {
                Key::Esc => return Ok(Some(PageId::ZeugtrisMenu)),
                Key::Char('p') | Key::Char('P') => {
                    self.game.toggle_pause();
                }
                _ => {}
            }
            return Ok(None);
        }
        
        match key {
            Key::Char('z') | Key::Char('Z') => {
                self.game.rotate_left();
            }
            Key::Char('x') | Key::Char('X') => {
                self.game.rotate_right();
            }
            Key::Left => {
                self.game.move_left();
            }
            Key::Right => {
                self.game.move_right();
            }
            Key::Down => {
                self.game.soft_drop();
            }
            Key::Up | Key::Char(' ') => {
                self.game.hard_drop();
            }
            Key::Char('c') | Key::Char('C') => {
                self.game.hold_current_piece();
            }
            Key::Esc => return Ok(Some(PageId::ZeugtrisMenu)),
            Key::Char('r') | Key::Char('R') => {
                self.game.reset()?;
            }
            _ => {}
        }
        
        Ok(None)
    }
}