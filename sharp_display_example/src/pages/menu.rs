// src/pages/menu.rs
use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;

pub struct MenuPage {
    items: Vec<String>,
    selected: usize,
}

impl MenuPage {
    pub fn new() -> Result<Self> {
        let items = std::fs::read_to_string("/home/kramwriter/KramWriter/assets/Settings_0")
            .map(|content| content.lines().map(String::from).collect())
            .unwrap_or_else(|_| vec![
                "Brightness".to_string(),
                "Contrast".to_string(),
                "Exit".to_string(),
            ]);
            
        Ok(Self {
            items,
            selected: 0,
        })
    }
}

impl Page for LogoPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear();
        
        display.draw_text(150, 20, "MENU");
        
        for (i, item) in self.items.iter().enumerate().take(10) {
            let y = 50 + i * 20;
            if i == self.selected {
                display.draw_text(40, y, ">");
            }
            display.draw_text(60, y, item);
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: termion::event::Key) -> Result<Option<PageId>> {
        match key {
            termion::event::Key::Char('\n') => Ok(Some(PageId::Logo)),
            termion::event::Key::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Ok(None)
            }
            termion::event::Key::Down => {
                if self.selected < self.items.len() - 1 {
                    self.selected += 1;
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}