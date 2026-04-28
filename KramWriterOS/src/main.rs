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
        
        // 1. ASYNC INPUT SETUP
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let stdin = stdin();
            for key in stdin.keys() {
                if let Ok(k) = key {
                    if tx.send(k).is_err() { break; }
                }
            }
        });

        self.render()?;

        loop {
            // Wait for 100ms so the loop "ticks" regularly even without typing
            let key_event = rx.recv_timeout(Duration::from_millis(100)).ok();

            if let Some(Key::Ctrl('x')) = key_event {
                self.display.clear(&self.ctx);
                self.display.update()?;
                return Ok(());
            }

            let mut should_render = false;
            let action = if let Some(top_page) = self.stack.last_mut() {
                match key_event {
                    Some(key) => {
                        should_render = true; 
                        top_page.update(key, &mut self.ctx)
                    }
                    None => {
                        let tick_action = top_page.tick(&mut self.ctx);
                        
                        // FIX: Use is_none() helper instead of != Action::None
                        if !tick_action.is_none() {
                            should_render = true;
                        }
                        tick_action
                    }
                }
            } else {
                Action::Exit
            };

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
                // FIX: Handle the Redraw variant explicitly
                Action::Redraw => {
                    should_render = true;
                },
                Action::Exit => break,
                Action::None => {},
            }

            if self.stack.is_empty() { 
                break; 
            }

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