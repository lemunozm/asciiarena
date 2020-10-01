use super::super::gui::util::{self, Context};

use crate::client::state::{State, VersionInfo};
use crate::client::util::store::{StateManager};
use crate::version::{self, Compatibility};

use tui::widgets::{Block, Borders, BorderType, Paragraph};
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
pub const DIMENSION: (u16, u16) = (70, 22);

pub struct Menu {}

impl Menu {
    pub fn new() -> Menu {
        Menu {}
    }

    pub fn draw(&mut self, ctx: &mut Context, space: Rect) {
        let gui_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(MAIN_TITLE.chars().filter(|&c| c == '\n').count() as u16),
                Constraint::Length(2), // Margin
                Constraint::Length(3),
                Constraint::Length(2), // Margin
                Constraint::Min(0),
                Constraint::Length(1), // Margin
                Constraint::Length(1),
            ].as_ref())
            .split(space);

        self.draw_menu_panel(ctx, gui_layout[0]);

        let client_layout = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(4)
            .constraints([ //Borrar?
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ].as_ref())
            .split(gui_layout[2]);

        self.draw_client_info_panel(ctx, client_layout);

        let server_layout = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(2)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Length(2), // Margin
                Constraint::Percentage(40),
            ].as_ref())
            .split(gui_layout[4]);

        self.draw_server_info_panel(ctx, server_layout[0]);
        self.draw_waiting_room_panel(ctx, server_layout[2]);

        self.draw_starting_notify_panel(ctx, gui_layout[6]);
    }

    fn draw_menu_panel(&self, ctx: &mut Context, space: Rect) {
        let main_title = Paragraph::new(MAIN_TITLE)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);

        ctx.frame.render_widget(main_title, space);
    }

    fn draw_client_info_panel(&self, ctx: &mut Context, spaces: Vec<Rect>) {
       self.draw_server_address_panel(ctx, spaces[0]);
       self.draw_version_panel(ctx, spaces[1]);
       self.draw_player_name_panel(ctx, spaces[2]);
    }

    fn draw_version_panel(&self, ctx: &mut Context, space: Rect) {
        let version = Spans::from(vec![
            Span::raw("Client version:  "),
            Span::styled(version::current(), Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let (message, hint_color) = match ctx.state.get().server().version_info() {
            Some(VersionInfo {version: _, compatibility}) => match compatibility {
                Compatibility::Fully => {
                    ("Compatible", Color::Green)
                },
                Compatibility::NotExact => {
                    ("Compatible", Color::Yellow)
                },
                Compatibility::None => {
                    ("Not compatible", Color::Red)
                },
            },
            None => ("", Color::White)
        };

        let hint = Spans::from(vec![
            Span::styled(message, Style::default().fg(hint_color)),
        ]);

        let left_panel = Paragraph::new(version).alignment(Alignment::Left);
        let right_panel = Paragraph::new(hint).alignment(Alignment::Right);

        ctx.frame.render_widget(left_panel, space);
        ctx.frame.render_widget(right_panel, space);
    }

    fn draw_server_address_panel(&self, ctx: &mut Context, space: Rect) {
        let server_addrees = Spans::from(vec![
            Span::raw("Server address:  "),
            Span::styled(ctx.state.get().server().addr().to_string(),
                Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let hint_color = Color::Green; //TODO: depends of connection
        let hint = Spans::from(vec![
            Span::styled("Connected", Style::default().fg(hint_color)),
        ]);

        let left_panel = Paragraph::new(server_addrees).alignment(Alignment::Left);
        let right_panel = Paragraph::new(hint).alignment(Alignment::Right);

        ctx.frame.render_widget(left_panel, space);
        ctx.frame.render_widget(right_panel, space);
    }

    fn draw_player_name_panel(&self, ctx: &mut Context, space: Rect) {
        let player_name = Spans::from(vec![
            Span::raw("Player name:     "),
            Span::styled("L", Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let hint_color = Color::Green; //TODO: depends of login status
        let hint = Spans::from(vec![
            Span::styled("Valid", Style::default().fg(hint_color)),
        ]);

        let left_panel = Paragraph::new(player_name).alignment(Alignment::Left);
        let right_panel = Paragraph::new(hint).alignment(Alignment::Right);

        ctx.frame.render_widget(left_panel, space);
        ctx.frame.render_widget(right_panel, space);
    }

    fn draw_server_info_panel(&self, ctx: &mut Context, space: Rect) {
        let panel = Paragraph::new(" Version:\n Players:\n Map size:\n Winner points:\n UDP port:")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    "Server info",
                    Style::default().add_modifier(Modifier::BOLD)
                )))
            .alignment(Alignment::Left);

        ctx.frame.render_widget(panel, space);
    }

    fn draw_waiting_room_panel(&self, ctx: &mut Context, space: Rect) {
        let panel = Paragraph::new(" Player 1:\n Player 2:\n Player 3:\n Player 4:")
            .block(Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "Waiting room",
                    Style::default().add_modifier(Modifier::BOLD)
                )))
            .alignment(Alignment::Left);

        ctx.frame.render_widget(panel, space);
    }

    fn draw_starting_notify_panel(&self, ctx: &mut Context, space: Rect) {
        let message = "Waiting for players: 1/4";
                      //" Starting arena in 2..."
                      //"Game already started"
        let panel = Paragraph::new(message)
            .alignment(Alignment::Center);

        ctx.frame.render_widget(panel, space);
    }
}
