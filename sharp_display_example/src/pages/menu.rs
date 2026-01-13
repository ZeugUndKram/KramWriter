use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use termion::event::Key;

pub struct MenuPage;

impl MenuPage {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
    
    fn load_text() -> String {
        std::fs::read_to_string("/home/kramwriter/KramWriter/assets/Write_0")
            .unwrap_or_else(|_| "Write Mode".to_string())
    }
}

impl Page for MenuPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear()?;
        
        let text = Self::load_text();
        let lines: Vec<&str> = text.lines().collect();
        
        for (i, line) in lines.iter().enumerate().take(10) {
            let y = 100 + i * 20;
            let text_width = line.len() * 6;
            let x = (400usize.saturating_sub(text_width)) / 2;
            display.draw_text(x, y, line);
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<Option<PageId>> {
        match key {
            Key::Char('\n') => Ok(Some(PageId::Logo)),
            _ => Ok(None),
        }
    }
}