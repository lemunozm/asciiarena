use crate::client::configuration::{Config};
use crate::client::state::{State, GameStatus};
use crate::client::util::store::{Store};
use crate::client::actions::{ActionManager, Action};
use crate::client::gui::input::{InputEvent};
use crate::client::gui::element::{Context, GuiElement};

use tui::widgets::{Paragraph};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment};
use tui::style::{Style, Modifier};
use tui::text::{Span, Spans};

use crossterm::event::{KeyCode};

pub struct Arena {}

impl GuiElement for Arena {
    fn process_event(&mut self, store: &mut Store<ActionManager>, event: InputEvent) {
        match event {
            InputEvent::KeyPressed(key_event) => match key_event.code {
                KeyCode::Enter => {
                    if let GameStatus::Finished = store.state().server.game.status {
                        store.dispatch(Action::CloseGame);
                    }
                }
                _ => (),
            },
            InputEvent::ResizeDisplay(_, _) => {},
        }
    }

    fn update(&mut self, _store: &State) {
    }

    fn render(&self, ctx: &mut Context, space: Rect) {
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
}

impl Arena {
    pub fn new(_config: &Config) -> Arena {
        Arena {}
    }

    pub fn required_dimension(&self, state: &State) -> (u16, u16) {
        let map_size = state.server.game_info.as_ref()
            .expect("'game_info' must me known at this point").map_size as u16;
        (map_size + 15, map_size + 3)
    }
}
