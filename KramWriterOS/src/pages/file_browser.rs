use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size_kb: u64,
    pub path: PathBuf,
}

pub struct FileBrowserPage {
    home_icon: Option<Bitmap>,
    back_icon: Option<Bitmap>,
    folder_icon: Option<Bitmap>,
    file_icon: Option<Bitmap>,
    renderer: FontRenderer,
    current_directory: PathBuf,
    entries: Vec<FileEntry>,
    selected_index: usize,
    scroll_offset: usize, // New: Tracks the first visible index
}

impl FileBrowserPage {
    pub fn new() -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        let icon_path = "/home/kramwriter/KramWriter/assets/FileBrowser";
        let start_dir = PathBuf::from("/home/kramwriter/folder/");
        
        let mut page = Self {
            home_icon: Bitmap::load(&format!("{}/icon_home.bmp", icon_path)).ok(),
            back_icon: Bitmap::load(&format!("{}/icon_back.bmp", icon_path)).ok(),
            folder_icon: Bitmap::load(&format!("{}/icon_folder.bmp", icon_path)).ok(),
            file_icon: Bitmap::load(&format!("{}/icon_file.bmp", icon_path)).ok(),
            renderer,
            current_directory: start_dir,
            entries: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
        };

        page.refresh_entries();
        page
    }

    fn refresh_entries(&mut self) {
        self.entries.clear();
        if self.current_directory != PathBuf::from("/") {
            if let Some(parent) = self.current_directory.parent() {
                self.entries.push(FileEntry {
                    name: String::from(".."),
                    is_dir: true,
                    size_kb: 0,
                    path: parent.to_path_buf(),
                });
            }
        }

        if let Ok(read_dir) = fs::read_dir(&self.current_directory) {
            for entry in read_dir.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    self.entries.push(FileEntry {
                        name: entry.file_name().to_string_lossy().into_owned(),
                        is_dir: metadata.is_dir(),
                        size_kb: metadata.len() / 1024,
                        path: entry.path(),
                    });
                }
            }
        }
        self.entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    }

    /// Truncates path from the left if it's too long
    fn format_header_path(&self) -> String {
        let path_str = self.current_directory.to_string_lossy().to_uppercase();
        let max_chars = 35; // Adjust based on font size/screen width
        if path_str.len() > max_chars {
            format!("...{}", &path_str[path_str.len() - max_chars..])
        } else {
            path_str
        }
    }

    fn draw_icon_colored(&self, display: &mut SharpDisplay, bmp: &Bitmap, x_off: usize, y_off: usize, color: Pixel, ctx: &Context) {
        for y in 0..bmp.height {
            for x in 0..bmp.width {
                if bmp.pixels[y * bmp.width + x] == Pixel::Black {
                    let sx = x + x_off;
                    let sy = y + y_off;
                    if sx < 400 && sy < 240 {
                        display.draw_pixel(sx, sy, color, ctx);
                    }
                }
            }
        }
    }

    fn draw_list_row(&self, display: &mut SharpDisplay, ctx: &Context, index: usize, y: i32, entry: &FileEntry) {
        let is_selected = self.selected_index == index;
        let row_height = 22;
        let draw_color = if is_selected { Pixel::White } else { Pixel::Black };
        
        if is_selected {
            for sy in y..(y + row_height) {
                for sx in 0..400 {
                    display.draw_pixel(sx as usize, sy as usize, Pixel::Black, ctx);
                }
            }
        }

        let icon = if entry.name == ".." { &self.back_icon } 
                   else if entry.is_dir { &self.folder_icon } 
                   else { &self.file_icon };

        if let Some(bmp) = icon {
            self.draw_icon_colored(display, bmp, 5, (y + 3) as usize, draw_color, ctx);
        }

        let display_name = if entry.is_dir && entry.name != ".." {
            format!("/ {} /", entry.name.to_uppercase())
        } else if entry.name == ".." {
            String::from("/ ... /")
        } else {
            entry.name.clone()
        };

        self.renderer.draw_text_colored(display, &display_name, 35, y + 17, 18.0, draw_color, ctx);

        if !entry.is_dir {
            let size_str = format!("{}KB", entry.size_kb);
            self.renderer.draw_text_colored(display, &size_str, 340, y + 17, 16.0, draw_color, ctx);
        }
    }
}

impl Page for FileBrowserPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    // Scroll up if selection goes above visible area
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
                Action::None
            }
            Key::Down => {
                if self.selected_index < self.entries.len() - 1 {
                    self.selected_index += 1;
                    // Scroll down if selection goes below visible area (8 rows visible)
                    if self.selected_index >= self.scroll_offset + 8 {
                        self.scroll_offset = self.selected_index - 7;
                    }
                }
                Action::None
            }
            Key::Char('\n') => {
                let selected = self.entries[self.selected_index].clone();
                if selected.is_dir {
                    self.current_directory = selected.path;
                    self.refresh_entries();
                    self.selected_index = 0;
                    self.scroll_offset = 0;
                    Action::None
                } else {
                    println!("Opening: {:?}", selected.path);
                    Action::None
                }
            }
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // 1. Header Line
        for x in 0..400 { display.draw_pixel(x, 22, Pixel::Black, ctx); }
        
        // 2. Path Header with Truncation
        if let Some(bmp) = &self.home_icon {
            self.draw_icon_colored(display, bmp, 5, 4, Pixel::Black, ctx);
        }
        let header_path = self.format_header_path();
        self.renderer.draw_text_colored(display, &header_path, 35, 18, 20.0, Pixel::Black, ctx);

        // 3. Draw Visible Entries
        let start_y = 23;
        let row_h = 22;
        let max_visible = 8; // Adjust based on your bottom bar height

        for i in 0..max_visible {
            let entry_idx = i + self.scroll_offset;
            if entry_idx < self.entries.len() {
                let y_pos = start_y + (i as i32 * row_h);
                self.draw_list_row(display, ctx, entry_idx, y_pos, &self.entries[entry_idx]);
            }
        }
    }
}