use crate::pages::{Page, Action};
use crate::context::{Context, KeyboardLayout};
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use termion::event::Key;
use rpi_memory_display::Pixel;
use crate::pages::simplenote_setup::SimpleNoteSetupPage;
use crate::pages::timezone::TimezonePage;

// Swapped "drive" for "simplenote"
const SETTINGS_OPTIONS: [&str; 5] = ["timezone", "location", "darkmode", "simplenote", "keyboard"];

pub struct SettingsPage {
    current_index: usize,
    images: Vec<[Option<Bitmap>; 4]>,
}

impl SettingsPage {
    pub fn new() -> Self {
        let mut images = Vec::new();

        for option in SETTINGS_OPTIONS.iter() {
            let mut variants = [None, None, None, None];

            if *option == "darkmode" {
                let paths = [
                    "/home/kramwriter/KramWriter/assets/Settings/darkmode_0.bmp",
                    "/home/kramwriter/KramWriter/assets/Settings/darkmode_1.bmp",
                    "/home/kramwriter/KramWriter/assets/Settings/darkmode_2.bmp",
                    "/home/kramwriter/KramWriter/assets/Settings/darkmode_3.bmp",
                ];
                for (i, path) in paths.iter().enumerate() {
                    variants[i] = Bitmap::load(path).ok();
                }
            } else {
                // Suffix logic: keyboard uses _3 for selected, others use _1
                let suffix_sel = if *option == "keyboard" { "3" } else { "1" };
                let path_0 = format!("/home/kramwriter/KramWriter/assets/Settings/{}_0.bmp", option);
                let path_sel = format!("/home/kramwriter/KramWriter/assets/Settings/{}_{}.bmp", option, suffix_sel);
                
                variants[0] = Bitmap::load(&path_0).ok();
                variants[1] = Bitmap::load(&path_sel).ok();
            }

            images.push(variants);
        }

        Self { current_index: 0, images }
    }
}

impl Page for SettingsPage {
    fn update(&mut self, key: Key, ctx: &mut Context) -> Action {
        match key {
            Key::Up => {
                if self.current_index > 0 { self.current_index -= 1; }
                Action::None
            }
            Key::Down => {
                if self.current_index < SETTINGS_OPTIONS.len() - 1 { self.current_index += 1; }
                Action::None
            }
            Key::Char('\n') => {
                match self.current_index {
                    0 => Action::Push(Box::new(TimezonePage::new())),
                    2 => {
                        ctx.dark_mode = !ctx.dark_mode;
                        Action::None
                    }
                    3 => {
                        // Triggers the Simplenote email/password entry flow
                        Action::Push(Box::new(SimpleNoteSetupPage::new()))
                    }
                    4 => {
                        ctx.layout = match ctx.layout {
                            KeyboardLayout::Qwerty => KeyboardLayout::Qwertz,
                            KeyboardLayout::Qwertz => KeyboardLayout::Qwerty,
                        };
                        Action::None
                    }
                    _ => Action::None,
                }
            }
            Key::Esc => Action::Pop,
            _ => Action::None,
        }
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        for (i, variants) in self.images.iter().enumerate() {
            let variant_idx;

            if SETTINGS_OPTIONS[i] == "darkmode" {
                // Logic: Unselected Dark=2, Selected Dark=3, Unselected Light=0, Selected Light=1
                variant_idx = if ctx.dark_mode {
                    if i == self.current_index { 3 } else { 2 }
                } else {
                    if i == self.current_index { 1 } else { 0 }
                };
            } else {
                // Standard: Unselected=0, Selected=1
                variant_idx = if i == self.current_index { 1 } else { 0 };
            }

            if let Some(bmp) = &variants[variant_idx] {
                for y in 0..bmp.height.min(240) {
                    for x in 0..bmp.width.min(400) {
                        let pixel = bmp.pixels[y * bmp.width + x];
                        if pixel == Pixel::Black {
                            display.draw_pixel(x, y, pixel, ctx);
                        }
                    }
                }
            }
        }
    }
}