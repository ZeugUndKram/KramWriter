use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::fs;
use std::path::{Path, PathBuf};

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
}

impl FileBrowserPage {
    pub fn new() -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        let icon_path = "/home/kramwriter/KramWriter/assets/FileBrowser";
        
        // Starting path
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
        };

        page.refresh_entries();
        page
    }

    fn refresh_entries(&mut self) {
        self.entries.clear();
        
        // 1. Add "Back" entry if not at base folder
        if self.current_directory != PathBuf::from("/home/kramwriter/") {
            if let Some(parent) = self.current_directory.parent() {
                self.entries.push(FileEntry {
                    name: String::from(".."),
                    is_dir: true,
                    size_kb: 0,
                    path: parent.to_path_buf(),
                });
            }
        }

        // 2. Read actual files from disk
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

        // 3. Sort: Directories first, then alphabetical
        self.entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    }

    fn draw_list_row(&self, display: &mut SharpDisplay, ctx: &Context, index: usize, y: i32, entry: &FileEntry) {
        let is_selected = self.selected_index == index;
        let row_height = 22;
        
        // 1. Draw Selection Background
        if is_selected {
            for sy in y..(y + row_height) {
                for sx in 0..400 {
                    display.draw_pixel(sx as usize, sy as usize, Pixel::Black, ctx);
                }
            }
        }

        // 2. Determine Icon and Color
        // If selected, we draw pixels as WHITE to show up against the black bar
        let draw_color = if is_selected { Pixel::White } else { Pixel::Black };
        
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

        // 3. Draw Filename
        let display_name = if entry.is_dir && entry.name != ".." {
            format!("/ {} /", entry.name.to_uppercase())
        } else if entry.name == ".." {
            String::from("/ ... /")
        } else {
            entry.name.clone()
        };

        self.renderer.draw_text_colored(display, &display_name, 35, y + 17, 18.0, draw_color, ctx);

        // 4. Draw Size (for files)
        if !entry.is_dir {
            let size_str = format!("{}KB", entry.size_kb);
            self.renderer.draw_text_colored(display, &size_str, 340, y + 17, 16.0, draw_color, ctx);
        }

        // 5. Bottom Separator Line (only if not selected, or it disappears)
        if !is_selected {
            for x in 0..400 {
                display.draw_pixel(x, (y + row_height - 1) as usize, Pixel::Black, ctx);
            }
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
}

impl Page for FileBrowserPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        match key {
            Key::Up => {
                if self.selected_index > 0 { self.selected_index -= 1; }
                Action::None
            }
            Key::Down => {
                if self.selected_index < self.entries.len() - 1 { self.selected_index += 1; }
                Action::None
            }
            Key::Char('\n') => {
                let selected = self.entries[self.selected_index].clone();
                if selected.is_dir {
                    self.current_directory = selected.path;
                    self.refresh_entries();
                    self.selected_index = 0;
                    Action::None
                } else {
                    println!("FILE SELECTED: {:?}", selected.path);
                    Action::None
                }
            }
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        // Header line
        for x in 0..400 { display.draw_pixel(x, 22, Pixel::Black, ctx); }
        
        // Header Icon & Path
        if let Some(bmp) = &self.home_icon {
            self.draw_icon_colored(display, bmp, 5, 4, Pixel::Black, ctx);
        }
        let path_display = self.current_directory.to_string_lossy().to_uppercase();
        self.renderer.draw_text_colored(display, &path_display, 35, 18, 20.0, Pixel::Black, ctx);

        // List entries
        let start_y = 23;
        let row_h = 22;
        for (i, entry) in self.entries.iter().enumerate() {
            let y_pos = start_y + (i as i32 * row_h);
            if y_pos < 220 { // Keep some space for the bottom bar
                self.draw_list_row(display, ctx, i, y_pos, entry);
            }
        }
    }
}