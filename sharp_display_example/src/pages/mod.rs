pub mod logo;
pub mod menu;
pub mod write_menu;

use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageId {
    Logo,
    Menu,
    WriteMenu,
}

pub trait Page {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()>;
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>>;
}

pub use logo::LogoPage;
pub use menu::MenuPage;
pub use write_menu::WriteMenuPage;