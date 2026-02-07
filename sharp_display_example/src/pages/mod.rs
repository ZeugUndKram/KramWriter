pub mod logo;
pub mod menu;
pub mod zeugtris_menu;
pub mod zeugtris;
pub mod writing_game;
pub mod writing_renderer;
pub mod writing;

use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageId {
    Logo,
    Menu,
    ZeugtrisMenu,
    Zeugtris,
    Writing,
}

pub trait Page {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()>;
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>>;
}

pub use logo::LogoPage;
pub use menu::MenuPage;
pub use zeugtris_menu::ZeugtrisMenuPage;
pub use zeugtris::ZeugtrisPage;
pub use writing::WritingPage;