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
    cursor_pos: usize,
    scroll_line_offset: usize,
    is_dirty: bool,
    renderer: FontRenderer,
    font_size: f32,
    save_icons: [Option<Bitmap>; 2],
    wifi_icons: [Option<Bitmap>; 5],
    weather_icons: Vec<Option<Bitmap>>,
}

impl EditorPage {
    pub fn new(path: PathBuf) -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        let asset_path = "/home/kramwriter/KramWriter/assets/Writing";
        let content = fs::read_to_string(&path).unwrap_or_default();
        let len = content.len();

        Self {
            path,
            content,
            cursor_pos: len,
            scroll_line_offset: 0,
            is_dirty: false,
            renderer,
            font_size: 22.0,
            save_icons: [
                Bitmap::load(&format!("{}/save_0.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/save_1.bmp", asset_path)).ok(),
            ],
            wifi_icons: [
                Bitmap::load(&format!("{}/wifi_0.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/wifi_1.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/wifi_2.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/wifi_3.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/wifi_4.bmp", asset_path)).ok(),
            ],
            weather_icons: vec![
                Bitmap::load(&format!("{}/sunny.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/cloudy.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/rainy.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/snowy.bmp", asset_path)).ok(),
                Bitmap::load(&format!("{}/stormy.bmp", asset_path)).ok(),
            ],
        }
    }

    fn get_word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    fn save_file(&mut self) {
        if fs::write(&self.path, &self.content).is_ok() {
            self.is_dirty = false;
        }
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

    fn get_wrapped_lines(&self, max_width: f32) -> Vec<(String, usize)> {
        let mut lines = Vec::new();
        let mut current_pos = 0;

        for paragraph in self.content.split_inclusive('\n') {
            let mut current_line = String::new();
            let mut line_start_pos = current_pos;

            for word in paragraph.split_inclusive(' ') {
                let test_line = format!("{}{}", current_line, word);
                // FIXED: Explicit cast to f32 for the width comparison
                if (self.renderer.calculate_width(&test_line, self.font_size) as f32) > max_width && !current_line.is_empty() {
                    lines.push((current_line.clone(), line_start_pos));
                    line_start_pos += current_line.len();
                    current_line = word.to_string();
                } else {
                    current_line = test_line;
                }
            }
            lines.push((current_line.clone(), line_start_pos));
            current_pos += paragraph.len();
        }
        lines
    }

    fn draw_scrollbar(&self, display: &mut SharpDisplay, total_lines: usize, visible_count: usize, ctx: &Context) {
        if total_lines <= visible_count { return; }
        let track_top = 30;
        let track_bottom = 210;
        let track_h = track_bottom - track_top;

        let thumb_h = ((visible_count as f32 / total_lines as f32) * track_h as f32) as i32;
        let thumb_h = thumb_h.max(15);

        let scrollable_dist = (total_lines - visible_count) as f32;
        let thumb_y = track_top + ((self.scroll_line_offset as f32 / scrollable_dist) * (track_h - thumb_h) as f32) as i32;

        for y in thumb_y..(thumb_y + thumb_h) {
            for x in 395..399 {
                display.draw_pixel(x, y as usize, Pixel::Black, ctx);
            }
        }
    }

    fn draw_bottom_bar(&self, display: &mut SharpDisplay, ctx: &Context) {
        let y_start = 218;
        let y_text = y_start as i32 + 18;
        for x in 0..400 { display.draw_pixel(x, y_start, Pixel::Black, ctx); }
        let save_icon = if self.is_dirty { &self.save_icons[0] } else { &self.save_icons[1] };
        if let Some(bmp) = save_icon { self.draw_icon(display, bmp, 5, y_start + 3, ctx); }
        let filename = self.path.file_name().map(|n| n.to_string_lossy().to_string().to_uppercase()).unwrap_or_else(|| "UNTITLED.TXT".to_string());
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
        let max_width = 370.0; // Reduced to give scrollbar room
        let lines = self.get_wrapped_lines(max_width);
        
        match key {
            Key::Esc => return Action::Pop,
            Key::Ctrl('s') => { self.save_file(); }
            Key::Left => { if self.cursor_pos > 0 { self.cursor_pos -= 1; } }
            Key::Right => { if self.cursor_pos < self.content.len() { self.cursor_pos += 1; } }
            Key::Up => {
                if let Some(curr_idx) = lines.iter().position(|l| self.cursor_pos >= l.1 && self.cursor_pos <= l.1 + l.0.len()) {
                    if curr_idx > 0 {
                        let offset = self.cursor_pos - lines[curr_idx].1;
                        let prev = &lines[curr_idx - 1];
                        self.cursor_pos = prev.1 + offset.min(prev.0.len().saturating_sub(1));
                    }
                }
            }
            Key::Down => {
                if let Some(curr_idx) = lines.iter().position(|l| self.cursor_pos >= l.1 && self.cursor_pos <= l.1 + l.0.len()) {
                    if curr_idx < lines.len() - 1 {
                        let offset = self.cursor_pos - lines[curr_idx].1;
                        let next = &lines[curr_idx + 1];
                        self.cursor_pos = next.1 + offset.min(next.0.len().saturating_sub(1));
                    }
                }
            }
            Key::Char(c) => {
                self.content.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.is_dirty = true;
            }
            Key::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.content.remove(self.cursor_pos);
                    self.is_dirty = true;
                }
            }
            _ => {}
        }

        // Logic to update scroll_line_offset based on cursor position
        let line_height = (self.font_size * 1.2) as i32;
        let visible_lines = 180 / line_height;
        if let Some(cursor_line) = lines.iter().position(|l| self.cursor_pos >= l.1 && self.cursor_pos <= l.1 + l.0.len()) {
            if cursor_line < self.scroll_line_offset {
                self.scroll_line_offset = cursor_line;
            } else if cursor_line >= self.scroll_line_offset + visible_lines as usize {
                self.scroll_line_offset = cursor_line - (visible_lines as usize - 1);
            }
        }

        Action::None
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        let margin = 10;
        let max_width = 370.0;
        let line_height = (self.font_size * 1.2) as i32;
        let lines = self.get_wrapped_lines(max_width);
        let visible_lines = 180 / line_height;

        let mut cursor_line_idx = 99999; // Default out of range
        for (idx, line) in lines.iter().enumerate() {
            if self.cursor_pos >= line.1 && self.cursor_pos <= line.1 + line.0.len() {
                cursor_line_idx = idx;
                break;
            }
        }

        let mut draw_y = 30;
        for (idx, (text, start_pos)) in lines.iter().enumerate().skip(self.scroll_line_offset) {
            if draw_y > 210 { break; }
            self.renderer.draw_text_colored(display, text, margin, draw_y, self.font_size, Pixel::Black, ctx);

            if idx == cursor_line_idx {
                let relative_pos = self.cursor_pos - start_pos;
                let text_before = &text[..relative_pos.min(text.len())];
                let cursor_x = margin + self.renderer.calculate_width(text_before, self.font_size) as i32;
                let cursor_top = draw_y - (self.font_size * 0.8) as i32;
                for cy in cursor_top..draw_y {
                    if cy > 0 && cy < 218 && cursor_x < 400 { 
                        display.draw_pixel(cursor_x as usize, cy as usize, Pixel::Black, ctx); 
                    }
                }
            }
            draw_y += line_height;
        }

        self.draw_scrollbar(display, lines.len(), visible_lines as usize, ctx);
        self.draw_bottom_bar(display, ctx);
    }
}