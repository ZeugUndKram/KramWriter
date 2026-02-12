mod display;
mod context;
mod pages;

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