use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;

pub struct WriteMenuPage {
    current_index: usize, // 0 for New File, 1 for Open File
    title: Option<Bitmap>,
    // Buttons stored as [Selected, Unselected]
    new_file_variants: [Option<Bitmap>; 2],
    open_file_variants: [Option<Bitmap>; 2],
}

impl WriteMenuPage {
    pub fn new() -> Self {
        let title = Bitmap::load("/home/kramwriter/KramWriter/assets/Write/title.bmp").ok();
        
        // Based on your list: 
        // new_file.bmp (_0 equivalent) is selected, new_file_1.bmp is unselected
        let new_file_variants = [
            Bitmap::load("/home/kramwriter/KramWriter/assets/Write/new_file.bmp").ok(),
            Bitmap::load("/home/kramwriter/KramWriter/assets/Write/new_file_1.bmp").ok(),
        ];

        // open_file_0.bmp is selected, open_file_1.bmp is unselected
        let open_file_variants = [
            Bitmap::load("/home/kramwriter/KramWriter/assets/Write/open_file_0.bmp").ok(),
            Bitmap::load("/home/kramwriter/KramWriter/assets/Write/open_file_1.bmp").ok(),
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
            Key::Up | Key::Down => {
                self.current_index = if self.current_index == 0 { 1 } else { 0 };
                Action::None
            }
            Key::Char('\n') => {
                if self.current_index == 0 {
                    // Placeholder: Logic for starting a new document
                    println!("Creating new document...");
                } else {
                    // Placeholder: Logic for opening the file browser
                    println!("Opening file browser...");
                }
                Action::None
            }
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // Draw the background/title first
        if let Some(bmp) = &self.title {
            self.draw_layer(display, bmp, ctx);
        }

        // Draw New File button
        let new_idx = if self.current_index == 0 { 0 } else { 1 };
        if let Some(bmp) = &self.new_file_variants[new_idx] {
            self.draw_layer(display, bmp, ctx);
        }

        // Draw Open File button
        let open_idx = if self.current_index == 1 { 0 } else { 1 };
        if let Some(bmp) = &self.open_file_variants[open_idx] {
            self.draw_layer(display, bmp, ctx);
        }
    }
}

impl WriteMenuPage {
    fn draw_layer(&self, display: &mut SharpDisplay, bmp: &Bitmap, ctx: &Context) {
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