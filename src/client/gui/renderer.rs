use super::elements::gui::{Gui, GuiWidget};

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

    pub fn render(&mut self, state: &State, gui: &Gui) {
        self.terminal.draw(|frame| {
            let main_widget = GuiWidget::new(state, gui);
            let area = frame.size();
            let mut cursor = None;

            frame.render_stateful_widget(main_widget, area, &mut cursor);

            if let Some(cursor) = cursor {
                frame.set_cursor(cursor.0, cursor.1);
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
