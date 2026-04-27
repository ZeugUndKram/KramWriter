pub mod startup;
pub mod menu;
pub mod credits;
pub mod settings;
pub mod timezone;
pub mod write_menu;
pub mod file_browser;
pub mod name_entry;
pub mod editor;
pub mod simplenote_setup;

use crate::context::Context;
use crate::display::SharpDisplay;
use termion::event::Key;

pub enum Action {
    None,
    Push(Box<dyn Page>),
    Pop,
    Replace(Box<dyn Page>),
    Exit,
}

pub trait Page {
    fn update(&mut self, key: Option<Key>, ctx: &mut Context) -> Action {
    // 1. Progress check logic here (runs every tick)
    // ... (Your streaming logic from the previous fix) ...

    // 2. Key handling logic (only runs if a key exists)
    if let Some(k) = key {
        match self.focus {
            // ... (Your key matching logic) ...
        }
    } else {
        Action::None
    }
}
    fn draw(&self, display: &mut SharpDisplay, ctx: &Context);
}