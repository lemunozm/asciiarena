use crate::client::configuration::{Config};
use crate::client::state::{State, VersionInfo};
use crate::client::server_proxy::{ConnectionStatus};
use crate::client::store::{Store, Action};
use crate::client::gui::input::{InputEvent};
use crate::client::gui::element::{Context, GuiElement};
use crate::client::gui::widgets::{InputTextWidget, InputCapitalLetterWidget};

use crate::version::{self, Compatibility};
use crate::message::{LoginStatus};

use tui::widgets::{Block, Borders, BorderType, Paragraph};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment, Margin};
use tui::style::{Style, Modifier, Color};
use tui::text::{Span, Spans};

use crossterm::event::{KeyCode};

use std::net::{SocketAddr};
use std::time::{Instant};

const MAIN_TITLE: &'static str = concat!(
r"   _____                .__.__   _____                                ", "\n",
r"  /  _  \   ______ ____ |__|__| /  _  \_______   ____   ____ _____    ", "\n",
r" /  /_\  \ /  ___// ___\|  |  |/  /_\  \_  __ \_/ __ \ /    \\__  \   ", "\n",
r"/    |    \\___ \\  \___|  |  /    |    \  | \/\  ___/|   |  \/ __ \_ ", "\n",
r"\____|__  /______>\_____>__|__\____|__  /__|    \_____>___|__(______/ ", "\n",
r"        \/                            \/", "\n",
);
pub const DIMENSION: (u16, u16) = (70, 23);

pub struct Menu {
    server_addr_input: InputTextWidget,
    character_input: InputCapitalLetterWidget,
}

impl GuiElement for Menu {
    fn process_event(&mut self, store: &mut Store, event: InputEvent) {
        match event {
            InputEvent::KeyPressed(key_event) => {
                match key_event.code {
                    KeyCode::Enter => {
                        if self.server_addr_input.has_focus() {
                            let content = self.server_addr_input.content();
                            if let Ok(addr) = content.parse::<SocketAddr>() {
                                store.dispatch(Action::Connect(addr));
                            }
                        }
                        else if self.character_input.has_focus() {
                            if let Some(character) = self.character_input.content() {
                                store.dispatch(Action::Login(character));
                            }
                        }
                    }
                    KeyCode::Esc => {
                        if let Some(LoginStatus::Logged(..)) = store.state().user.login_status {
                            if !store.state().server.is_full() {
                                store.dispatch(Action::Logout);
                            }
                        }
                        else if store.state().server.connection_status.is_connected() {
                            store.dispatch(Action::Disconnect);
                        }
                        else {
                            return store.dispatch(Action::Close);
                        }
                    },
                    _ => (),
                }
                self.server_addr_input.key_pressed(key_event);
                self.character_input.key_pressed(key_event);
            },
            InputEvent::ResizeDisplay(_, _) => {},
        }
    }

    fn update(&mut self, state: &State) {
        let (server_addr_focus, character_focus) =

        if !state.server.connection_status.is_connected()
        || !state.server.has_compatible_version() {
            (true, false)
        }
        else if !state.user.is_logged() {
            (false, true)
        }
        else {
            (false, false)
        };

        self.character_input.focus(character_focus);
        self.server_addr_input.focus(server_addr_focus);
    }

    fn render(&self, ctx: &mut Context, space: Rect) {
        let gui_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(MAIN_TITLE.chars().filter(|&c| c == '\n').count() as u16),
                Constraint::Length(3), // Margin
                Constraint::Length(2),
                Constraint::Length(2), // Margin
                Constraint::Length(7),
                Constraint::Length(1), // Margin
                Constraint::Length(2),
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
}

impl Menu {
    pub fn new(config: &Config) -> Menu {
        Menu {
            server_addr_input: InputTextWidget::new(
                config.server_addr.map(|addr| addr.to_string()) //TODO: into()?
            ),
            character_input: InputCapitalLetterWidget::new(config.character),
        }
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
        self.draw_character_panel(ctx, spaces[1]);
    }

    fn draw_server_address_panel(&self, ctx: &mut Context, space: Rect) {
        let server_addrees = Spans::from(vec![
            Span::raw("Server address:  "),
            Span::styled(
                self.server_addr_input.content(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]);

        let left_panel = Paragraph::new(server_addrees).alignment(Alignment::Left);
        ctx.frame.render_widget(left_panel, space);

        let (message, hint_color) =
        if self.server_addr_input.content().is_empty() {
            ("Not connected", Color::DarkGray)
        }
        else {
            match self.server_addr_input.content().parse::<SocketAddr>() {
                Err(_) => ("Use 'ip:port' syntax", Color::Yellow),
                Ok(_) => match ctx.state.server.connection_status {
                    ConnectionStatus::Connected => ("Connected", Color::LightGreen),
                    ConnectionStatus::NotConnected => {
                        ("Not connected", Color::DarkGray)
                    }
                    ConnectionStatus::NotFound => ("Server not found", Color::LightRed),
                    ConnectionStatus::Lost => {
                        if !ctx.state.server.has_compatible_version() {
                            ("Version error", Color::LightRed)
                        }
                        else {
                            ("Connection lost", Color::LightRed)
                        }
                    }
                }
            }
        };

        let hint = Span::styled(message, Style::default().fg(hint_color));
        let right_panel = Paragraph::new(hint).alignment(Alignment::Right);
        ctx.frame.render_widget(right_panel, space);

        if let Some(ref cursor) = self.server_addr_input.cursor_position() {
            ctx.frame.set_cursor(space.x + 17 + *cursor as u16, space.y);
        }
    }

    fn draw_character_panel(&self, ctx: &mut Context, space: Rect) {
        let character_input = match self.character_input.content() {
            Some(character) => character.to_string(),
            None => String::new(),
        };

        let character = Spans::from(vec![
            Span::raw("Character name:  "),
            Span::styled(character_input, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let left_panel = Paragraph::new(character).alignment(Alignment::Left);
        ctx.frame.render_widget(left_panel, space);

        let (status_message, status_color) =
        if let Some(login_status) = ctx.state.user.login_status {
            match login_status {
                LoginStatus::Logged(_, _) => {
                    ("Logged", Color::LightGreen)
                },
                LoginStatus::InvalidPlayerName => {
                    ("Invalid player name", Color::LightRed)
                },
                LoginStatus::AlreadyLogged => {
                    ("Name already chosen", Color::LightRed)
                },
                LoginStatus::PlayerLimit => {
                    ("Player limit reached", Color::LightYellow)
                },
            }
        }
        else {
            ("Not logged", Color::DarkGray)
        };

        let hint = Span::styled(status_message, Style::default().fg(status_color));
        let right_panel = Paragraph::new(hint).alignment(Alignment::Right);
        ctx.frame.render_widget(right_panel, space);

        if self.character_input.has_focus() {
            ctx.frame.set_cursor(space.x + 17, space.y);
        }
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
        self.draw_server_info_udp_panel(ctx, layout[1]);
        self.draw_server_info_map_size_panel(ctx, layout[2]);
        self.draw_server_info_points_panel(ctx, layout[3]);
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
        if let Some(static_game_info) = &ctx.state.server.game_info {
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
        if let Some(static_game_info) = &ctx.state.server.game_info {
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

            if let Some(LoginStatus::Logged(..)) = ctx.state.user.login_status {
                let (status_message, status_color) =
                match ctx.state.server.udp_confirmed {
                    Some(value) => match value {
                        true => ("Checked", Color::LightGreen),
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
        if let Some(static_game_info) = &ctx.state.server.game_info {
            let current_players_number = ctx.state.server.logged_players.len();

            let players_ratio = format!("{}/{}", current_players_number, static_game_info.players_number);
            let left = Spans::from(vec![
                Span::raw("Players:  "),
                Span::styled(players_ratio, Style::default().add_modifier(Modifier::BOLD)),
            ]);

            let left_panel = Paragraph::new(left).alignment(Alignment::Left);
            ctx.frame.render_widget(left_panel, space);

            let (status_message, status_color) =
            if current_players_number == static_game_info.players_number {
                let login_status = ctx.state.user.login_status;
                if let Some(LoginStatus::Logged(..)) = login_status {
                    let waiting_secs = match ctx.state.server.game.next_arena_timestamp {
                        Some(timestamp) => {
                            timestamp.saturating_duration_since(Instant::now()).as_secs()
                        }
                        None => 0
                    };
                    let message = format!("Ready in {}...", waiting_secs);
                    (message, Color::LightGreen)
                }
                else {
                    ("Completed".into(), Color::LightRed)
                }
            }
            else {
                ("Waiting...".into(), Color::LightYellow)
            };

            let right = Span::styled(status_message, Style::default().fg(status_color));

            let right_panel = Paragraph::new(right).alignment(Alignment::Right);
            ctx.frame.render_widget(right_panel, space);
        }
    }

    fn draw_waiting_room_panel(&self, ctx: &mut Context, space: Rect) {
        let panel = Paragraph::new("")
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
        let enter = Span::styled(" <Enter> ", Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan));

        let esc = Span::styled(" <Esc> ", Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Yellow));

        let messages =
        if !ctx.state.server.is_connected() || !ctx.state.server.has_compatible_version() {
            vec![
                Spans::from(vec![
                    Span::raw("Press"), enter, Span::raw("to connect to server")
                ]),
                Spans::from(vec![
                    Span::raw("Press"), esc, Span::raw("to exit from asciiarena")
                ]),
            ]
        }
        else if !ctx.state.user.is_logged() {
            vec![
                if self.character_input.content().is_none() {
                    Spans::from(vec![
                        Span::raw("Choose a character (an ascii uppercase letter)"),
                    ])
                }
                else {
                    Spans::from(vec![
                        Span::raw("Press"), enter, Span::raw("to login with the character")
                    ])
                },
                Spans::from(vec![
                    Span::raw("Press"), esc, Span::raw("to disconnect from the server")
                ]),
            ]
        }
        else {
            vec![
                Spans::from(vec![
                    Span::raw("Press"), esc, Span::raw("to logout the character")
                ]),
            ]
        };

        //" Starting arena in 2..."
        let panel = Paragraph::new(messages)
            .alignment(Alignment::Center);

        ctx.frame.render_widget(panel, space);
    }
}
