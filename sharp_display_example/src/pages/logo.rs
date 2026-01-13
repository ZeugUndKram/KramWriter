use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;

pub struct LogoPage;

impl LogoPage {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl Page for LogoPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear();
        display.draw_text(150, 100, "LOGO");
        display.draw_text(120, 120, "Press ENTER for menu");
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
            Key::Char('\n') => Ok(Some(PageId::Menu)),
            _ => Ok(None),
        }
    }
}