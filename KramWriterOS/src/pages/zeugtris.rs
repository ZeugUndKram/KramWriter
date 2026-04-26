use super::{Action, Page};
use crate::display::SharpDisplay;
use crate::context::Context;
use crate::game::TetrisGame;
use termion::event::Key;

pub struct ZeugtrisPage {
    game: TetrisGame,
}

impl ZeugtrisPage {
    pub fn new() -> Self {
        Self {
            // Using expect here or changing the return type of new to Result<Self>
            game: TetrisGame::new().expect("Failed to initialize Zeugtris"),
        }
    }
}

impl Page for ZeugtrisPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        // 1. Handle Game Over State
        if self.game.is_game_over() {
            match key {
                Key::Esc => return Action::Pop,
                Key::Char('r') | Key::Char('R') => {
                    let _ = self.game.reset();
                }
                _ => {}
            }
            return Action::None;
        }
        
        // 2. Handle Global Commands (Pause)
        if key == Key::Char('p') || key == Key::Char('P') {
            self.game.toggle_pause();
            return Action::None;
        }
        
        // 3. Handle Paused State
        if self.game.is_paused() {
            if key == Key::Esc { return Action::Pop; }
            return Action::None;
        }
        
        // 4. Active Game Controls
        match key {
            Key::Char('z') | Key::Char('Z') => { self.game.rotate_left(); }
            Key::Char('x') | Key::Char('X') => { self.game.rotate_right(); }
            Key::Left => { self.game.move_left(); }
            Key::Right => { self.game.move_right(); }
            Key::Down => { self.game.soft_drop(); }
            Key::Up | Key::Char(' ') => { self.game.hard_drop(); }
            Key::Esc => return Action::Pop,
            Key::Char('r') | Key::Char('R') => { let _ = self.game.reset(); }
            _ => {}
        }
        
        Action::None
    }

    fn tick(&mut self, _ctx: &mut Context) -> bool {
        // This is called by your main loop at ~60fps.
        // The game internal logic handles gravity and timings.
        self.game.update(); 
        
        let redraw = self.game.needs_redraw();
        
        if redraw {
            self.game.clear_redraw_flag();
        }
        
        redraw
    }

    fn draw(&mut self, display: &mut SharpDisplay, ctx: &Context) {
        // Draw the game state. 
        // Note: The clearing of the display is now handled by App::render()
        self.game.draw(display, ctx);
    }
}