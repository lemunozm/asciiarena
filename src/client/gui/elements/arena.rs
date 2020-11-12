use crate::client::configuration::{Config};
use crate::client::state::{State, GameStatus};
use crate::client::store::{Store, Action};
use crate::client::gui::input::{InputEvent};
use crate::client::gui::element::{Context, GuiElement};

use tui::widgets::{Paragraph, Block, Borders};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment};
use tui::style::{Style, Modifier, Color};
use tui::text::{Span, Spans};

use crossterm::event::{KeyCode};

pub struct Arena {}

impl GuiElement for Arena {
    fn process_event(&mut self, store: &mut Store, event: InputEvent) {
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

    fn update(&mut self, _state: &State) { }

    fn render(&self, ctx: &mut Context, space: Rect) {
        let map_size = ctx.state.server.game_info.as_ref().unwrap().map_size as u16;
        let gui_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(map_size),
            ].as_ref())
            .split(space);

        self.draw_arena_info(ctx, gui_vertical[0]);

        let gui_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(15),
                Constraint::Length(1),
                Constraint::Length(map_size * 2 + 1),
            ].as_ref())
            .split(gui_vertical[1]);

        self.draw_character_info_panels(ctx, gui_horizontal[0]);
        self.draw_map(ctx, gui_horizontal[2]);
    }
}

impl Arena {
    pub fn new(_config: &Config) -> Arena {
        Arena {}
    }

    pub fn required_dimension(&self, state: &State) -> (u16, u16) {
        let map_size = state.server.game_info.as_ref()
            .expect("'game_info' must me known at this point").map_size as u16;
        (15 + 1 + map_size * 2 + 1, 1 + map_size)
    }

    fn draw_arena_info(&self, ctx: &mut Context, space: Rect) {
        let winner_points = ctx.state.server.game_info.as_ref().unwrap().winner_points;
        let arena = ctx.state.server.game.arena.as_ref().unwrap();

        let title = Spans::from(vec![
            Span::raw("Arena "),
            Span::styled(arena.number.to_string(), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Points to win: "),
            Span::styled(winner_points.to_string(), Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let panel = Paragraph::new(title)
            .alignment(Alignment::Center);

        ctx.frame.render_widget(panel, space);
    }

    fn draw_character_info_panels(&self, ctx: &mut Context, space: Rect) {
        let info_panels = Block::default()
            .style(Style::default().fg(Color::DarkGray))
            .borders(Borders::ALL);

        ctx.frame.render_widget(info_panels, space);
    }

    fn draw_map(&self, ctx: &mut Context, mut space: Rect) {
        let map_size = ctx.state.server.game_info.as_ref().unwrap().map_size as u16;

        let map_y_offset = (space.height - map_size) / 2;
        if map_y_offset > 0 {
            space.y += map_y_offset;
            space.width -= map_y_offset;
        }

        let map = Block::default().borders(Borders::ALL);
        ctx.frame.render_widget(map, space);
    }

    fn draw_under_construction_panel(&self, ctx: &mut Context, space: Rect) {
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
                   Span::styled(" <Enter> ", Style::default()
                       .add_modifier(Modifier::BOLD)
                       .fg(Color::Cyan)),
                   Span::raw("to back to the menu"),
                ]),
                "Finished")
            }
            _ => (Spans::default(), "Playing"),
        };

        let number = ctx.state.server.game.arena().number;
        let text = vec![
            Spans::from(Span::raw(format!("Arena {} view ({})", number, status))),
            Spans::from(Span::raw(format!("Under construcion..."))),
            Spans::from(""),
            enter_key_spans
        ];

        let building_comment = Paragraph::new(text)
            .alignment(Alignment::Center);

        ctx.frame.render_widget(building_comment, gui_layout[1]);
    }
}
