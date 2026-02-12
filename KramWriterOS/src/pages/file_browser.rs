use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::fs;
use std::path::PathBuf;

#[derive(PartialEq)]
pub enum BrowserFocus {
    List,
    Footer,
}

#[derive(Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size_kb: u64,
    pub path: PathBuf,
}

pub struct FileBrowserPage {
    home_icon: Option<Bitmap>,
    back_icon: Option<Bitmap>, // This will now correctly load icon_up.bmp
    folder_icon: Option<Bitmap>,
    file_icon: Option<Bitmap>,
    footer_variants: [Option<Bitmap>; 4],
    renderer: FontRenderer,
    current_directory: PathBuf,
    entries: Vec<FileEntry>,
    selected_index: usize,
    scroll_offset: usize,
    focus: BrowserFocus,
    footer_index: usize,
}

impl FileBrowserPage {
    pub fn new() -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        let asset_path = "/home/kramwriter/KramWriter/assets/FileBrowser";
        
        let footer_variants = [
            Bitmap::load(&format!("{}/bottom_bar_3.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/bottom_bar_4.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/bottom_bar_5.bmp", asset_path)).ok(),
            Bitmap::load(&format!("{}/bottom_bar_6.bmp", asset_path)).ok(),
        ];

        let mut page = Self {
            home_icon: Bitmap::load(&format!("{}/icon_home.bmp", asset_path)).ok(),
            // FIXED: Changed filename to match your 'ls' output
            back_icon: Bitmap::load(&format!("{}/icon_up.bmp", asset_path)).ok(),
            folder_icon: Bitmap::load(&format!("{}/icon_folder.bmp", asset_path)).ok(),
            file_icon: Bitmap::load(&format!("{}/icon_file.bmp", asset_path)).ok(),
            footer_variants,
            renderer,
            current_directory: PathBuf::from("/home/kramwriter/folder"),
            entries: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            focus: BrowserFocus::List,
            footer_index: 0,
        };

        page.refresh_entries();
        page
    }

    fn refresh_entries(&mut self) {
        self.entries.clear();
        
        let home_base = "/home/kramwriter";
        let current_str = self.current_directory.to_string_lossy().to_string();

        // Show "Back" if we are deeper than the home directory
        if current_str.len() > home_base.len() {
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

        // Sort: ".." always first, then Folders, then alphabetical
        self.entries.sort_by(|a, b| {
            if a.name == ".." { return std::cmp::Ordering::Less; }
            if b.name == ".." { return std::cmp::Ordering::Greater; }
            b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
    }

    fn format_header_path(&self) -> String {
        let full_path = self.current_directory.to_string_lossy().to_string();
        let home_prefix = "/home/kramwriter";
        
        let mut display_path = if full_path.starts_with(home_prefix) {
            full_path.replacen(home_prefix, "", 1)
        } else {
            full_path
        };

        // If at home, the string is empty, so we just show the Home icon
        if display_path.is_empty() || display_path == "/" { 
            display_path = String::from(""); 
        }
        
        display_path = display_path.to_uppercase();

        let max_chars = 30; 
        if display_path.len() > max_chars {
            format!("...{}", &display_path[display_path.len() - max_chars..])
        } else {
            display_path
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
        let is_selected = self.focus == BrowserFocus::List && self.selected_index == index;
        let row_height = 22;
        let draw_color = if is_selected { Pixel::White } else { Pixel::Black };
        
        if is_selected {
            for sy in y..(y + row_height) {
                for sx in 0..400 {
                    display.draw_pixel(sx as usize, sy as usize, Pixel::Black, ctx);
                }
            }
        }

        let icon = if entry.name == ".." { 
            &self.back_icon 
        } else if entry.is_dir { 
            &self.folder_icon 
        } else { 
            &self.file_icon 
        };

        if let Some(bmp) = icon {
            self.draw_icon_colored(display, bmp, 5, (y + 3) as usize, draw_color, ctx);
        }

        let display_name = if entry.name == ".." {
            String::from("/ ... /")
        } else if entry.is_dir {
            format!("/ {} /", entry.name.to_uppercase())
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
        match self.focus {
            BrowserFocus::List => match key {
                Key::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                        if self.selected_index < self.scroll_offset { self.scroll_offset = self.selected_index; }
                    }
                    Action::None
                }
                Key::Down => {
                    if self.selected_index < self.entries.len() - 1 {
                        self.selected_index += 1;
                        if self.selected_index >= self.scroll_offset + 8 { self.scroll_offset = self.selected_index - 7; }
                    }
                    Action::None
                }
                Key::Left | Key::Right => {
                    self.focus = BrowserFocus::Footer;
                    self.footer_index = if key == Key::Left { 0 } else { 1 };
                    Action::None
                }
                Key::Char('\n') => {
                    if let Some(entry) = self.entries.get(self.selected_index) {
                        let selected = entry.clone();
                        if selected.is_dir {
                            self.current_directory = selected.path;
                            self.refresh_entries();
                            self.selected_index = 0;
                            self.scroll_offset = 0;
                        }
                    }
                    Action::None
                }
                Key::Esc => Action::Pop,
                _ => Action::None,
            },
            BrowserFocus::Footer => match key {
                Key::Up | Key::Down => {
                    self.focus = BrowserFocus::List;
                    Action::None
                }
                Key::Left => {
                    if self.footer_index > 0 { self.footer_index -= 1; }
                    Action::None
                }
                Key::Right => {
                    if self.footer_index < 2 { self.footer_index += 1; }
                    Action::None
                }
                Key::Char('\n') => {
                    match self.footer_index {
                        0 => Action::Pop,
                        1 | 2 => Action::None,
                        _ => Action::None
                    }
                }
                _ => Action::None,
            }
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // 1. Header (Black line separator)
        for x in 0..400 { display.draw_pixel(x, 22, Pixel::Black, ctx); }
        
        // Home icon at 2px from left and top
        if let Some(bmp) = &self.home_icon {
            self.draw_icon_colored(display, bmp, 2, 2, Pixel::Black, ctx);
        }
        
        let header_path = self.format_header_path();
        // Path starts after the home icon area
        self.renderer.draw_text_colored(display, &header_path, 35, 18, 20.0, Pixel::Black, ctx);

        // 2. Visible List
        let start_y = 23;
        for i in 0..8 {
            let entry_idx = i + self.scroll_offset;
            if entry_idx < self.entries.len() {
                self.draw_list_row(display, ctx, entry_idx, start_y + (i as i32 * 22), &self.entries[entry_idx]);
            }
        }

        // 3. Footer (Aligned to bottom)
        let footer_idx = if self.focus == BrowserFocus::List { 0 } else { self.footer_index + 1 };
        if let Some(bmp) = &self.footer_variants[footer_idx] {
            let y_start = 216; 
            for y in 0..bmp.height {
                let sy = y as i32 + y_start;
                if sy < 240 {
                    for x in 0..bmp.width.min(400) {
                        if bmp.pixels[y * bmp.width + x] == Pixel::Black {
                            display.draw_pixel(x, sy as usize, Pixel::Black, ctx);
                        }
                    }
                }
            }
        }
    }
}