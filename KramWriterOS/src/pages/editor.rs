use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;
use std::fs;

// Timezone and Time imports
use chrono::{Utc, FixedOffset}; // Add FixedOffset to your imports
use chrono_tz::Tz;

// --- LAYOUT STRUCTURE ---
#[derive(Debug, Clone)]
struct VisualLine {
    text: String,       
    start_index: usize, 
    len: usize,         
    is_hard_break: bool,
}

impl VisualLine {
    fn end_index(&self) -> usize {
        self.start_index + self.len
    }
}

pub struct EditorPage {
    path: PathBuf,
    content: String,
    cursor_pos: usize,
    scroll_line_offset: usize,
    target_cursor_x: Option<i32>, 
    is_dirty: bool,
    renderer: FontRenderer,
    font_size: f32,
    top_margin: i32,
    save_icons: [Option<Bitmap>; 2],
    wifi_icons: [Option<Bitmap>; 5],
    weather_icons: Vec<Option<Bitmap>>,
}

impl EditorPage {
    pub fn new(path: PathBuf) -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/Inter_28pt-Medium.ttf");
        let asset_path = "/home/kramwriter/KramWriter/assets/Writing";
        let content = fs::read_to_string(&path).unwrap_or_default();
        let len = content.len();

        Self {
            path,
            content,
            cursor_pos: len,
            scroll_line_offset: 0,
            target_cursor_x: None,
            is_dirty: false,
            renderer,
            font_size: 22.0,
            top_margin: 25,
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

    fn cursor_is_on_this_line(&self, line: &VisualLine) -> bool {
        if self.cursor_pos >= line.start_index && self.cursor_pos < line.end_index() {
            true
        } else if self.cursor_pos == line.end_index() {
            line.is_hard_break || line.len == 0 || self.cursor_pos == self.content.len()
        } else {
            false
        }
    }

    fn build_layout(&self, max_width: f32) -> Vec<VisualLine> {
        let mut visual_lines = Vec::new();
        let mut current_abs_index = 0;
        let paragraphs: Vec<&str> = self.content.split('\n').collect();
        let total_paras = paragraphs.len();

        for (i, para) in paragraphs.iter().enumerate() {
            let is_last_para = i == total_paras - 1;
            
            if para.is_empty() {
                visual_lines.push(VisualLine {
                    text: String::new(),
                    start_index: current_abs_index,
                    len: 0,
                    is_hard_break: !is_last_para,
                });
                current_abs_index += 1;
                continue;
            }

            let mut current_line = String::new();
            let mut line_start_rel = 0;
            
            for word in para.split_inclusive(' ') {
                let test_line = format!("{}{}", current_line, word);
                let w = self.renderer.calculate_width(&test_line, self.font_size);
                
                if (w as f32) > max_width && !current_line.is_empty() {
                    let len = current_line.len();
                    visual_lines.push(VisualLine {
                        text: current_line,
                        start_index: current_abs_index + line_start_rel,
                        len,
                        is_hard_break: false,
                    });
                    line_start_rel += len;
                    current_line = word.to_string();
                } else {
                    current_line = test_line;
                }
            }

            let len = current_line.len();
            visual_lines.push(VisualLine {
                text: current_line,
                start_index: current_abs_index + line_start_rel,
                len,
                is_hard_break: !is_last_para,
            });

            current_abs_index += para.len();
            if !is_last_para { current_abs_index += 1; }
        }
        visual_lines
    }

    fn move_cursor_vertical(&mut self, direction: i32, layout: &[VisualLine]) {
        let current_line_idx = layout.iter().position(|l| self.cursor_is_on_this_line(l));

        if let Some(idx) = current_line_idx {
            let next_idx = idx as i32 + direction;
            if next_idx >= 0 && next_idx < layout.len() as i32 {
                let current_line = &layout[idx];
                let target_line = &layout[next_idx as usize];

                let target_x = if let Some(tx) = self.target_cursor_x {
                    tx
                } else {
                    let offset = self.cursor_pos.saturating_sub(current_line.start_index);
                    let text_before = &current_line.text[..offset.min(current_line.text.len())];
                    self.renderer.calculate_width(text_before, self.font_size) as i32
                };

                let mut best_offset = 0;
                let mut min_diff = i32::MAX;
                for i in 0..=target_line.text.len() {
                    let sub = &target_line.text[..i];
                    let w = self.renderer.calculate_width(sub, self.font_size) as i32;
                    let diff = (w - target_x).abs();
                    if diff < min_diff {
                        min_diff = diff;
                        best_offset = i;
                    }
                }
                self.cursor_pos = target_line.start_index + best_offset;
                self.target_cursor_x = Some(target_x);
            }
        }
    }

    fn get_word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

   fn draw_bottom_bar(&self, display: &mut SharpDisplay, ctx: &Context) {
        let y_start = 218;
        let y_text = y_start as i32 + 18;

        // Draw the separator line
        for x in 0..400 { 
            display.draw_pixel(x, y_start, Pixel::Black, ctx); 
        }
        
        // 1. Save Icon (Left)
        let save_icon = if self.is_dirty { &self.save_icons[0] } else { &self.save_icons[1] };
        if let Some(bmp) = save_icon { 
            self.draw_icon(display, bmp, 5, y_start + 3, ctx); 
        }
        
        // 2. Filename (Upper Case)
        let filename = self.path.file_name()
            .map(|n| n.to_string_lossy().to_string().to_uppercase())
            .unwrap_or_else(|| "UNTITLED.TXT".to_string());
        self.renderer.draw_text_colored(display, &filename, 28, y_text, 18.0, Pixel::Black, ctx);
        
        // 3. Word Count (Center-ish)
        let w_count = format!("W:{}", self.get_word_count());
        self.renderer.draw_text_colored(display, &w_count, 180, y_text, 18.0, Pixel::Black, ctx);
        
        // 4. --- DYNAMIC TIMEZONE LOGIC ---
        // Parse offset string (e.g. "-5" or "3.5") to seconds
        let offset_hours = ctx.timezone.parse::<f32>().unwrap_or(0.0);
        let offset_seconds = (offset_hours * 3600.0) as i32;
        
        let time_str = if let Some(offset) = FixedOffset::east_opt(offset_seconds) {
            let now = Utc::now().with_timezone(&offset);
            now.format("%H:%M").to_string()
        } else {
            Utc::now().format("%H:%M").to_string()
        };
        
        self.renderer.draw_text_colored(display, &time_str, 305, y_text, 18.0, Pixel::Black, ctx);
        
        // 5. Weather Icon
        let weather_idx = (ctx.status.weather_icon as usize).min(self.weather_icons.len() - 1);
        if let Some(bmp) = &self.weather_icons[weather_idx] { 
            self.draw_icon(display, bmp, 348, y_start + 3, ctx); 
        }

        // 6. WiFi Strength Icon
        let wifi_idx = (ctx.status.wifi_strength as usize).min(4);
        if let Some(bmp) = &self.wifi_icons[wifi_idx] { 
            self.draw_icon(display, bmp, 372, y_start + 3, ctx); 
        }
    }

    fn draw_icon(&self, display: &mut SharpDisplay, bmp: &Bitmap, x_off: usize, y_off: usize, ctx: &Context) {
        for y in 0..bmp.height {
            for x in 0..bmp.width {
                if bmp.pixels[y * bmp.width + x] == Pixel::Black {
                    let sx = x + x_off;
                    let sy = y + y_off;
                    if sx < 400 && sy < 240 { display.draw_pixel(sx, sy, Pixel::Black, ctx); }
                }
            }
        }
    }

    fn draw_scrollbar(&self, display: &mut SharpDisplay, total_lines: usize, visible_count: usize, ctx: &Context) {
        if total_lines <= visible_count { return; }
        let track_top = self.top_margin;
        let track_bottom = 210;
        let track_h = track_bottom - track_top;
        let thumb_h = (((visible_count as f32 / total_lines as f32) * track_h as f32) as i32).max(10);
        let scrollable_dist = (total_lines - visible_count) as f32;
        let thumb_y = track_top + ((self.scroll_line_offset as f32 / scrollable_dist) * (track_h - thumb_h) as f32) as i32;
        for y in thumb_y..(thumb_y + thumb_h) {
            for x in 395..399 { display.draw_pixel(x, y as usize, Pixel::Black, ctx); }
        }
    }
}

impl Page for EditorPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        let max_width = 370.0;
        let layout = self.build_layout(max_width);
        let line_height = (self.font_size * 1.2) as i32;
        let visible_lines = ((218 - self.top_margin) / line_height) as usize;

        match key {
            Key::Esc => return Action::Pop,
            Key::Ctrl('s') => { let _ = fs::write(&self.path, &self.content); self.is_dirty = false; }
            Key::Alt('+') | Key::Alt('=') => { if self.font_size < 60.0 { self.font_size += 2.0; } }
            Key::Alt('-') => { if self.font_size > 10.0 { self.font_size -= 2.0; } }
            Key::Left => { if self.cursor_pos > 0 { self.cursor_pos -= 1; } self.target_cursor_x = None; }
            Key::Right => { if self.cursor_pos < self.content.len() { self.cursor_pos += 1; } self.target_cursor_x = None; }
            Key::Up => self.move_cursor_vertical(-1, &layout),
            Key::Down => self.move_cursor_vertical(1, &layout),
            Key::Char(c) => {
                self.content.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.is_dirty = true;
                self.target_cursor_x = None;
            }
            Key::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.content.remove(self.cursor_pos);
                    self.is_dirty = true;
                    self.target_cursor_x = None;
                }
            }
            _ => {}
        }

        let layout = self.build_layout(max_width); 
        let current_line_idx = layout.iter().position(|l| self.cursor_is_on_this_line(l)).unwrap_or(0);

        if current_line_idx < self.scroll_line_offset {
            self.scroll_line_offset = current_line_idx;
        } else if current_line_idx >= self.scroll_line_offset + visible_lines {
            self.scroll_line_offset = current_line_idx - (visible_lines.saturating_sub(1));
        }

        Action::None
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        let margin = 10;
        let max_width = 370.0;
        let layout = self.build_layout(max_width);
        let line_height = (self.font_size * 1.2) as i32;
        let visible_lines = ((218 - self.top_margin) / line_height) as usize;

        let mut draw_y = self.top_margin;

        for (_idx, line) in layout.iter().enumerate().skip(self.scroll_line_offset) {
            if draw_y + line_height > 218 { break; }

            if !line.text.is_empty() {
                self.renderer.draw_text_colored(display, &line.text, margin, draw_y + (self.font_size as i32), self.font_size, Pixel::Black, ctx);
            }

            if self.cursor_is_on_this_line(line) {
                let offset = self.cursor_pos.saturating_sub(line.start_index);
                let sub_text = &line.text[..offset.min(line.text.len())];
                let cursor_x = margin + self.renderer.calculate_width(sub_text, self.font_size) as i32;
                
                let cursor_height = self.font_size as i32;
                for cy in draw_y..(draw_y + cursor_height) {
                    if cy < 218 && cursor_x < 398 {
                        display.draw_pixel(cursor_x as usize, cy as usize, Pixel::Black, ctx);
                        display.draw_pixel((cursor_x + 1) as usize, cy as usize, Pixel::Black, ctx);
                    }
                }
            }
            draw_y += line_height;
        }

        self.draw_scrollbar(display, layout.len(), visible_lines, ctx);
        self.draw_bottom_bar(display, ctx);
    }
}