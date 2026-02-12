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
        // Start with the Logo page
        let startup_page = Box::new(pages::startup::LogoPage::new());

        Ok(Self {
            display,
            ctx,
            stack: vec![startup_page],
        })
    }

    fn run(&mut self) -> Result<()> {
        // Raw mode is necessary to catch individual keypresses without Enter
        let _stdout = stdout().into_raw_mode()?;
        let stdin = stdin();
        let mut keys = stdin.keys();

        self.render()?; // Initial frame

        loop {
            if let Some(Ok(key)) = keys.next() {
                // 1. GLOBAL INTERCEPT: Check for Ctrl+X first
                if key == Key::Ctrl('x') {
                    println!("Exiting kramwriter...");
                    // Clear the display before leaving (optional but clean)
                    self.display.clear()?;
                    self.display.update()?;
                    return Ok(()); // This breaks the loop and exits the app
                }

                // 2. LOGIC HANDLING: Pass other keys to the current page
                let action = current_page.update(key, &mut context);
                
                match action {
                    Action::Replace(next_page) => {
                        current_page = next_page;
                    }
                    Action::None => {}
                }
            }
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

                if self.stack.is_empty() { break; }
                self.render()?;
            }
        }
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.display.clear();
        
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