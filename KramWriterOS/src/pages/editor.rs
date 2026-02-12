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
    cursor_pos: usize,         // Character index in the string
    scroll_line_offset: usize,    // Which line of the wrapped text is at the top
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

        let content = fs::read_to_string(&path).unwrap_or_default();
        let initial_cursor = content.len();

        Self {
            path,
            content,
            cursor_pos: initial_cursor,
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

    fn draw_scrollbar(&self, display: &mut SharpDisplay, total_lines: usize, visible_count: usize, ctx: &Context) {
        if total_lines <= visible_count { return; }

        let bar_x = 394;
        let view_top = 30;
        let view_bottom = 210;
        let view_height = view_bottom - view_top;

        let bar_height = (((visible_count as f32 / total_lines as f32) * view_height as f32) as usize).max(15);
        let max_scroll = total_lines - visible_count;
        let scroll_pct = self.scroll_line_offset as f32 / max_scroll as f32;
        let bar_y = view_top + (scroll_pct * (view_height - bar_height as i32) as f32) as i32;

        for y in bar_y..(bar_y + bar_height as i32) {
            for x in bar_x..(bar_x + 4) {
                if x < 400 { display.draw_pixel(x as usize, y as usize, Pixel::Black, ctx); }
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
            Key::Esc => return Action::Pop,
            Key::Ctrl('s') => self.save_file(),
            Key::Ctrl('+') | Key::Ctrl('=') => { if self.font_size < 40.0 { self.font_size += 2.0; } }
            Key::Ctrl('-') => { if self.font_size > 12.0 { self.font_size -= 2.0; } }
            
            Key::Left => { if self.cursor_pos > 0 { self.cursor_pos -= 1; } }
            Key::Right => { if self.cursor_pos < self.content.len() { self.cursor_pos += 1; } }
            
            Key::Char('\n') => {
                self.content.insert(self.cursor_pos, '\n');
                self.cursor_pos += 1;
                self.is_dirty = true;
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
        Action::None
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        let margin = 10;
        let max_width = 375.0; 
        let line_height = (self.font_size * 1.2) as i32;
        let visible_area_height = 180;
        let max_lines_visible = visible_area_height / line_height;

        let mut all_wrapped_lines = Vec::new();
        let mut cursor_line_idx = 0;
        let mut cursor_x_pos = margin;
        let mut chars_processed = 0;

        // Process Wrap and find Cursor X/Y simultaneously
        for paragraph in self.content.split_inclusive('\n') {
            let mut current_line = String::new();
            
            for word in paragraph.split_inclusive(' ') {
                let test_line = format!("{}{}", current_line, word);
                let test_width = self.renderer.calculate_width(&test_line, self.font_size);

                if (test_width as f32) > max_width && !current_line.is_empty() {
                    all_wrapped_lines.push(current_line.clone());
                    chars_processed += current_line.len();
                    current_line = word.to_string();
                } else {
                    current_line = test_line;
                }

                // Check if cursor is in this segment
                if self.cursor_pos >= chars_processed && self.cursor_pos <= chars_processed + current_line.len() {
                    cursor_line_idx = all_wrapped_lines.len();
                    let relative_pos = self.cursor_pos - chars_processed;
                    let text_before_cursor = &current_line[..relative_pos];
                    cursor_x_pos = margin + self.renderer.calculate_width(text_before_cursor, self.font_size) as i32;
                }
            }
            all_wrapped_lines.push(current_line.clone());
            chars_processed += current_line.len();
        }

        // Logic for auto-scrolling the viewport to the cursor
        let mut scroll_offset = self.scroll_line_offset;
        if cursor_line_idx < scroll_offset {
            scroll_offset = cursor_line_idx;
        } else if cursor_line_idx >= scroll_offset + max_lines_visible as usize {
            scroll_offset = cursor_line_idx - (max_lines_visible as usize - 1);
        }

        // Final Rendering
        let mut draw_y = 30;
        for (idx, line) in all_wrapped_lines.iter().enumerate().skip(scroll_offset) {
            if draw_y > 210 { break; }
            
            self.renderer.draw_text_colored(display, line, margin, draw_y, self.font_size, Pixel::Black, ctx);

            let cursor_top = draw_y - (self.font_size * 0.8) as i32;

            for cy in cursor_top..(draw_y + 2) {
                if cy > 0 && cy < 218 {
                    display.draw_pixel(cursor_x_pos as usize, cy as usize, Pixel::Black, ctx);
                }
            }
            draw_y += line_height;
        }

        self.draw_scrollbar(display, all_wrapped_lines.len(), max_lines_visible as usize, ctx);
        self.draw_bottom_bar(display, ctx);
    }
}