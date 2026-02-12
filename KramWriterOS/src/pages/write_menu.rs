use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;

// Reduced spacing: Moved them closer to the center
const NEW_FILE_Y: i32 = 75; 
const OPEN_FILE_Y: i32 = 125;

pub struct WriteMenuPage {
    current_index: usize, // 0 for New File, 1 for Open File
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
            // No looping: Up only works if we are at the bottom, Down only works if we are at the top
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
                    println!("Creating new document...");
                    // Future: Action::Push(Box::new(EditorPage::new()))
                } else {
                    println!("Opening file browser...");
                }
                Action::None
            }
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        if let Some(bmp) = &self.title {
            self.draw_layer(display, bmp, 0, ctx);
        }

        // New File (Index 0)
        let new_idx = if self.current_index == 0 { 0 } else { 1 };
        if let Some(bmp) = &self.new_file_variants[new_idx] {
            self.draw_layer(display, bmp, NEW_FILE_Y, ctx);
        }

        // Open File (Index 1)
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
            if screen_y >= 0 && screen_y < 240 {
                for x in 0..bmp.width.min(400) {
                    let pixel = bmp.pixels[y * bmp.width + x];
                    if pixel == Pixel::Black {
                        display.draw_pixel(x, (screen_y) as usize, pixel, ctx);
                    }
                }
            }
        }
    }
}