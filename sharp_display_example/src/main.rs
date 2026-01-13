mod pages;
mod display;

use anyhow::Result;
use pages::{PageId, LogoPage, MenuPage};
use display::SharpDisplay;
use std::collections::HashMap;

struct App {
    display: SharpDisplay,
    current_page: PageId,
    pages: HashMap<PageId, Box<dyn pages::Page>>,
}

impl App {
    fn new() -> Result<Self> {
        let display = SharpDisplay::new(6)?;
        
        let mut pages: HashMap<PageId, Box<dyn pages::Page>> = HashMap::new();
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
        let _stdout = std::io::stdout().into_raw_mode()?;
        
        self.draw_current_page()?;
        
        for key in stdin.keys() {
            let key = key?;
            
            if let Some(page) = self.pages.get_mut(&self.current_page) {
                if let Some(next_page) = page.handle_key(key)? {
                    self.current_page = next_page;
                }
            }
            
            self.draw_current_page()?;
            
            if key == termion::event::Key::Ctrl('c') {
                break;
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