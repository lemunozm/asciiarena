use super::super::gui::util::{self, Context};

use crate::client::state::{State, VersionInfo, ConnectionStatus};
use crate::client::util::store::{StateManager};
use crate::version::{self, Compatibility};

use tui::widgets::{Block, Borders, BorderType, Paragraph};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment, Margin};
use tui::style::{Style, Modifier, Color};
use tui::text::{Span, Spans};

use std::io::{self, Stdout};

const MAIN_TITLE: &'static str = concat!(
r"   _____                .__.__   _____                                ", "\n",
r"  /  _  \   ______ ____ |__|__| /  _  \_______   ____   ____ _____    ", "\n",
r" /  /_\  \ /  ___// ___\|  |  |/  /_\  \_  __ \_/ __ \ /    \\__  \   ", "\n",
r"/    |    \\___ \\  \___|  |  /    |    \  | \/\  ___/|   |  \/ __ \_ ", "\n",
r"\____|__  /______>\_____>__|__\____|__  /__|    \_____>___|__(______/ ", "\n",
r"        \/                            \/", "\n",
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
                Constraint::Length(3), // Margin
                Constraint::Length(2),
                Constraint::Length(2), // Margin
                Constraint::Length(7),
                Constraint::Length(1), // Margin
                Constraint::Length(1),
            ].as_ref())
            .split(space);

        self.draw_menu_panel(ctx, gui_layout[0]);

        let version_space = Rect::new(gui_layout[0].x + 54, gui_layout[0].y + 6, 14, 1);
        self.draw_version_panel(ctx, version_space);

        let client_layout = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(4)
            .constraints([
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
            .alignment(Alignment::Left);

        ctx.frame.render_widget(main_title, space);
    }

    fn draw_client_info_panel(&self, ctx: &mut Context, spaces: Vec<Rect>) {
        self.draw_server_address_panel(ctx, spaces[0]);
        self.draw_player_name_panel(ctx, spaces[1]);
    }

    fn draw_version_panel(&self, ctx: &mut Context, space: Rect) {
        let message = format!("version: {}", version::current());
        let left = Span::styled(message, Style::default().fg(Color::Gray));
        let left_panel = Paragraph::new(left).alignment(Alignment::Left);
        ctx.frame.render_widget(left_panel, space);
    }

    fn draw_server_address_panel(&self, ctx: &mut Context, space: Rect) {
        let addr = ctx.state.server().addr();
        let server_addrees = Spans::from(vec![
            Span::raw("Server address:  "),
            Span::styled(addr.to_string(), Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let left_panel = Paragraph::new(server_addrees).alignment(Alignment::Left);
        ctx.frame.render_widget(left_panel, space);

        let (message, hint_color) = match ctx.state.server().connection_status() {
            ConnectionStatus::Connected => ("Connected", Color::LightGreen),
            ConnectionStatus::NotConnected => ("Not connected", Color::Yellow),
            ConnectionStatus::NotFound => ("Server not found", Color::LightRed),
            ConnectionStatus::Lost => {
                let mut pair = ("Connection lost", Color::LightRed);
                if let Some(VersionInfo {version: _, compatibility}) = ctx.state.server().version_info() {
                    if !compatibility.is_compatible() {
                        pair = ("Version error", Color::LightRed)
                    }
                }
                pair
            }
        };

        let hint = Span::styled(message, Style::default().fg(hint_color));
        let right_panel = Paragraph::new(hint).alignment(Alignment::Right);
        ctx.frame.render_widget(right_panel, space);
    }

    fn draw_player_name_panel(&self, ctx: &mut Context, space: Rect) {
        let player_name = Spans::from(vec![
            Span::raw("Player name:     "),
            Span::styled("L", Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let left_panel = Paragraph::new(player_name).alignment(Alignment::Left);
        ctx.frame.render_widget(left_panel, space);

        let hint_color = Color::LightGreen; //TODO: depends of login status
        let hint = Span::styled("Valid", Style::default().fg(hint_color));
        let right_panel = Paragraph::new(hint).alignment(Alignment::Right);
        ctx.frame.render_widget(right_panel, space);
    }

    fn draw_server_info_panel(&self, ctx: &mut Context, space: Rect) {
        let border = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                "Server info",
                Style::default().add_modifier(Modifier::BOLD)
            ));

        ctx.frame.render_widget(border, space);

        let inner = space.inner(&Margin {vertical: 1, horizontal: 2});

        let vertical_center_inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // Margin
                Constraint::Length(1),
            ].as_ref())
            .split(inner)[1];

        if let Some(VersionInfo {version, compatibility}) = ctx.state.server().version_info() {
            if !compatibility.is_compatible() {
                return self.draw_server_info_panel_err_version(ctx, vertical_center_inner, version);
            }

            if ctx.state.server().connection_status() == ConnectionStatus::Connected {
                return self.draw_server_info_panel_ok(ctx, inner);
            }
        }

        self.draw_server_info_panel_no_info(ctx, vertical_center_inner);
    }

    fn draw_server_info_panel_no_info(&self, ctx: &mut Context, space: Rect) {
        let message = Span::styled("Without information", Style::default().fg(Color::Gray));
        let right_panel = Paragraph::new(message).alignment(Alignment::Center);
        ctx.frame.render_widget(right_panel, space);
    }

    fn draw_server_info_panel_err_version(&self, ctx: &mut Context, space: Rect, version: &str) {
        let message = Spans::from(vec![
            Span::styled("Incompatible versions: ", Style::default().fg(Color::LightRed)),
            Span::styled(version, Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
        ]);
        let right_panel = Paragraph::new(message).alignment(Alignment::Center);
        ctx.frame.render_widget(right_panel, space);
    }

    fn draw_server_info_panel_ok(&self, ctx: &mut Context, space: Rect){
        let VersionInfo {version, compatibility} = ctx.state.server().version_info().unwrap();

        let left = Spans::from(vec![
            Span::raw("version:  "),
            Span::styled(version, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("\n"),
        ]);

        let left_panel = Paragraph::new(left).alignment(Alignment::Left);
        ctx.frame.render_widget(left_panel, space);

        let compatibility_color = match compatibility {
            Compatibility::Fully => Color::LightGreen,
            Compatibility::NotExact => Color::Yellow,
            Compatibility::None => unreachable!(),
        };

        let right = Spans::from(vec![
            Span::styled("compatible", Style::default().fg(compatibility_color)),
            Span::raw("\n"),
        ]);

        let right_panel = Paragraph::new(right).alignment(Alignment::Right);
        ctx.frame.render_widget(right_panel, space);
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
