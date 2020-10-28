use super::element::{Context, GuiElement};

use crate::client::state::{State};

use crossterm::terminal::{self, EnterAlternateScreen};
use crossterm::{ExecutableCommand};

use tui::{Terminal};
use tui::backend::{CrosstermBackend};

use std::io::{self, Stdout};

pub struct Renderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Renderer {
    pub fn new() -> Renderer {
        terminal::enable_raw_mode().unwrap();
        io::stdout().execute(EnterAlternateScreen).unwrap();
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout())).unwrap();

        Renderer {
            terminal: terminal,
        }
    }

    pub fn render(&mut self, state: &State, element: &dyn GuiElement) {
        self.terminal.draw(|frame| {
            let space = frame.size();
            element.render(&mut Context::new(&state, frame), space);
        }).unwrap();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        io::stdout().execute(terminal::LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap()
    }
}
