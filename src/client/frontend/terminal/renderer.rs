use crate::client::frontend::{Renderer};
use crate::client::state::{State};
use crate::client::util::store::{StateManager};

use crossterm::terminal::{self, EnterAlternateScreen};
use crossterm::{ExecutableCommand};

use tui::{Terminal, Frame};
use tui::backend::{CrosstermBackend};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment};
use tui::style::{Style, Modifier, Color};
use tui::text::{Span, Spans};

use std::io::{self, Stdout};

const MAIN_TITLE: &'static str = concat!(
r"   _____                .__.__   _____                                ", "\n",
r"  /  _  \   ______ ____ |__|__| /  _  \_______   ____   ____ _____    ", "\n",
r" /  /_\  \ /  ___// ___\|  |  |/  /_\  \_  __ \_/ __ \ /    \\__  \   ", "\n",
r"/    |    \\___ \\  \___|  |  /    |    \  | \/\  ___/|   |  \/ __ \_ ", "\n",
r"\____|__  /______>\_____>__|__\____|__  /__|    \_____>___|__(______/ ", "\n",
r"        \/                            \/                              ", "\n",
);
const UI_DIMENSION: (u16, u16) = (70, 20);

/*
    Server address:   127.0.0.1:3000                   (Connected!)
    Player name:      L                              Capital letter

    --Server Info----------    -- Waiting room ---------------------
    | Players: 4          |    | Player 1: L                       |
    | Map size: 30x30     |    | Player 2: T                       |
    | Winner points: 15   |    | Player 3: E                       |
    | UDP port: 3456      |    | Player 4: (waiting...)            |
    -----------------------    -------------------------------------

                         Initializing Arena...
*/

pub struct TerminalRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalRenderer {
    pub fn new() -> TerminalRenderer {
        terminal::enable_raw_mode().unwrap();
        io::stdout().execute(EnterAlternateScreen).unwrap();
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout())).unwrap();

        TerminalRenderer { terminal }
    }

    fn centered_space(base: Rect, dimension: (u16, u16)) -> Rect {
        let width_diff = base.width as i16 - dimension.0 as i16;
        let height_diff = base.height as i16 - dimension.1 as i16;
        let x = if width_diff > 0 { width_diff / 2 } else { 0 };
        let y = if height_diff > 0 { height_diff / 2 } else { 0 };
        let width = if base.width > dimension.0 { dimension.0 } else { base.width };
        let height = if base.height > dimension.1 { dimension.1 } else { base.height };
        Rect::new(x as u16, y as u16, width, height)
    }
}

impl Renderer for TerminalRenderer {
    fn render(&mut self, state: &StateManager<State>) {
        self.terminal.draw(|frame| {

            let ui_space = Self::centered_space(frame.size(), UI_DIMENSION);
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(100),
                ].as_ref())
                .split(ui_space);

            let main_title = Paragraph::new(MAIN_TITLE)
                .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);

            frame.render_widget(main_title, layout[0]);

        }).unwrap();
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        io::stdout().execute(terminal::LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap()
    }
}
