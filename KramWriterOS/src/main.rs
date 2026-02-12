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
use termion::event::Key; // Added this import
use anyhow::Result;

struct App {
    display: SharpDisplay,
    ctx: Context,
    stack: Vec<Box<dyn Page>>,
}

impl App {
    fn new() -> Result<Self> {
        // Using your CS pin 6
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
        let stdin = stdin();
        let mut keys = stdin.keys();

        self.render()?;

        loop {
            if let Some(Ok(key)) = keys.next() {
                // 1. GLOBAL INTERCEPT: Ctrl+X to kill the app
                if key == Key::Ctrl('x') {
                    self.display.clear(); // Removed the ?
                    self.display.update()?; // Keep this one, update returns Result
                    return Ok(());
                }

                // 2. GET ACTION: Pass key to the top page of the stack
                let action = if let Some(top_page) = self.stack.last_mut() {
                    top_page.update(key, &mut self.ctx)
                } else {
                    Action::Exit
                };

                // 3. PROCESS ACTION
                match action {
                    Action::Push(new_page) => self.stack.push(new_page),
                    Action::Pop => { self.stack.pop(); },
                    Action::Replace(new_page) => {
                        self.stack.pop();
                        self.stack.push(new_page);
                    },
                    Action::Exit => break,
                    Action::None => {},
                }

                // If we popped everything, exit the app
                if self.stack.is_empty() { 
                    break; 
                }

                // 4. RENDER the new state
                self.render()?;
            }
        }
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
    // Pass &self.ctx so the display knows what color to clear with
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