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

// Added PartialEq so the main loop can compare actions (e.g., action != Action::None)
// Added Debug to help with any future troubleshooting
#[derive(PartialEq, Debug)]
pub enum Action {
    None,
    Redraw, // Added this so tick() can trigger a render without changing pages
    Push(Box<dyn Page>),
    Pop,
    Replace(Box<dyn Page>),
    Exit,
}

pub trait Page {
    fn update(&mut self, key: Key, ctx: &mut Context) -> Action;
    
    // Default implementation returns None so existing pages don't need to change
    fn tick(&mut self, _ctx: &mut Context) -> Action {
        Action::None
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context);
}

// Note: Because we added PartialEq to Action, and Action contains a Box<dyn Page>,
// we have to manually tell Rust how to compare Actions containing Pages. 
// Since we don't actually need to compare page contents, we implement 
// PartialEq for Action manually or simplify the logic. 

// A cleaner way to handle the comparison in main.rs without complex trait bounds:
impl Action {
    pub fn is_none(&self) -> bool {
        matches!(self, Action::None)
    }
}