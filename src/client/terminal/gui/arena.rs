use super::super::gui::util::{Context};

use crate::client::state::{State, GameStatus};

use tui::widgets::{Block, Borders, BorderType, Paragraph};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment, Margin};
use tui::style::{Style, Modifier, Color};
use tui::text::{Span, Spans};

pub struct Arena {}

impl Arena {
    pub fn new() -> Arena {
        Arena {}
    }

    pub fn draw(&mut self, ctx: &mut Context, space: Rect) {
        let gui_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(space);

        let (enter_key_spans, status) = match ctx.state.server.game.status {
            GameStatus::Finished => {
                (Spans::from(vec![
                   Span::raw("Press"),
                   Span::styled(" Enter ", Style::default().add_modifier(Modifier::BOLD)),
                   Span::raw("key to back to the menu"),
                ]),
                "Finished")
            }
            _ => (Spans::default(), "Playing"),
        };

        let text = vec![
            Spans::from(Span::raw(format!("Arena view ({})", status))),
            Spans::from(""),
            enter_key_spans
        ];

        let building_comment = Paragraph::new(text)
            .alignment(Alignment::Center);

        ctx.frame.render_widget(building_comment, gui_layout[1]);
    }

    pub fn required_dimension(&self, state: &State) -> (u16, u16) {
        let map_size = state.server.game_info.as_ref()
            .expect("'game_info' must me known at this point").map_size as u16;
        (map_size + 13, map_size + 3)
    }
}
