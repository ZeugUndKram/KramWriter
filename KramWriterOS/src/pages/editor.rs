use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;
use std::fs;

pub struct EditorPage {
    path: PathBuf,
    content: String,
    is_dirty: bool,
    renderer: FontRenderer,
    font_size: f32,
    // Assets
    save_icons: [Option<Bitmap>; 2],
    wifi_icons: [Option<Bitmap>; 5],
    weather_icons: Vec<Option<Bitmap>>,
}

impl EditorPage {
    pub fn new(path: PathBuf) -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        let asset_path = "/home/kramwriter/KramWriter/assets/Writing";

        let save_icons = [
            Bitmap::load(&format!("{}/save_0.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/save_1.bmp", asset_path)).ok(),
        ];

        let wifi_icons = [
            Bitmap::load(&format!("{}/wifi_0.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/wifi_1.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/wifi_2.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/wifi_3.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/wifi_4.bmp", asset_path)).ok(),
        ];

        let weather_icons = vec![
            Bitmap::load(&format!("{}/sunny.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/cloudy.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/rainy.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/snowy.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/stormy.bmp", asset_path)).ok(),
        ];

        let content = fs::read_to_string(&path).unwrap_or_default();

        Self {
            path,
            content,
            is_dirty: false,
            renderer,
            font_size: 22.0, // Default size
            save_icons,
            wifi_icons,
            weather_icons,
        }
    }

    fn save_file(&mut self) {
        if fs::write(&self.path, &self.content).is_ok() {
            self.is_dirty = false;
        }
    }

    fn get_word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    fn draw_icon(&self, display: &mut SharpDisplay, bmp: &Bitmap, x_off: usize, y_off: usize, ctx: &Context) {
        for y in 0..bmp.height {
            for x in 0..bmp.width {
                if bmp.pixels[y * bmp.width + x] == Pixel::Black {
                    let sx = x + x_off;
                    let sy = y + y_off;
                    if sx < 400 && sy < 240 {
                        display.draw_pixel(sx, sy, Pixel::Black, ctx);
                    }
                }
            }
        }
    }

    fn draw_bottom_bar(&self, display: &mut SharpDisplay, ctx: &Context) {
        let y_start = 218;
        let y_text = y_start as i32 + 18;

        for x in 0..400 { display.draw_pixel(x, y_start, Pixel::Black, ctx); }

        let save_icon = if self.is_dirty { &self.save_icons[0] } else { &self.save_icons[1] };
        if let Some(bmp) = save_icon { self.draw_icon(display, bmp, 5, y_start + 3, ctx); }

        let filename = self.path.file_name()
            .map(|n| n.to_string_lossy().to_string().to_uppercase())
            .unwrap_or_else(|| "UNTITLED.TXT".to_string());
        
        self.renderer.draw_text_colored(display, &filename, 28, y_text, 18.0, Pixel::Black, ctx);

        let w_count = format!("W:{}", self.get_word_count());
        self.renderer.draw_text_colored(display, &w_count, 180, y_text, 18.0, Pixel::Black, ctx);

        self.renderer.draw_text_colored(display, "12:32", 305, y_text, 18.0, Pixel::Black, ctx);

        if let Some(bmp) = &self.weather_icons[0] { self.draw_icon(display, bmp, 348, y_start + 3, ctx); }
        if let Some(bmp) = &self.wifi_icons[4] { self.draw_icon(display, bmp, 372, y_start + 3, ctx); }
    }
}

impl Page for EditorPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Esc => Action::Pop,
            
            // CTRL + S: Save
            Key::Ctrl('s') => {
                self.save_file();
                Action::None
            }

            // CTRL + '+': Increase Font Size
            Key::Ctrl('+') | Key::Ctrl('=') => {
                if self.font_size < 48.0 { self.font_size += 2.0; }
                Action::None
            }

            // CTRL + '-': Decrease Font Size
            Key::Ctrl('-') => {
                if self.font_size > 12.0 { self.font_size -= 2.0; }
                Action::None
            }

            Key::Char('\n') => {
                self.content.push('\n');
                self.is_dirty = true;
                Action::None
            }

            Key::Char(c) => {
                self.content.push(c);
                self.is_dirty = true;
                Action::None
            }

            Key::Backspace => {
                self.content.pop();
                self.is_dirty = true;
                Action::None
            }

            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        let margin = 10;
        let max_width = 380;
        let mut cur_y = 30;
        let line_height = (self.font_size * 1.2) as i32;

        // Simple Line Wrapping Logic
        let paragraphs = self.content.split('\n');
        
        for para in paragraphs {
            let mut current_line = String::new();
            let words = para.split_inclusive(' ');

            for word in words {
                let test_line = format!("{}{}", current_line, word);
                let width = self.renderer.calculate_width(&test_line, self.font_size);

                if width > max_width as f32 && !current_line.is_empty() {
                    // Draw current line and start new one
                    self.renderer.draw_text_colored(display, &current_line, margin, cur_y, self.font_size, Pixel::Black, ctx);
                    cur_y += line_height;
                    current_line = word.to_string();
                } else {
                    current_line = test_line;
                }

                // Bounds check: stop drawing if we hit the bottom bar
                if cur_y > 200 { break; }
            }

            // Draw the last remaining bit of the paragraph
            if cur_y <= 200 {
                self.renderer.draw_text_colored(display, &current_line, margin, cur_y, self.font_size, Pixel::Black, ctx);
                cur_y += line_height;
            }
            
            if cur_y > 200 { break; }
        }

        self.draw_bottom_bar(display, ctx);
    }
}