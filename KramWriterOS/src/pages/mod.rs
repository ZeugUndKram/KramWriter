pub mod startup;
pub mod menu;
pub mod credits;
pub mod settings;
pub mod timezone;
pub mod write_menu;

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
    fn draw(&self, display: &mut SharpDisplay, ctx: &Context);
}