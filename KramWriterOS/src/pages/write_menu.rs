use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;
use crate::pages::file_browser::{FileBrowserPage, BrowserMode};

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
        let asset_path = "/home/kramwriter/KramWriter/assets/Writing/menu";
        Self {
            current_index: 0,
            title: Bitmap::load(&format!("{}/title.bmp", asset_path)).ok(),
            new_file_variants: [
                Bitmap::load(&format!("{}/new_file.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/new_file_1.bmp", asset_path)).ok(),
            ],
            open_file_variants: [
                Bitmap::load(&format!("{}/open_file_0.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/open_file_1.bmp", asset_path)).ok(),
            ],
        }
    }

    fn draw_layer(&self, display: &mut SharpDisplay, bmp: &Bitmap, y_offset: i32, ctx: &Context) {
        for y in 0..bmp.height {
            let screen_y = y as i32 + y_offset;
            if screen_y >= 0 && screen_y < 240 {
                for x in 0..bmp.width.min(400) {
                    if bmp.pixels[y * bmp.width + x] == Pixel::Black {
                        display.draw_pixel(x, screen_y as usize, Pixel::Black, ctx);
                    }
                }
            }
        }
    }
}

impl Page for WriteMenuPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Up => {
                if self.current_index == 1 { self.current_index = 0; }
                Action::None
            }
            Key::Down => {
                if self.current_index == 0 { self.current_index = 1; }
                Action::None
            }
            Key::Char('\n') => {
                if self.current_index == 0 {
                    Action::Push(Box::new(FileBrowserPage::new(BrowserMode::Full)))
                } else {
                    Action::Push(Box::new(FileBrowserPage::new(BrowserMode::OpenFile)))
                }
            }
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        if let Some(bmp) = &self.title { self.draw_layer(display, bmp, 0, ctx); }

        let new_idx = if self.current_index == 0 { 0 } else { 1 };
        if let Some(bmp) = &self.new_file_variants[new_idx] {
            self.draw_layer(display, bmp, NEW_FILE_Y, ctx);
        }

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
                        display.draw_pixel(x, screen_y as usize, pixel, ctx);
                    }
                }
            }
        }
    }
}