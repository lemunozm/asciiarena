use super::util::{self};

use crate::client::state::{State, GameStatus};
use crate::client::store::{Store, Action};
use crate::client::terminal::input::{InputEvent};

use tui::buffer::{Buffer};
use tui::widgets::{Paragraph, Block, Borders, BorderType, Widget};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment, Margin};
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


#[derive(derive_new::new)]
pub struct ArenaWidget<'a> {
    state: &'a State,
    _arena: &'a Arena,
}

impl<'a> ArenaWidget<'a> {
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
        let map_dim = MapWidget::dimension(map_size);

        let column = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(ArenaInfoLabelWidget::HEIGHT),
                Constraint::Length(map_dim.1),
            ].as_ref())
            .split(area);

        ArenaInfoLabelWidget::new(self.state)
            .render(column[0], buffer);

        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(CharacterPanelListWidget::WIDTH),
                Constraint::Length(1),
                Constraint::Length(map_dim.0),
            ].as_ref())
            .split(column[1]);

        CharacterPanelListWidget::new(self.state)
            .render(row[0], buffer);

        MapWidget::new(self.state)
            .render(row[2], buffer);
    }
}

#[derive(derive_new::new)]
struct ArenaInfoLabelWidget<'a> {state: &'a State}

impl ArenaInfoLabelWidget<'_> {
    pub const HEIGHT: u16 = 1;
}

impl Widget for ArenaInfoLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let points = self.state.server.game_info.as_ref().unwrap().winner_points;
        let number = self.state.server.game.arena_number;

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

#[derive(derive_new::new)]
struct CharacterPanelListWidget<'a> {state: &'a State}

impl CharacterPanelListWidget<'_> {
    pub const WIDTH: u16 = CharacterPanelWidget::DIMENSION.0;
}

impl Widget for CharacterPanelListWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let logged_players = &self.state.server.logged_players;

        let mut constraints = vec![Constraint::Length(1)]; //Top margin
        constraints.extend(
            logged_players
                .iter()
                .map(|_| Constraint::Length(CharacterPanelWidget::DIMENSION.1))
        );
        constraints.push(Constraint::Min(0)); // Bottom margin

        let row = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        for (index, player) in self.state.server.logged_players.iter().enumerate() {
            CharacterPanelWidget::new(self.state, *player)
                .render(row[index + 1], buffer)
        }
    }
}


#[derive(derive_new::new)]
struct CharacterPanelWidget<'a> {
    _state: &'a State,
    player: char,
}

impl<'a> CharacterPanelWidget<'a> {
    pub const DIMENSION: (u16, u16) = (27, 5);
}

impl Widget for CharacterPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let box_style = Style::default().fg(Color::White);

        // Char panel
        let char_area = Rect::new(area.x, area.y, 5, 3).intersection(area);
        Block::default()
            .borders(Borders::ALL)
            .style(box_style)
            .border_type(BorderType::Rounded)
            .render(char_area, buffer);

        let player_style  = Style::default().add_modifier(Modifier::BOLD);
        buffer.set_string(area.x + 2, area.y + 1, self.player.to_string(), player_style);

        // Main panel
        let panel_area = Rect::new(char_area.right(), area.y, 22, 4).intersection(area);
        Block::default()
            .title(Span::styled("Pts: 3", Style::default().add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .style(box_style)
            .border_type(BorderType::Rounded)
            .render(panel_area, buffer);

        // Bars
        let content = panel_area.inner(&Margin {vertical: 1, horizontal: 1});

        let bar_area = Rect::new(content.x, content.y, content.width, 1).intersection(area);
        BarWidget::new(100, 100, Color::Green)
            .render(bar_area, buffer);

        let bar_area = Rect::new(content.x, content.y + 1, content.width, 1).intersection(area);
        BarWidget::new(80, 100, Color::Cyan)
            .render(bar_area, buffer);

        // Bottom
        let arrow_bold = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
        let bottom = Rect::new(panel_area.x, panel_area.bottom() - 1, panel_area.width, 1)
            .intersection(area);
        let bottom = bottom.inner(&Margin {vertical: 0, horizontal: 2});

        let clean_row = (0..bottom.width).map(|_|" ").collect::<String>();
        buffer.set_string(bottom.x, bottom.y, clean_row, arrow_bold);

        buffer.set_string(bottom.x, bottom.y, &">", arrow_bold);
        buffer.set_string(bottom.right() - 1, bottom.y, &"<", arrow_bold);
    }
}

#[derive(derive_new::new)]
struct BarWidget {
    current: usize,
    max: usize,
    color: Color,
}

impl Widget for BarWidget {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let bar_style = Style::default().fg(self.color).add_modifier(Modifier::BOLD);
        let bar_off_style = Style::default().fg(self.color);

        let bar_len = 10 as u16;
        let current_len = ((self.current + 9) as f32 / bar_len as f32) as usize;
        let bar = (0..bar_len as usize)
            .map(|index| {
                if index < current_len {
                    Span::styled("=", bar_style)
                }
                else {
                    Span::styled("-", bar_off_style)
                }
            })
            .collect::<Vec<_>>();

        let numbers = format!("{:>3}/{}", self.current, self.max);

        let limit_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
        buffer.set_string(area.x, area.y, "[", limit_style);
        buffer.set_string(area.x + bar_len + 1, area.y, "]", limit_style);

        buffer.set_spans(area.x + 1, area.y, &Spans::from(bar), 10);
        buffer.set_string(area.x + bar_len + 3, area.y, numbers, bar_style);
    }
}

#[derive(derive_new::new)]
struct MapWidget<'a> {state: &'a State}

impl MapWidget<'_> {
    pub fn dimension(map_size: u16) -> (u16, u16) {
        (2 + map_size * 2 + 1, 2 + map_size)
    }
}

impl Widget for MapWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .render(area, buffer);

        let inner = area.inner(&Margin {vertical: 1, horizontal: 1});

        FinishGameMessageWidget::new(self.state)
            .render(inner, buffer);
    }
}

#[derive(derive_new::new)]
struct FinishGameMessageWidget<'a> {state: &'a State}

impl Widget for FinishGameMessageWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let number = self.state.server.game.arena_number;
        let message = match self.state.server.game.status {
            GameStatus::Finished => {
                vec![
                    Spans::from(Span::raw(format!("Arena {} view (Finished)", number))),
                    Spans::from(Span::raw(format!("Under construcion..."))),
                    Spans::from(Span::raw("")),
                    Spans::from(vec![
                       Span::raw("Press"),
                       Span::styled(" <Enter> ", Style::default()
                           .add_modifier(Modifier::BOLD)
                           .fg(Color::Cyan)),
                       Span::raw("to back to the menu"),
                    ]),
                ]
            }
            _ => {
                vec![
                    Spans::from(Span::raw(format!("Arena {} view (Playing)", number))),
                    Spans::from(Span::raw(format!("Under construcion..."))),
                ]
            }
        };

        let height = message.len() as u16;
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .render(util::vertically_centered(area, height), buffer);
    }
}
