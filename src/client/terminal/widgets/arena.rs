use crate::client::state::{State, GameStatus};
use crate::client::store::{Store, Action};
use crate::client::terminal::input::{InputEvent};

use tui::buffer::{Buffer};
use tui::widgets::{Paragraph, Block, Borders, Widget};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment};
use tui::style::{Style, Modifier, Color};
use tui::text::{Span, Spans};

use crossterm::event::{KeyCode};

pub struct Arena {}

impl Arena {
    pub fn process_event(&mut self, store: &mut Store, event: InputEvent) {
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

    pub fn update(&mut self, _state: &State) { }
}

pub struct ArenaWidget<'a> {
    state: &'a State,
    _arena: &'a Arena,
}

impl<'a> ArenaWidget<'a> {
    pub fn new(state: &'a State, _arena: &'a Arena) -> ArenaWidget<'a> {
        ArenaWidget { state, _arena }
    }

    pub fn dimension(state: &State) -> (u16, u16) {
        let map_size = state.server.game_info.as_ref().unwrap().map_size as u16;
        let map_dim = MapWidget::dimension(map_size);

        (CharacterPanelListWidget::WIDTH + 1 + map_dim.0,
         ArenaInfoLabelWidget::HEIGHT + map_dim.1)
    }
}

impl Widget for ArenaWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let map_size = self.state.server.game_info.as_ref().unwrap().map_size as u16;
        let column = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(ArenaInfoLabelWidget::HEIGHT),
                Constraint::Length(map_size),
            ].as_ref())
            .split(area);

        ArenaInfoLabelWidget{state: self.state}
            .render(column[0], buffer);

        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(CharacterPanelListWidget::WIDTH),
                Constraint::Length(1),
                Constraint::Length(map_size * 2 + 1),
            ].as_ref())
            .split(column[1]);


        CharacterPanelListWidget{state: self.state}
            .render(row[0], buffer);

        MapWidget{state: self.state}
            .render(row[2], buffer);
    }
}


struct ArenaInfoLabelWidget<'a> {state: &'a State}

impl ArenaInfoLabelWidget<'_> {
    pub const HEIGHT: u16 = 1;
}

impl Widget for ArenaInfoLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let points = self.state.server.game_info.as_ref().unwrap().winner_points;
        let number = self.state.server.game.arena.as_ref().unwrap().number;

        let title = Spans::from(vec![
            Span::raw("Arena "),
            Span::styled(number.to_string(), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Points to win: "),
            Span::styled(points.to_string(), Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(title)
            .alignment(Alignment::Center)
            .render(area, buffer);
    }
}


struct CharacterPanelListWidget<'a> {state: &'a State}

impl CharacterPanelListWidget<'_> {
    pub const WIDTH: u16 = CharacterPanelWidget::DIMENSION.0;
}

impl Widget for CharacterPanelListWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let logged_players = &self.state.server.logged_players;

        Block::default()
            .style(Style::default().fg(Color::DarkGray))
            .borders(Borders::ALL)
            .render(area, buffer);

        let constraints = logged_players
            .iter()
            .map(|_| Constraint::Length(CharacterPanelWidget::DIMENSION.1))
            .collect::<Vec<_>>();

        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        for (index, player) in self.state.server.logged_players.iter().enumerate() {
            CharacterPanelWidget::new(self.state, *player)
                .render(row[index], buffer)
        }
    }
}


struct CharacterPanelWidget<'a> {
    _state: &'a State,
    _player: char,
}

impl<'a> CharacterPanelWidget<'a> {
    pub const DIMENSION: (u16, u16) = (18, 3);

    pub fn new(_state: &'a State, _player: char) -> CharacterPanelWidget<'a> {
        CharacterPanelWidget {_state, _player}
    }
}

impl Widget for CharacterPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Block::default()
            .style(Style::default().fg(Color::DarkGray))
            .borders(Borders::ALL)
            .render(area, buffer);
    }
}


struct MapWidget<'a> {state: &'a State}

impl MapWidget<'_> {
    pub fn dimension(map_size: u16) -> (u16, u16) {
        (2 + map_size, 2 + map_size * 2 + 1)
    }
}

impl Widget for MapWidget<'_> {
    fn render(self, mut area: Rect, buffer: &mut Buffer) {
        let map_size = self.state.server.game_info.as_ref().unwrap().map_size as u16;

        let map_y_offset = (area.height - map_size) / 2;
        if map_y_offset > 0 {
            area.y += map_y_offset;
            area.height -= map_y_offset;
        }

        Block::default()
            .borders(Borders::ALL)
            .render(area, buffer);
    }
}
