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
    fn update(&mut self, key: Key, ctx: &mut Context) -> Action;
    
    // Add this new optional method with a default "do nothing" implementation
    fn tick(&mut self, _ctx: &mut Context) -> Action {
        Action::None
    }
    fn draw(&self, display: &mut SharpDisplay, ctx: &Context);
}
