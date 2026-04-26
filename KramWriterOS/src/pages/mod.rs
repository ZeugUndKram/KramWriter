pub mod startup;
pub mod menu;
pub mod credits;
pub mod settings;
pub mod timezone;
pub mod write_menu;
pub mod file_browser;
pub mod name_entry;
pub mod editor;
pub mod zeugtris;

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
    
    // NEW: Runs continuously. Returns `true` if the screen needs to be redrawn.
    fn tick(&mut self, _ctx: &mut Context) -> bool {
        false // Default for text pages: no continuous redraw needed
    }
    
    // CHANGED: Now takes &mut self so animations/games can update internal states
    fn draw(&mut self, display: &mut SharpDisplay, ctx: &Context);
}