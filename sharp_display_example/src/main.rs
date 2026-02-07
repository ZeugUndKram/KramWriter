mod pages;
mod display;
mod game;
mod writing_game;
mod writing_renderer;
mod writing_menu;

use anyhow::Result;
use pages::{PageId, LogoPage, MenuPage, ZeugtrisMenuPage, ZeugtrisPage, WritingPage};  // Updated
use display::SharpDisplay;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::io;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct App {
    display: SharpDisplay,
    current_page: PageId,
    pages: HashMap<PageId, Box<dyn pages::Page>>,
    last_frame_time: Instant,
    frame_duration: Duration,
}

impl App {
    fn new() -> Result<Self> {
        let display = SharpDisplay::new(6)?;
        
        let mut pages: HashMap<PageId, Box<dyn pages::Page>> = HashMap::new();
        pages.insert(PageId::Logo, Box::new(LogoPage::new()?));
        pages.insert(PageId::Menu, Box::new(MenuPage::new()?));
        pages.insert(PageId::ZeugtrisMenu, Box::new(ZeugtrisMenuPage::new()?));
        pages.insert(PageId::Zeugtris, Box::new(ZeugtrisPage::new()?));
        pages.insert(PageId::Writing, Box::new(WritingPage::new()?));  // New
        
        Ok(Self {
            display,
            current_page: PageId::Logo,
            pages,
            last_frame_time: Instant::now(),
            frame_duration: Duration::from_millis(16),
        })
    }
    
    fn run(&mut self) -> Result<()> {
        let _stdout = io::stdout().into_raw_mode()?;
        
        // Create a keys iterator
        let stdin = io::stdin();
        let mut keys = stdin.lock().keys();
        
        // Initial draw
        self.draw_current_page()?;
        
        loop {
            // Check for keyboard input
            if let Some(Ok(key)) = keys.next() {
                self.handle_key(key)?;
                
                if key == Key::Ctrl('c') {
                    break;
                }
                
                // Force redraw after handling key
                self.draw_current_page()?;
                self.last_frame_time = Instant::now();
                continue;
            }
            
            // Fixed frame rate update
            let now = Instant::now();
            if now.duration_since(self.last_frame_time) >= self.frame_duration {
                self.draw_current_page()?;
                self.last_frame_time = now;
            }
            
            std::thread::sleep(Duration::from_millis(5));
        }
        
        Ok(())
    }
    
    fn handle_key(&mut self, key: Key) -> Result<()> {
        if let Some(page) = self.pages.get_mut(&self.current_page) {
            if let Some(next_page) = page.handle_key(key)? {
                self.current_page = next_page;
                // Force redraw on page change
                self.draw_current_page()?;
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