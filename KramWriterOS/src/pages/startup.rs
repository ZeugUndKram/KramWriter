use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use termion::event::Key;

pub struct LogoPage;

impl LogoPage {
    pub fn new() -> Self { Self }
}

impl Page for LogoPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Char('\n') => Action::Replace(Box::new(crate::pages::menu::MenuPage::new())),
            Key::Esc => Action::Exit,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, _ctx: &Context) {
        display.draw_text(150, 100, "LOGO SCREEN");
        display.draw_text(130, 130, "Press Enter to Start");
    }
}