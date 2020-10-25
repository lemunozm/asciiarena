use super::gui::util::{self, Context};
use super::gui::menu::{self, Menu};

use crate::client::state::{State};
use crate::client::util::store::{StateManager};

use crossterm::terminal::{self, EnterAlternateScreen};
use crossterm::{ExecutableCommand};

use tui::{Terminal, Frame};
use tui::backend::{CrosstermBackend};

use std::io::{self, Stdout};

pub struct Renderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    menu: Menu,
}

impl Renderer {
    pub fn new() -> Renderer {
        terminal::enable_raw_mode().unwrap();
        io::stdout().execute(EnterAlternateScreen).unwrap();
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout())).unwrap();

        Renderer {
            terminal: terminal,
            menu: Menu::new(),
        }
    }

    pub fn render(&mut self, state: &StateManager<State>) {
        let &mut Self {ref mut terminal, ref mut menu} = self;

        terminal.draw(|frame: &mut Frame<CrosstermBackend<Stdout>>| {
            let menu_space = util::centered_space(frame.size(), menu::DIMENSION);
            menu.draw(&mut Context::new(&state.get(), frame), menu_space);

        }).unwrap();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        io::stdout().execute(terminal::LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap()
    }
}
