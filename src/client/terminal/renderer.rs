use super::gui::util::{self, Context};
use super::gui::menu::{self, Menu};
use super::gui::arena::{Arena};

use crate::client::state::{State, Gui};

use crossterm::terminal::{self, EnterAlternateScreen};
use crossterm::{ExecutableCommand};

use tui::{Terminal};
use tui::backend::{CrosstermBackend};

use std::io::{self, Stdout};

pub struct Renderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    menu: Menu,
    arena: Arena,
}

impl Renderer {
    pub fn new() -> Renderer {
        terminal::enable_raw_mode().unwrap();
        io::stdout().execute(EnterAlternateScreen).unwrap();
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout())).unwrap();

        Renderer {
            terminal: terminal,
            menu: Menu::new(),
            arena: Arena::new(),
        }
    }

    pub fn render(&mut self, state: &State) {
        let &mut Self {ref mut terminal, ref mut menu, ref mut arena} = self;

        terminal.draw(|frame| {
            match state.gui {
                Gui::Menu(_) => {
                    let menu_space = util::centered_space(frame.size(), menu::DIMENSION);
                    menu.draw(&mut Context::new(&state, frame), menu_space);
                }
                Gui::Arena(_) => {
                    let arena_dimension = arena.required_dimension(state);
                    let arena_space = util::centered_space(frame.size(), arena_dimension);
                    arena.draw(&mut Context::new(&state, frame), arena_space);
                }
            }

        }).unwrap();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        io::stdout().execute(terminal::LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap()
    }
}
