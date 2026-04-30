use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use termion::event::Key;

pub struct ZeugtrisHighscoresPage;

impl ZeugtrisHighscoresPage {
    pub fn new() -> Self {
        Self
    }
}

impl Page for ZeugtrisHighscoresPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        if key == Key::Esc {
            return Action::Pop;
        }
        Action::None
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        display.clear(ctx);
    }
}