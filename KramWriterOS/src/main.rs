mod display;
mod context;
mod pages;
mod ui;

use crate::display::SharpDisplay;
use crate::context::Context;
use crate::pages::{Page, Action};
use std::io::{stdin, stdout};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::event::Key;
use anyhow::Result;

struct App {
    display: SharpDisplay,
    ctx: Context,
    stack: Vec<Box<dyn Page>>,
}

impl App {
    fn new() -> Result<Self> {
        let display = SharpDisplay::new(6)?;
        let ctx = Context::new();
        let startup_page = Box::new(pages::startup::LogoPage::new());

        Ok(Self {
            display,
            ctx,
            stack: vec![startup_page],
        })
    }

    fn run(&mut self) -> Result<()> {
        let _stdout = stdout().into_raw_mode()?;

        // 1. Setup background input thread
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let stdin = stdin();
            for key in stdin.keys() {
                if let Ok(k) = key {
                    // If the main thread drops rx (exits), this send will fail gracefully
                    if tx.send(k).is_err() {
                        break;
                    }
                }
            }
        });

        // Initial render
        self.render()?;

        // 2. Main Game Loop (~60 FPS)
        loop {
            let mut needs_redraw = false;

            // 3. Process all queued input (Non-blocking)
            while let Ok(key) = rx.try_recv() {
                // Global Intercept
                if key == Key::Ctrl('x') {
                    self.display.clear(&self.ctx);
                    self.display.update()?;
                    return Ok(());
                }

                let action = if let Some(top_page) = self.stack.last_mut() {
                    top_page.update(key, &mut self.ctx)
                } else {
                    Action::Exit
                };

                match action {
                    Action::Push(new_page) => self.stack.push(new_page),
                    Action::Pop => { self.stack.pop(); },
                    Action::Replace(new_page) => {
                        self.stack.pop();
                        self.stack.push(new_page);
                    },
                    Action::Exit => return Ok(()),
                    Action::None => {},
                }
                
                needs_redraw = true;
            }

            if self.stack.is_empty() { break; }

            // 4. Tick logic (for animations, physics, or timers)
            if let Some(top_page) = self.stack.last_mut() {
                if top_page.tick(&mut self.ctx) {
                    needs_redraw = true;
                }
            }

            // 5. Render only if necessary
            if needs_redraw {
                self.render()?;
            }

            // Cap execution speed
            thread::sleep(Duration::from_millis(16));
        }
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.display.clear(&self.ctx);
        
        if let Some(top_page) = self.stack.last_mut() {
            top_page.draw(&mut self.display, &self.ctx);
        }
        
        self.display.update()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()
}