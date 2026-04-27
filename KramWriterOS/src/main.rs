mod display;
mod context;
mod pages;
mod ui;

use crate::display::SharpDisplay;
use crate::context::Context;
use crate::pages::{Page, Action};
use std::io::{stdin, stdout};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::event::Key;
use anyhow::Result;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
        
        // --- ASYNC INPUT SETUP ---
        // Create a channel for keys so they don't block the main loop
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let stdin = stdin();
            for key in stdin.keys() {
                if let Ok(k) = key {
                    let _ = tx.send(k);
                }
            }
        });

        self.render()?;

        loop {
            // Check for a key, but only wait for 50ms. 
            // This "tick" allows the screen to refresh even if no key is pressed.
            let key_event = rx.recv_timeout(Duration::from_millis(50)).ok();

            // 1. GLOBAL INTERCEPT: Ctrl+X to kill the app
            if let Some(Key::Ctrl('x')) = key_event {
                self.display.clear(&self.ctx);
                self.display.update()?;
                return Ok(());
            }

            // 2. GET ACTION
            // We pass Option<Key> to the page. 
            // If it's None, the page can still check background tasks.
            let action = if let Some(top_page) = self.stack.last_mut() {
                top_page.update(key_event, &mut self.ctx)
            } else {
                Action::Exit
            };

            // 3. PROCESS ACTION
            match action {
                Action::Push(new_page) => {
                    self.stack.push(new_page);
                    self.render()?; // Immediate render on new page
                },
                Action::Pop => { 
                    self.stack.pop(); 
                    self.render()?; 
                },
                Action::Replace(new_page) => {
                    self.stack.pop();
                    self.stack.push(new_page);
                    self.render()?;
                },
                Action::Exit => break,
                Action::None => {
                    // Even if no action happened, we render. 
                    // This is what makes the "Uploading..." text appear live.
                    self.render()?;
                },
            }

            if self.stack.is_empty() { 
                break; 
            }
        }
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.display.clear(&self.ctx);
        
        if let Some(top_page) = self.stack.last() {
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