// src/pages/mod.rs
pub mod logo;
pub mod menu;

use crate::display::SharpDisplay;
use anyhow::Result;

pub trait Page {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()>;
    fn handle_key(&mut self, key: termion::event::Key) -> Result<Option<PageId>>;
}

// Re-export
pub use logo::LogoPage;
pub use menu::MenuPage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageId {
    Logo,
    Menu,
}