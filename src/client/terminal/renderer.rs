use super::widgets::gui::{Gui, GuiWidget};

use crate::client::state::{State};

use crossterm::terminal::{self};
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
        io::stdout().execute(terminal::EnterAlternateScreen).unwrap();

        Renderer {
            terminal: Terminal::new(CrosstermBackend::new(io::stdout())).unwrap();
        }
    }

    pub fn render(&mut self, state: &State, gui: &Gui) {
        self.terminal.draw(|frame| {
            let main_widget = GuiWidget::new(state, gui);
            let area = frame.size();
            let mut cursor = Cursor::default();

            frame.render_stateful_widget(main_widget, area, &mut cursor);

            if let Some(position) = cursor.take() {
                frame.set_cursor(position.0, position.1);
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

#[derive(Default)]
pub struct Cursor {
    position: Option<(u16, u16)>,
}

impl Cursor {
    pub fn set(&mut self, x: u16, y: u16) {
        self.position = Some((x, y));
    }

    fn take(&mut self) -> Option<(u16, u16)> {
        self.position.take()
    }
}
