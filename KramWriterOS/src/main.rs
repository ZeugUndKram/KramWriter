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
        // IntoRawMode must be held in a variable to keep the terminal in raw mode
        let _stdout = stdout().into_raw_mode()?;
        
        // 1. ASYNC INPUT SETUP
        // We move keyboard listening to a thread so it doesn't block the loop
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let stdin = stdin();
            for key in stdin.keys() {
                if let Ok(k) = key {
                    if tx.send(k).is_err() { break; }
                }
            }
        });

        // Initial render
        self.render()?;

        loop {
            // 2. WAIT WITH TIMEOUT
            // Check for a key, but only wait for 100ms so tick() can run.
            let key_event = rx.recv_timeout(Duration::from_millis(100)).ok();

            // Handle Global Exit
            if let Some(Key::Ctrl('x')) = key_event {
                self.display.clear(&self.ctx);
                self.display.update()?;
                return Ok(());
            }

            // 3. PAGE LOGIC
            let mut should_render = false;
            let action = if let Some(top_page) = self.stack.last_mut() {
                match key_event {
                    Some(key) => {
                        should_render = true; // Always render if user pressed a key
                        top_page.update(key, &mut self.ctx)
                    }
                    None => {
                        // This is the "Automatic" part: call tick() even without input.
                        let tick_action = top_page.tick(&mut self.ctx);
                        
                        // If tick returns anything other than None, we should redraw.
                        if !matches!(tick_action, Action::None) {
                            should_render = true;
                        }
                        tick_action
                    }
                }
            } else {
                Action::Exit
            };

            // 4. PROCESS ACTION
            match action {
                Action::Push(new_page) => {
                    self.stack.push(new_page);
                    should_render = true;
                },
                Action::Pop => { 
                    self.stack.pop(); 
                    should_render = true;
                },
                Action::Replace(new_page) => {
                    self.stack.pop();
                    self.stack.push(new_page);
                    should_render = true;
                },
                Action::Redraw => {
                    // Force a render even if no page change or keypress occurred
                    should_render = true;
                },
                Action::Exit => break,
                Action::None => {},
            }

            if self.stack.is_empty() { 
                break; 
            }

            // 5. CONDITIONAL RENDER
            // Only update the display if something actually changed.
            if should_render {
                self.render()?;
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