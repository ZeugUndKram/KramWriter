// src/pages/logo.rs
use super::{Page, PageId};
use crate::display::SharpDisplay;
use anyhow::Result;
use rpi_memory_display::Pixel;

pub struct LogoPage {
    // Could store logo image data here
}

impl LogoPage {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
    
    fn load_logo() -> Option<Vec<u8>> {
        // Try to load from /home/kramwriter/KramWriter/assets/logo.bmp
        std::fs::read("/home/kramwriter/KramWriter/assets/logo.bmp").ok()
    }
}

impl Page for LogoPage {
    fn draw(&mut self, display: &mut SharpDisplay) -> Result<()> {
        display.clear();
        
        // Try to draw BMP logo
        if let Some(bmp_data) = Self::load_logo() {
            // Simple BMP parsing (header skip + monochrome)
            // For now, draw placeholder
            display.draw_text(150, 100, "LOGO");
            display.draw_text(120, 120, "Press ENTER for menu");
        } else {
            display.draw_text(150, 100, "NO LOGO");
            display.draw_text(120, 120, "Press ENTER for menu");
        }
        
        display.update()?;
        Ok(())
    }
    
    fn handle_key(&mut self, key: termion::event::Key) -> Result<Option<PageId>> {
        match key {
            termion::event::Key::Char('\n') => Ok(Some(PageId::Menu)),
            _ => Ok(None),
        }
    }
}