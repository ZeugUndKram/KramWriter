use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;

pub struct EditorPage {
    path: PathBuf,
    content: String,
    is_dirty: bool,
    renderer: FontRenderer,
    // Assets
    save_icons: [Option<Bitmap>; 2],
    wifi_icons: [Option<Bitmap>; 5],
    weather_icons: Vec<Option<Bitmap>>, // 0: sunny, 1: cloudy, 2: rainy, 3: snowy, 4: stormy
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

        let content = std::fs::read_to_string(&path).unwrap_or_default();

        Self {
            path,
            content,
            is_dirty: false,
            renderer,
            save_icons,
            wifi_icons,
            weather_icons,
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

        // Divider Line
        for x in 0..400 { display.draw_pixel(x, y_start, Pixel::Black, ctx); }

        // 1. Save Icon
        let save_icon = if self.is_dirty { &self.save_icons[0] } else { &self.save_icons[1] };
        if let Some(bmp) = save_icon { self.draw_icon(display, bmp, 5, y_start + 3, ctx); }

        // 2. Filename
        let filename = self.path.file_name()
            .map(|n| n.to_string_lossy().to_string().to_uppercase())
            .unwrap_or_else(|| "UNTITLED.TXT".to_string());
        self.renderer.draw_text_colored(display, &filename, 28, y_start + 18, 18.0, Pixel::Black, ctx);

        // 3. Word Count
        let w_count = format!("W:{}", self.get_word_count());
        self.renderer.draw_text_colored(display, &w_count, 180, y_start + 18, 18.0, Pixel::Black, ctx);

        // 4. Time (Static for now)
        self.renderer.draw_text_colored(display, "12:32", 305, y_start + 18, 18.0, Pixel::Black, ctx);

        // 5. Weather (Sunny)
        if let Some(bmp) = &self.weather_icons[0] { self.draw_icon(display, bmp, 348, y_start + 3, ctx); }

        // 6. Wifi (Full)
        if let Some(bmp) = &self.wifi_icons[4] { self.draw_icon(display, bmp, 372, y_start + 3, ctx); }
    }
}

impl Page for EditorPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Esc => Action::Pop,
            Key::Char('\n') => { self.content.push('\n'); self.is_dirty = true; Action::None }
            Key::Char(c) => { self.content.push(c); self.is_dirty = true; Action::None }
            Key::Backspace => { self.content.pop(); self.is_dirty = true; Action::None }
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // Main Text Rendering
        self.renderer.draw_text_colored(display, &self.content, 10, 30, 22.0, Pixel::Black, ctx);
        
        // Bottom Status Bar
        self.draw_bottom_bar(display, ctx);
    }
}