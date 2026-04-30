use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use termion::event::Key;

pub struct ZeugtrisPage;

impl ZeugtrisPage {
    pub fn new() -> Self {
        Self
    }
}

impl Page for ZeugtrisPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        if key == Key::Esc {
            return Action::Pop;
        }
        Action::None
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        display.clear(ctx);
        // Placeholder text or graphic logic goes here later
    }
}