use crate::client::configuration::{Config};
use crate::client::state::{State, VersionInfo};
use crate::client::server_proxy::{ConnectionStatus};
use crate::client::store::{Store, Action};
use crate::client::gui::input::{InputEvent};
use crate::client::gui::widgets::{InputText, InputCapitalLetter};
use crate::client::gui::waiting_room::{WaitingRoom, WaitingRoomWidget};

use crate::version::{self, Compatibility};
use crate::message::{LoginStatus};

use tui::buffer::{Buffer};
use tui::widgets::{Block, Borders, BorderType, Paragraph, Widget, StatefulWidget};
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

pub const WAITING_ROOM_DIMENSION: (u16, u16) = (20, 7);
pub const DIMENSION: (u16, u16) = (70, 23);

pub struct Menu {
    server_addr_input: InputText,
    character_input: InputCapitalLetter,
    waiting_room: WaitingRoom,
}

impl Menu {
    pub fn new(config: &Config) -> Menu {
        Menu {
            server_addr_input: InputText::new(
                config.server_addr.map(|addr| addr.to_string())
            ),
            character_input: InputCapitalLetter::new(config.character),
            waiting_room: WaitingRoom::new(
                WAITING_ROOM_DIMENSION.0 - 2,
                WAITING_ROOM_DIMENSION.1 - 2
            ),
        }
    }

    pub fn dimension() {
        //TODO
    }

    pub fn process_event(&mut self, store: &mut Store, event: InputEvent) {
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

    pub fn update(&mut self, state: &State) {
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
        self.waiting_room.update(state);
    }
}

pub struct MenuWidget<'a> {
    state: &'a State,
    menu: &'a Menu
}

impl<'a> MenuWidget<'a> {
    pub fn new(state: &'a State, menu: &'a Menu) -> MenuWidget<'a> {
        MenuWidget { state, menu }
    }
}

impl StatefulWidget for MenuWidget<'_> {
    type State = Option<(u16, u16)>;
    fn render(self, area: Rect, buffer: &mut Buffer, cursor: &mut Option<(u16, u16)>) {
        let column = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(MAIN_TITLE.chars().filter(|&c| c == '\n').count() as u16),
                Constraint::Length(1),
                Constraint::Length(2), // Margin
                Constraint::Length(2),
                Constraint::Length(2), // Margin
                Constraint::Length(WAITING_ROOM_DIMENSION.1),
                Constraint::Length(1), // Margin
                Constraint::Length(2),
            ].as_ref())
            .split(area);

        TitlePanelWidget
            .render(column[0], buffer);

        VersionPanelWidget
            .render(column[1], buffer);

        ClientInfoPanelWidget{state: self.state, menu: self.menu}
            .render(column[3], buffer, cursor);

        let row = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(2)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(2), // Margin
                Constraint::Length(WAITING_ROOM_DIMENSION.0),
            ].as_ref())
            .split(column[5]);

        ServerInfoPanelWidget{state: self.state}
            .render(row[0], buffer);

        WaitingRoomPanelWidget{menu: self.menu}
            .render(row[2], buffer);

        NotifyPanelWidget{state: self.state, menu: self.menu}
            .render(column[7], buffer);
    }
}

struct TitlePanelWidget;

impl Widget for TitlePanelWidget {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(MAIN_TITLE)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left)
            .render(area, buffer);
    }
}

struct VersionPanelWidget;

impl Widget for VersionPanelWidget {
    fn render(self, mut area: Rect, buffer: &mut Buffer) {
        area.x += 54;
        area.width -= 54;

        let message = format!("version: {}", version::current());
        let version = Span::styled(message, Style::default().fg(Color::Gray));

        Paragraph::new(version)
            .alignment(Alignment::Left)
            .render(area, buffer);
    }
}

struct ClientInfoPanelWidget<'a> {state: &'a State, menu: &'a Menu}

impl ClientInfoPanelWidget<'_> {
    pub const INITIAL_CURSOR: u16 = 17;
}

impl StatefulWidget for ClientInfoPanelWidget<'_> {
    type State = Option<(u16, u16)>;
    fn render(self, area: Rect, buffer: &mut Buffer, cursor: &mut Option<(u16, u16)>) {
        let column = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(4)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
            ].as_ref())
            .split(area);

        ServerAddressLabelWidget{state: self.state, menu: self.menu}
            .render(column[0], buffer, cursor);

        CharacterLabelWidget{state: self.state, menu: self.menu}
            .render(column[1], buffer, cursor);
    }
}

struct ServerAddressLabelWidget<'a> {state: &'a State, menu: &'a Menu}

impl StatefulWidget for ServerAddressLabelWidget<'_> {
    type State = Option<(u16, u16)>;
    fn render(self, area: Rect, buffer: &mut Buffer, cursor: &mut Option<(u16, u16)>) {
        let server_addrees = Spans::from(vec![
            Span::raw("Server address:  "),
            Span::styled(
                self.menu.server_addr_input.content(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]);

        Paragraph::new(server_addrees)
            .alignment(Alignment::Left)
            .render(area, buffer);

        let (message, hint_color) =
        if self.menu.server_addr_input.content().is_empty() {
            ("Not connected", Color::DarkGray)
        }
        else {
            match self.menu.server_addr_input.content().parse::<SocketAddr>() {
                Err(_) => ("Use 'ip:port' syntax", Color::Yellow),
                Ok(_) => match self.state.server.connection_status {
                    ConnectionStatus::Connected => ("Connected", Color::LightGreen),
                    ConnectionStatus::NotConnected => {
                        ("Not connected", Color::DarkGray)
                    }
                    ConnectionStatus::NotFound => ("Server not found", Color::LightRed),
                    ConnectionStatus::Lost => {
                        if !self.state.server.has_compatible_version() {
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
        Paragraph::new(hint)
            .alignment(Alignment::Right)
            .render(area, buffer);

        if let Some(ref pos) = self.menu.server_addr_input.cursor_position() {
            *cursor = Some((area.x + ClientInfoPanelWidget::INITIAL_CURSOR + *pos as u16, area.y));
        }
    }
}

struct CharacterLabelWidget<'a> {state: &'a State, menu: &'a Menu}

impl StatefulWidget for CharacterLabelWidget<'_> {
    type State = Option<(u16, u16)>;
    fn render(self, area: Rect, buffer: &mut Buffer, cursor: &mut Option<(u16, u16)>) {
        let character_input = match self.menu.character_input.content() {
            Some(character) => character.to_string(),
            None => String::new(),
        };

        let character = Spans::from(vec![
            Span::raw("Character name:  "),
            Span::styled(character_input, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(character)
            .alignment(Alignment::Left)
            .render(area, buffer);

        let (status_message, status_color) =
        if let Some(login_status) = self.state.user.login_status {
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
        Paragraph::new(hint)
            .alignment(Alignment::Right)
            .render(area, buffer);

        if self.menu.character_input.has_focus() {
            *cursor = Some((area.x + ClientInfoPanelWidget::INITIAL_CURSOR, area.y));
        }
    }
}

struct ServerInfoPanelWidget<'a> {state: &'a State}

impl Widget for ServerInfoPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                "Server info",
                Style::default().add_modifier(Modifier::BOLD)
            ))
            .render(area, buffer);

        let inner = area.inner(&Margin {vertical: 1, horizontal: 1});

        if self.state.server.game_info.is_some() {
            ServerInfoWithContentPanelWidget{state: self.state}
                .render(inner, buffer);
        }
        else {
            ServerInfoWithoutContentPanelWidget{state: self.state}
                .render(inner, buffer);
        }
    }
}

struct ServerInfoWithoutContentPanelWidget<'a> {state: &'a State}

impl Widget for ServerInfoWithoutContentPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let message =
        if let Some(VersionInfo{version, compatibility}) = &self.state.server.version_info {
            if !compatibility.is_compatible() {
                Spans::from(vec![
                    Span::styled("Incompatible versions: ", Style::default().fg(Color::LightRed)),
                    Span::styled(
                        version,
                        Style::default()
                            .fg(Color::LightRed)
                            .add_modifier(Modifier::BOLD)
                    ),
                ])
            }
            else {
                Spans::from(
                    Span::styled("Loading information...", Style::default().fg(Color::Gray))
                )
            }
        }
        else {
            Spans::from(
                Span::styled("Without information", Style::default().fg(Color::Gray))
            )
        };

        let vertical_center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // Margin
                Constraint::Length(1),
            ].as_ref())
            .split(area);

        Paragraph::new(message)
            .alignment(Alignment::Center)
            .render(vertical_center[1], buffer);
    }
}

struct ServerInfoWithContentPanelWidget<'a> {state: &'a State}

impl Widget for ServerInfoWithContentPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let column = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ].as_ref())
            .split(area);

        ServerInfoVersionLabelWidget{state: self.state}
            .render(column[0], buffer);

        ServerInfoUdpLabelWidget{state: self.state}
            .render(column[1], buffer);

        ServerInfoMapSizeLabelWidget{state: self.state}
            .render(column[2], buffer);

        ServerInfoPointsLabelWidget{state: self.state}
            .render(column[3], buffer);

        ServerInfoPlayersLabelWidget{state: self.state}
            .render(column[4], buffer);
    }
}

struct ServerInfoVersionLabelWidget<'a> {state: &'a State}

impl Widget for ServerInfoVersionLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let VersionInfo {version, compatibility} =
            self.state.server.version_info.as_ref().unwrap();

        let left = Spans::from(vec![
            Span::raw("Version:  "),
            Span::styled(version, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(left)
            .alignment(Alignment::Left)
            .render(area, buffer);

        let compatibility_color = match compatibility {
            Compatibility::Fully => Color::LightGreen,
            Compatibility::NotExact => Color::Yellow,
            Compatibility::None => unreachable!(),
        };

        let right = Span::styled("Compatible", Style::default().fg(compatibility_color));

        Paragraph::new(right)
            .alignment(Alignment::Right)
            .render(area, buffer);
    }
}

struct ServerInfoUdpLabelWidget<'a> {state: &'a State}

impl Widget for ServerInfoUdpLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let udp_port = self.state.server.udp_port.unwrap();
        let left = Spans::from(vec![
            Span::raw("UDP port: "),
            Span::styled(udp_port.to_string(), Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(left)
            .alignment(Alignment::Left)
            .render(area, buffer);

        let (status_message, status_color) =
        match self.state.server.udp_confirmed {
            Some(value) => match value {
                true => ("Checked", Color::LightGreen),
                false => ("Not available", Color::Yellow),
            }
            None => ("Checking...", Color::LightYellow)
        };

        let right = Span::styled(status_message, Style::default().fg(status_color));

        Paragraph::new(right)
            .alignment(Alignment::Right)
            .render(area, buffer);
    }
}

struct ServerInfoMapSizeLabelWidget<'a> {state: &'a State}

impl Widget for ServerInfoMapSizeLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let game_info = self.state.server.game_info.as_ref().unwrap();
        let map_size = game_info.map_size;
        let dimension = format!("{}x{}", map_size, map_size);
        let left = Spans::from(vec![
            Span::raw("Map size: "),
            Span::styled(dimension, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(left)
            .alignment(Alignment::Left)
            .render(area, buffer);
    }
}

struct ServerInfoPointsLabelWidget<'a> {state: &'a State}

impl Widget for ServerInfoPointsLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let game_info = self.state.server.game_info.as_ref().unwrap();
        let points = game_info.winner_points.to_string();
        let left = Spans::from(vec![
            Span::raw("Points:   "),
            Span::styled(points, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(left)
            .alignment(Alignment::Left)
            .render(area, buffer);
    }
}

struct ServerInfoPlayersLabelWidget<'a> {state: &'a State}

impl Widget for ServerInfoPlayersLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let game_info = self.state.server.game_info.as_ref().unwrap();
        let current_players_number = self.state.server.logged_players.len();

        let players_ratio = format!("{}/{}", current_players_number, game_info.players_number);
        let left = Spans::from(vec![
            Span::raw("Players:  "),
            Span::styled(players_ratio, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(left)
            .alignment(Alignment::Left)
            .render(area, buffer);

        let (status_message, status_color) =
        if current_players_number == game_info.players_number {
            if self.state.user.is_logged() {
                let waiting_secs = match self.state.server.game.next_arena_timestamp {
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
            ("Waiting other players...".into(), Color::LightYellow)
        };

        let right = Span::styled(status_message, Style::default().fg(status_color));

        Paragraph::new(right)
            .alignment(Alignment::Right)
            .render(area, buffer);
    }
}

struct WaitingRoomPanelWidget<'a>{menu: &'a Menu}

impl Widget for WaitingRoomPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                "Waiting room",
                Style::default().add_modifier(Modifier::BOLD)
            ))
            .render(area, buffer);

        let inner = area.inner(&Margin {vertical: 1, horizontal: 1});

        WaitingRoomWidget::new(&self.menu.waiting_room)
            .render(inner, buffer);
    }
}

struct NotifyPanelWidget<'a> {state: &'a State, menu: &'a Menu}

impl Widget for NotifyPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let enter = Span::styled(" <Enter> ", Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan));

        let esc = Span::styled(" <Esc> ", Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Yellow));

        let messages =
        if !self.state.server.is_connected() || !self.state.server.has_compatible_version() {
            vec![
                Spans::from(vec![
                    Span::raw("Press"), enter, Span::raw("to connect to server")
                ]),
                Spans::from(vec![
                    Span::raw("Press"), esc, Span::raw("to exit from asciiarena")
                ]),
            ]
        }
        else if !self.state.user.is_logged() {
            vec![
                if self.menu.character_input.content().is_none() {
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

        Paragraph::new(messages)
            .alignment(Alignment::Center)
            .render(area, buffer);
    }
}
