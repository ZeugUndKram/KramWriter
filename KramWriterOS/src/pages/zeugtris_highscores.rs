use crate::pages::{Page, Action};
use crate::context::Context;
use crate::display::SharpDisplay;
use crate::ui::bitmap::Bitmap;
use crate::ui::fonts::FontRenderer;
use termion::event::Key;
use rpi_memory_display::Pixel;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct ZeugtrisHighscoresPage {
    title_bmp: Option<Bitmap>,
    renderer: FontRenderer,
    scores: Vec<u32>,
}

impl ZeugtrisHighscoresPage {
    pub fn new() -> Self {
        let renderer = FontRenderer::new("/home/kramwriter/KramWriter/fonts/BebasNeue-Regular.ttf");
        let title_bmp = Bitmap::load("/home/kramwriter/KramWriter/assets/zeugtris/highscores/highscores.bmp").ok();

        let mut scores = Vec::new();
        let path = "/home/kramwriter/KramWriter/assets/zeugtris/highscores.txt";
        
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                if let Ok(val) = line.trim().parse::<u32>() {
                    scores.push(val);
                }
            }
        }
        
        // Sort descending to ensure proper order
        scores.sort_unstable_by(|a, b| b.cmp(a));
        
        // Ensure we always have exactly 10 entries for display purposes (pad with 0s if needed)
        scores.resize(10, 0);

        Self {
            title_bmp,
            renderer,
            scores,
        }
    }
}

impl Page for ZeugtrisHighscoresPage {
    fn update(&mut self, key: Key, _ctx: &mut Context) -> Action {
        if key == Key::Esc {
            return Action::Pop;
        }
        Action::None
    }

    fn draw(&self, display: &mut SharpDisplay, ctx: &Context) {
        display.clear(ctx);

        // Draw title bitmap centered horizontally
        if let Some(bmp) = &self.title_bmp {
            let x_off = (400 - bmp.width as i32) / 2;
            let y_off = 15; // Padding from top
            
            for y in 0..bmp.height {
                for x in 0..bmp.width {
                    if bmp.pixels[y * bmp.width + x] == Pixel::Black {
                        let px = (x as i32 + x_off) as usize;
                        let py = y + y_off;
                        if px < 400 && py < 240 {
                            display.draw_pixel(px, py, Pixel::Black, ctx);
                        }
                    }
                }
            }
        }

        // Display Scores in a two-column layout
        let font_size = 32.0;
        let start_y = 110;
        let spacing_y = 28;
        
        let col1_x = 70;
        let col2_x = 230;

        for (i, &score) in self.scores.iter().enumerate() {
            let text = format!("{}. {}", i + 1, score);
            
            let x = if i < 5 { col1_x } else { col2_x };
            let y = start_y + ((i % 5) as i32 * spacing_y);

            self.renderer.draw_text(display, &text, x, y, font_size, ctx);
        }
    }
}