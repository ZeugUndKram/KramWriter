mod pages;
mod display;

use anyhow::Result;
use pages::{PageId, LogoPage, MenuPage, WriteMenuPage, ZeugtrisMenuPage, ZeugtrisPage};
use display::SharpDisplay;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::io::{self, Read};
use termion::event::Key;
use termion::input::TermRead;

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
        pages.insert(PageId::WriteMenu, Box::new(WriteMenuPage::new()?));
        pages.insert(PageId::ZeugtrisMenu, Box::new(ZeugtrisMenuPage::new()?));
        pages.insert(PageId::Zeugtris, Box::new(ZeugtrisPage::new()?));
        
        Ok(Self {
            display,
            current_page: PageId::Logo,
            pages,
            last_frame_time: Instant::now(),
            frame_duration: Duration::from_millis(16), // ~60 FPS
        })
    }
    
    fn run(&mut self) -> Result<()> {
        use termion::raw::IntoRawMode;
        
        let stdin = io::stdin();
        let _stdout = io::stdout().into_raw_mode()?;
        
        // Set stdin to non-blocking mode
        let mut stdin_locked = stdin.lock();
        
        // Initial draw
        self.draw_current_page()?;
        
        loop {
            // Check for input without blocking
            if let Some(Ok(key)) = stdin_locked.by_ref().keys().next() {
                if let Some(page) = self.pages.get_mut(&self.current_page) {
                    if let Some(next_page) = page.handle_key(key)? {
                        self.current_page = next_page;
                        // Force immediate redraw on page change
                        self.draw_current_page()?;
                        self.last_frame_time = Instant::now();
                        continue;
                    }
                }
                
                if key == Key::Ctrl('c') {
                    break;
                }
                
                // Force redraw after handling key
                self.draw_current_page()?;
                self.last_frame_time = Instant::now();
            }
            
            // Redraw at fixed intervals for smooth animation
            let now = Instant::now();
            if now.duration_since(self.last_frame_time) >= self.frame_duration {
                self.draw_current_page()?;
                self.last_frame_time = now;
            }
            
            // Small sleep to prevent 100% CPU usage while still being responsive
            std::thread::sleep(Duration::from_millis(5));
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