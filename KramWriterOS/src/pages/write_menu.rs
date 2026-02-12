use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;

// Shifted both up: 75 -> 40 and 125 -> 90
const NEW_FILE_Y: i32 = 40; 
const OPEN_FILE_Y: i32 = 90;

pub struct WriteMenuPage {
    current_index: usize, 
    title: Option<Bitmap>,
    new_file_variants: [Option<Bitmap>; 2],
    open_file_variants: [Option<Bitmap>; 2],
}

impl WriteMenuPage {
    pub fn new() -> Self {
        let title = Bitmap::load("/home/kramwriter/KramWriter/assets/Writing/menu/title.bmp").ok();
        
        let new_file_variants = [
            Bitmap::load("/home/kramwriter/KramWriter/assets/Writing/menu/new_file.bmp").ok(),
            Bitmap::load("/home/kramwriter/KramWriter/assets/Writing/menu/new_file_1.bmp").ok(),
        ];

        let open_file_variants = [
            Bitmap::load("/home/kramwriter/KramWriter/assets/Writing/menu/open_file_0.bmp").ok(),
            Bitmap::load("/home/kramwriter/KramWriter/assets/Writing/menu/open_file_1.bmp").ok(),
        ];

        Self {
            current_index: 0,
            title,
            new_file_variants,
            open_file_variants,
        }
    }
}

impl Page for WriteMenuPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            // Strict navigation: No looping
            Key::Up => {
                if self.current_index == 1 {
                    self.current_index = 0;
                }
                Action::None
            }
            Key::Down => {
                if self.current_index == 0 {
                    self.current_index = 1;
                }
                Action::None
            }
            Key::Char('\n') => {
                if self.current_index == 0 {
                    Action::Push(Box::new(crate::pages::file_browser::FileBrowserPage::new()))
                } else {
                    Action::None
                }
            }
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // 1. Background Title
        if let Some(bmp) = &self.title {
            self.draw_layer(display, bmp, 0, ctx);
        }

        // 2. New File
        let new_idx = if self.current_index == 0 { 0 } else { 1 };
        if let Some(bmp) = &self.new_file_variants[new_idx] {
            self.draw_layer(display, bmp, NEW_FILE_Y, ctx);
        }

        // 3. Open File
        let open_idx = if self.current_index == 1 { 0 } else { 1 };
        if let Some(bmp) = &self.open_file_variants[open_idx] {
            self.draw_layer(display, bmp, OPEN_FILE_Y, ctx);
        }
    }
}

impl WriteMenuPage {
    fn draw_layer(&self, display: &mut SharpDisplay, bmp: &Bitmap, y_offset: i32, ctx: &Context) {
        for y in 0..bmp.height {
            let screen_y = y as i32 + y_offset;
            // Bounds check to prevent drawing off-screen
            if screen_y >= 0 && screen_y < 240 {
                for x in 0..bmp.width.min(400) {
                    let pixel = bmp.pixels[y * bmp.width + x];
                    if pixel == Pixel::Black {
                        display.draw_pixel(x, screen_y as usize, pixel, ctx);
                    }
                }
            }
        }
    }
}