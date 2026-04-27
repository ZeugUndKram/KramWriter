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
    Redraw,
    Push(Box<dyn Page>),
    Pop,
    Replace(Box<dyn Page>),
    Exit,
}

impl Action {
    // Helper so we don't need PartialEq
    pub fn is_none(&self) -> bool {
        matches!(self, Action::None)
    }
}

pub trait Page {
    fn update(&mut self, key: Key, ctx: &mut Context) -> Action;
    
    fn tick(&mut self, _ctx: &mut Context) -> Action {
        Action::None
    }
    fn draw(&self, display: &mut SharpDisplay, ctx: &Context);
}