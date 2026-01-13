// src/main.rs
mod pages;
mod display;

use anyhow::Result;
use pages::{Page, LogoPage, MenuPage};
use display::SharpDisplay;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PageId {
    Logo,
    Menu,
}

struct App {
    display: SharpDisplay,
    current_page: PageId,
    pages: HashMap<PageId, Box<dyn Page>>,
}

impl App {
    fn new() -> Result<Self> {
        let display = SharpDisplay::new(6)?;  // CS pin 6
        
        let mut pages = HashMap::new();
        pages.insert(PageId::Logo, Box::new(LogoPage::new()?));
        pages.insert(PageId::Menu, Box::new(MenuPage::new()?));
        
        Ok(Self {
            display,
            current_page: PageId::Logo,
            pages,
        })
    }
    
    fn run(&mut self) -> Result<()> {
        use termion::{input::TermRead, raw::IntoRawMode};
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout().into_raw_mode()?;
        
        // Draw initial page
        self.draw_current_page()?;
        
        for key in stdin.keys() {
            match key? {
                termion::event::Key::Char('\n') => {
                    match self.current_page {
                        PageId::Logo => self.current_page = PageId::Menu,
                        PageId::Menu => self.current_page = PageId::Logo,
                    }
                    self.draw_current_page()?;
                }
                termion::event::Key::Ctrl('c') => break,
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn draw_current_page(&mut self) -> Result<()> {
        if let Some(page) = self.pages.get_mut(&self.current_page) {
            page.draw(&mut self.display)?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()
}