use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;

// Matches your new simplified filenames: "3.bmp", "-3.bmp", "UTC.bmp", etc.
const OFFSETS: [&str; 31] = [
    "-11", "-10", "-9", "-8", "-7", "-6", "-5", "-4", "-3.5", "-3", "-2", "-1", 
    "UTC", "1", "2", "3", "3.5", "4", "4.5", "5", "5.5", "6", "6.5", "7", "8", "9", "9.5", "10", "10.5", "11", "12"
];

pub struct TimezonePage {
    current_index: usize,
    base_map: Option<Bitmap>,
    offset_images: Vec<Option<Bitmap>>,
}

impl TimezonePage {
    pub fn new() -> Self {
        let base_map = Bitmap::load("/home/kramwriter/KramWriter/assets/Timezone/map_clear.bmp").ok();
        let mut offset_images = Vec::new();

        for offset in OFFSETS.iter() {
            let path = format!("/home/kramwriter/KramWriter/assets/Timezone/{}.bmp", offset);
            let img = Bitmap::load(&path).ok();
            
            if img.is_none() {
                println!("⚠️ Timezone asset missing: {}", path);
            }
            offset_images.push(img);
        }

        // Start at UTC (Index 12)
        Self { 
            current_index: 12, 
            base_map, 
            offset_images 
        }
    }
}

impl Page for TimezonePage {
    fn update(&mut self, key: Key, ctx: &mut Context) -> Action {
        match key {
            Key::Right => {
                self.current_index = (self.current_index + 1) % OFFSETS.len();
                Action::None
            }
            Key::Left => {
                if self.current_index == 0 {
                    self.current_index = OFFSETS.len() - 1;
                } else {
                    self.current_index -= 1;
                }
                Action::None
            }
            // Pressing ENTER saves the selection
            Key::Char('\n') => {
                let selected_tz = OFFSETS[self.current_index].to_string();
                ctx.timezone = selected_tz; // Save to global context
                
                println!("Timezone saved: {}", ctx.timezone); // Debug print
                Action::Pop // Return to Settings
            }
            Key::Esc => Action::Pop, // Return without saving (optional)
            _ => Action::None,
        }
    }
    
    // ... draw function stays the same ...


    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // Draw the static map first
        if let Some(map) = &self.base_map {
            self.draw_full(display, map, ctx);
        }

        // Overlay the selected timezone (only black pixels for "transparency")
        if let Some(offset_bmp) = &self.offset_images[self.current_index] {
            self.draw_transparent(display, offset_bmp, ctx);
        }
    }
}

impl TimezonePage {
    fn draw_full(&self, display: &mut SharpDisplay, bmp: &Bitmap, ctx: &Context) {
        for y in 0..bmp.height.min(240) {
            for x in 0..bmp.width.min(400) {
                display.draw_pixel(x, y, bmp.pixels[y * bmp.width + x], ctx);
            }
        }
    }

    fn draw_transparent(&self, display: &mut SharpDisplay, bmp: &Bitmap, ctx: &Context) {
        for y in 0..bmp.height.min(240) {
            for x in 0..bmp.width.min(400) {
                let pixel = bmp.pixels[y * bmp.width + x];
                if pixel == Pixel::Black {
                    display.draw_pixel(x, y, pixel, ctx);
                }
            }
        }
    }
}