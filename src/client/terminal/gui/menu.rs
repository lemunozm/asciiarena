use super::super::gui::util::{Context};

use crate::client::state::{VersionInfo, ConnectionStatus};
use crate::version::{self, Compatibility};
use crate::message::{LoginStatus};

use tui::widgets::{Block, Borders, BorderType, Paragraph};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment, Margin};
use tui::style::{Style, Modifier, Color};
use tui::text::{Span, Spans};

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

    fn draw_version_panel(&self, ctx: &mut Context, space: Rect) {
        let message = format!("version: {}", version::current());
        let version = Span::styled(message, Style::default().fg(Color::Gray));
        let panel = Paragraph::new(version).alignment(Alignment::Left);
        ctx.frame.render_widget(panel, space);
    }

    fn draw_client_info_panel(&self, ctx: &mut Context, spaces: Vec<Rect>) {
        self.draw_server_address_panel(ctx, spaces[0]);
        self.draw_player_name_panel(ctx, spaces[1]);
    }

    fn draw_server_address_panel(&self, ctx: &mut Context, space: Rect) {
        let input_addr = &ctx.state.gui.menu().server_addr_input;

        let server_addrees = Spans::from(vec![
            Span::raw("Server address:  "),
            Span::styled(input_addr, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let left_panel = Paragraph::new(server_addrees).alignment(Alignment::Left);
        ctx.frame.render_widget(left_panel, space);

        let (message, hint_color) = match ctx.state.server.connection_status {
            ConnectionStatus::Connected => ("Connected", Color::LightGreen),
            ConnectionStatus::NotConnected => ("Not connected", Color::Yellow),
            ConnectionStatus::NotFound => ("Server not found", Color::LightRed),
            ConnectionStatus::Lost => {
                let mut pair = ("Connection lost", Color::LightRed);
                if let Some(VersionInfo {version: _, compatibility}) = ctx.state.server.version_info {
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

        if let Some(VersionInfo {version, compatibility}) = &ctx.state.server.version_info {
            if !compatibility.is_compatible() {
                return self.draw_server_info_panel_err_version(ctx, vertical_center_inner, &version);
            }

            if let ConnectionStatus::Connected = ctx.state.server.connection_status {
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
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ].as_ref())
            .split(space);

        self.draw_server_info_version_panel(ctx, layout[0]);
        self.draw_server_info_map_size_panel(ctx, layout[1]);
        self.draw_server_info_points_panel(ctx, layout[2]);
        self.draw_server_info_udp_panel(ctx, layout[3]);
        self.draw_server_info_players_panel(ctx, layout[4]);
    }

    fn draw_server_info_version_panel(&self, ctx: &mut Context, space: Rect) {
        let VersionInfo {version, compatibility} = ctx.state.server.version_info.as_ref().unwrap();
        let left = Spans::from(vec![
            Span::raw("Version:  "),
            Span::styled(version, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let left_panel = Paragraph::new(left).alignment(Alignment::Left);
        ctx.frame.render_widget(left_panel, space);

        let compatibility_color = match compatibility {
            Compatibility::Fully => Color::LightGreen,
            Compatibility::NotExact => Color::Yellow,
            Compatibility::None => unreachable!(),
        };

        let right = Span::styled("Compatible", Style::default().fg(compatibility_color));

        let right_panel = Paragraph::new(right).alignment(Alignment::Right);
        ctx.frame.render_widget(right_panel, space);
    }

    fn draw_server_info_map_size_panel(&self, ctx: &mut Context, space: Rect) {
        if let Some(static_game_info) = &ctx.state.server.game.static_info {
            let map_size = static_game_info.map_size;
            let dimension = format!("{}x{}", map_size, map_size);
            let left = Spans::from(vec![
                Span::raw("Map size: "),
                Span::styled(dimension, Style::default().add_modifier(Modifier::BOLD)),
            ]);

            let left_panel = Paragraph::new(left).alignment(Alignment::Left);
            ctx.frame.render_widget(left_panel, space);
        }
    }

    fn draw_server_info_points_panel(&self, ctx: &mut Context, space: Rect) {
        if let Some(static_game_info) = &ctx.state.server.game.static_info {
            let points = static_game_info.winner_points.to_string();
            let left = Spans::from(vec![
                Span::raw("Points:   "),
                Span::styled(points, Style::default().add_modifier(Modifier::BOLD)),
            ]);

            let left_panel = Paragraph::new(left).alignment(Alignment::Left);
            ctx.frame.render_widget(left_panel, space);
        }
    }

    fn draw_server_info_udp_panel(&self, ctx: &mut Context, space: Rect) {
        if let Some(udp_port) = ctx.state.server.udp_port {
            let left = Spans::from(vec![
                Span::raw("UDP port: "),
                Span::styled(udp_port.to_string(), Style::default().add_modifier(Modifier::BOLD)),
            ]);

            let left_panel = Paragraph::new(left).alignment(Alignment::Left);
            ctx.frame.render_widget(left_panel, space);

            if let Some(LoginStatus::Logged(..)) = ctx.state.server.game.login_status {
                let (status_message, status_color) =
                match ctx.state.server.udp_confirmed {
                    Some(value) => match value {
                        true => ("Available", Color::LightGreen),
                        false => ("Not available", Color::Yellow),
                    }
                    None => ("Checking...", Color::LightYellow)
                };

                let right = Span::styled(status_message, Style::default().fg(status_color));

                let right_panel = Paragraph::new(right).alignment(Alignment::Right);
                ctx.frame.render_widget(right_panel, space);
            }
        }
    }

    fn draw_server_info_players_panel(&self, ctx: &mut Context, space: Rect) {
        if let Some(static_game_info) = &ctx.state.server.game.static_info {
            let current_players_number = ctx.state.server.game.logged_players.len();

            let players_ratio = format!("{}/{}", current_players_number, static_game_info.players_number);
            let left = Spans::from(vec![
                Span::raw("Players:  "),
                Span::styled(players_ratio, Style::default().add_modifier(Modifier::BOLD)),
            ]);

            let left_panel = Paragraph::new(left).alignment(Alignment::Left);
            ctx.frame.render_widget(left_panel, space);

            let (status_message, status_color) =
            if current_players_number == static_game_info.players_number {
                let login_status = ctx.state.server.game.login_status;
                if let Some(LoginStatus::Logged(..)) = login_status {
                    ("Ready!", Color::LightGreen) // TODO: Add a number with the remining time: Ready in 3.. 2.. 1..
                }
                else {
                    ("Completed", Color::LightRed)
                }
            }
            else {
                ("Waiting...", Color::LightYellow)
            };

            let right = Span::styled(status_message, Style::default().fg(status_color));

            let right_panel = Paragraph::new(right).alignment(Alignment::Right);
            ctx.frame.render_widget(right_panel, space);
        }
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
