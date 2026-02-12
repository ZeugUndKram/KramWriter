use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use termion::event::Key;

pub struct MenuPage {
    selected: usize,
}

impl MenuPage {
    pub fn new() -> Self {
        Self { selected: 0 }
    }
}

impl Page for MenuPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Char('q') => Action::Exit,
            Key::Esc => Action::Pop, // Goes back to Logo
            Key::Up => { if self.selected > 0 { self.selected -= 1; } Action::None },
            Key::Down => { if self.selected < 4 { self.selected += 1; } Action::None },
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        display.draw_text(10, 10, "MAIN MENU", ctx);
        let options = ["Write", "Learn", "Zeugtris", "Settings", "Credits"];
        
        for (i, opt) in options.iter().enumerate() {
            let prefix = if i == self.selected { "> " } else { "  " };
            display.draw_text(20, 40 + (i * 20), &format!("{}{}", prefix, opt), ctx);
        }
    }
}