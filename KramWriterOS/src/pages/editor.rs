use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::path::PathBuf;
use std::fs;
use chrono::Local;

// --- 1. THE LAYOUT STRUCTURE ---
// This tells us exactly where a line starts and ends in the main string.
#[derive(Debug, Clone)]
struct VisualLine {
    text: String,       // The actual text to draw (e.g., "Hello world")
    start_index: usize, // Index in 'content' where this line begins
    len: usize,         // Length of this line in bytes
    is_hard_break: bool,// Does this line end because of a \n?
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
    target_cursor_x: Option<i32>, // Remembers your horizontal position when moving Up/Down
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

    // --- 2. THE LAYOUT ENGINE ---
    // This turns the raw String into a list of lines that fit the screen.
    fn build_layout(&self, max_width: f32) -> Vec<VisualLine> {
        let mut visual_lines = Vec::new();
        let mut current_abs_index = 0;

        // Split by logical paragraphs (hard newlines)
        let paragraphs: Vec<&str> = self.content.split('\n').collect();
        let total_paras = paragraphs.len();

        for (i, para) in paragraphs.iter().enumerate() {
            let is_last_para = i == total_paras - 1;
            
            // Case 1: Empty Paragraph (Double Newline or Empty File)
            if para.is_empty() {
                visual_lines.push(VisualLine {
                    text: String::new(),
                    start_index: current_abs_index,
                    len: 0,
                    is_hard_break: !is_last_para,
                });
                current_abs_index += 1; // Skip the \n char
                continue;
            }

            // Case 2: Wrapping Logic
            let mut current_line = String::new();
            let mut line_start_rel = 0;
            
            // We split by spaces to wrap words, but keep the spaces attached to words logic usually,
            // strict split_inclusive works well for simple wrapping.
            for word in para.split_inclusive(' ') {
                let test_line = format!("{}{}", current_line, word);
                let w = self.renderer.calculate_width(&test_line, self.font_size);
                
                if (w as f32) > max_width && !current_line.is_empty() {
                    // Current line is full, push it
                    let len = current_line.len();
                    visual_lines.push(VisualLine {
                        text: current_line,
                        start_index: current_abs_index + line_start_rel,
                        len,
                        is_hard_break: false, // Soft wrap
                    });
                    line_start_rel += len;
                    current_line = word.to_string();
                } else {
                    current_line = test_line;
                }
            }

            // Push the remainder of the paragraph
            let len = current_line.len();
            visual_lines.push(VisualLine {
                text: current_line,
                start_index: current_abs_index + line_start_rel,
                len,
                is_hard_break: !is_last_para,
            });

            current_abs_index += para.len();
            if !is_last_para { current_abs_index += 1; } // Skip the \n
        }

        visual_lines
    }

    // --- 3. CURSOR MOVEMENT LOGIC ---
    fn move_cursor_vertical(&mut self, direction: i32, layout: &[VisualLine]) {
        // Find which line the cursor is currently on
        let current_line_idx = layout.iter().position(|l| {
            self.cursor_pos >= l.start_index && self.cursor_pos <= l.end_index()
        });

        if let Some(idx) = current_line_idx {
            let next_idx = idx as i32 + direction;
            
            // Bounds check
            if next_idx >= 0 && next_idx < layout.len() as i32 {
                let current_line = &layout[idx];
                let target_line = &layout[next_idx as usize];

                // 1. Calculate current visual X (if not already remembered)
                let target_x = if let Some(tx) = self.target_cursor_x {
                    tx
                } else {
                    let offset = self.cursor_pos - current_line.start_index;
                    let safe_offset = offset.min(current_line.text.len());
                    let text_before = &current_line.text[..safe_offset];
                    self.renderer.calculate_width(text_before, self.font_size) as i32
                };

                // 2. Find closest char in target line to that X
                let mut best_offset = 0;
                let mut min_diff = i32::MAX;

                // Scan through target line to find closest visual match
                // (Optimization: Binary search is better, linear is fine for <100 chars)
                for i in 0..=target_line.text.len() {
                    let sub = &target_line.text[..i];
                    let w = self.renderer.calculate_width(sub, self.font_size) as i32;
                    let diff = (w - target_x).abs();
                    if diff < min_diff {
                        min_diff = diff;
                        best_offset = i;
                    }
                }

                // 3. Move cursor
                self.cursor_pos = target_line.start_index + best_offset;
                self.target_cursor_x = Some(target_x); // Remember desired X column
            }
        }
    }

    fn get_word_count(&self) -> usize {
        self.content.split_whitespace().count()
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
        
        let time_str = Local::now().format("%H:%M").to_string();
        self.renderer.draw_text_colored(display, &time_str, 305, y_text, 18.0, Pixel::Black, ctx);
        
        if let Some(bmp) = &self.weather_icons[0] { self.draw_icon(display, bmp, 348, y_start + 3, ctx); }
        if let Some(bmp) = &self.wifi_icons[4] { self.draw_icon(display, bmp, 372, y_start + 3, ctx); }
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
        let track_top = 30;
        let track_bottom = 210;
        let track_h = track_bottom - track_top;
        let thumb_h = ((visible_count as f32 / total_lines as f32) * track_h as f32) as i32;
        let thumb_h = thumb_h.max(15);
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
        let visible_lines = 180 / line_height;

        match key {
            Key::Esc => return Action::Pop,
            Key::Ctrl('s') => { let _ = fs::write(&self.path, &self.content); self.is_dirty = false; }
            Key::Ctrl('+') | Key::Ctrl('=') => { if self.font_size < 40.0 { self.font_size += 2.0; } }
            Key::Ctrl('-') => { if self.font_size > 12.0 { self.font_size -= 2.0; } }

            // Navigation
            Key::Left => { 
                if self.cursor_pos > 0 { self.cursor_pos -= 1; }
                self.target_cursor_x = None; // Reset memory when moving manually
            }
            Key::Right => { 
                if self.cursor_pos < self.content.len() { self.cursor_pos += 1; }
                self.target_cursor_x = None;
            }
            Key::Up => self.move_cursor_vertical(-1, &layout),
            Key::Down => self.move_cursor_vertical(1, &layout),

            // Typing
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
            Key::Char('\n') => {
                self.content.insert(self.cursor_pos, '\n');
                self.cursor_pos += 1;
                self.is_dirty = true;
                self.target_cursor_x = None;
            }
            _ => {}
        }

        // --- VIEWPORT SCROLLING ---
        // Recalculate layout in case text changed to update scroll
        let layout = self.build_layout(max_width); 
        let current_line_idx = layout.iter().position(|l| {
            self.cursor_pos >= l.start_index && self.cursor_pos <= l.end_index()
        }).unwrap_or(layout.len().saturating_sub(1));

        if current_line_idx < self.scroll_line_offset {
            self.scroll_line_offset = current_line_idx;
        } else if current_line_idx >= self.scroll_line_offset + visible_lines as usize {
            self.scroll_line_offset = current_line_idx - (visible_lines as usize - 1);
        }

        Action::None
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        let margin = 10;
        let max_width = 370.0;
        let line_height = (self.font_size * 1.2) as i32;
        let layout = self.build_layout(max_width);
        let visible_lines = 180 / line_height;

        let mut draw_y = 30;

        for (idx, line) in layout.iter().enumerate().skip(self.scroll_line_offset) {
            if draw_y > 210 { break; }

            // Draw Text
            if !line.text.is_empty() {
                // Remove \n for rendering safety, though our build_layout should handle it
                let clean = line.text.replace('\n', ""); 
                self.renderer.draw_text_colored(display, &clean, margin, draw_y, self.font_size, Pixel::Black, ctx);
            }

            // Draw Cursor
            // Check if cursor is strictly inside this line range
            // inclusive start, inclusive end (because cursor can be AFTER the last char)
            if self.cursor_pos >= line.start_index && self.cursor_pos <= line.end_index() {
                // Exception: If this line ends in a hard break (\n), the cursor at the very end
                // conceptually belongs to the NEXT line (the empty new line).
                // BUT, build_layout creates a new empty VisualLine for that.
                // So we only skip if cursor == end_index AND it's a hard break 
                // AND it's not the very last EOF position.
                let is_cursor_here = if line.is_hard_break && self.cursor_pos == line.end_index() {
                   false // Cursor is actually on the start of the next line
                } else {
                   true
                };

                // Special case: EOF cursor position always renders on the last visual line component
                let is_eof = self.cursor_pos == self.content.len();
                let show_here = is_cursor_here || (is_eof && idx == layout.len() - 1);

                if show_here {
                    let offset = self.cursor_pos - line.start_index;
                    let safe_len = line.text.len();
                    let safe_offset = offset.min(safe_len);
                    
                    let sub_text = &line.text[..safe_offset];
                    let cursor_x = margin + self.renderer.calculate_width(sub_text, self.font_size) as i32;
                    let cursor_top = draw_y - (self.font_size * 0.8) as i32;
                    
                    for cy in cursor_top..draw_y {
                        if cy > 0 && cy < 218 && cursor_x < 400 {
                            display.draw_pixel(cursor_x as usize, cy as usize, Pixel::Black, ctx);
                        }
                    }
                }
            }

            draw_y += line_height;
        }

        self.draw_scrollbar(display, layout.len(), visible_lines as usize, ctx);
        self.draw_bottom_bar(display, ctx);
    }
}