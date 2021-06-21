use super::util::{self, InputText, InputCapitalLetter};
use super::waiting_room::{WaitingRoom, WaitingRoomWidget};

use crate::client::configuration::{Config};
use crate::client::state::{State, VersionInfo, GameStatus};
use crate::client::server_proxy::{ConnectionStatus};
use crate::client::store::{Store, Action};
use crate::client::terminal::input::{InputEvent};
use crate::client::terminal::renderer::{Cursor};

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

pub struct Menu {
    server_addr_input: InputText,
    character_symbol_input: InputCapitalLetter,
    waiting_room: WaitingRoom,
}

impl Menu {
    pub fn new(config: &Config) -> Menu {
        Menu {
            server_addr_input: InputText::new(config.server_addr.map(|addr| addr.to_string())),
            character_symbol_input: InputCapitalLetter::new(config.character),
            waiting_room: WaitingRoom::new(
                WaitingRoomPanelWidget::WIDTH - 2,
                ServerInfoPanelWidget::HEIGHT - 2,
            ),
        }
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
                        else if self.character_symbol_input.has_focus() {
                            if let Some(character) = self.character_symbol_input.content() {
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
                            return store.dispatch(Action::CloseApp)
                        }
                    }
                    _ => (),
                }
                self.server_addr_input.key_pressed(key_event);
                self.character_symbol_input.key_pressed(key_event);
            }
            InputEvent::ResizeDisplay(_, _) => {}
        }
    }

    pub fn update(&mut self, state: &State) {
        let (server_addr_focus, character_focus) = if !state.server.connection_status.is_connected()
            || !state.server.has_compatible_version()
        {
            (true, false)
        }
        else if !state.user.is_logged() {
            (false, true)
        }
        else {
            (false, false)
        };

        self.character_symbol_input.focus(character_focus);
        self.server_addr_input.focus(server_addr_focus);
        self.waiting_room.update(state);
    }
}

#[derive(derive_new::new)]
pub struct MenuWidget<'a> {
    state: &'a State,
    menu: &'a Menu,
}

impl<'a> MenuWidget<'a> {
    pub fn dimension() -> (u16, u16) {
        (
            TitlePanelWidget::MAIN_TITLE.find('\n').unwrap() as u16,
            TitlePanelWidget::dimension().1
                + VersionPanelWidget::HEIGHT
                + ClientInfoPanelWidget::HEIGHT
                + ServerInfoPanelWidget::HEIGHT
                + NotificationLabelWidget::HEIGHT
                + 5,
        ) // margin sum
    }
}

impl StatefulWidget for MenuWidget<'_> {
    type State = Cursor;
    fn render(self, area: Rect, buffer: &mut Buffer, cursor: &mut Cursor) {
        let column = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(TitlePanelWidget::dimension().1),
                    Constraint::Length(VersionPanelWidget::HEIGHT),
                    Constraint::Length(2), // Margin
                    Constraint::Length(ClientInfoPanelWidget::HEIGHT),
                    Constraint::Length(2), // Margin
                    Constraint::Length(ServerInfoPanelWidget::HEIGHT),
                    Constraint::Length(1), // Margin
                    Constraint::Length(NotificationLabelWidget::HEIGHT),
                ]
                .as_ref(),
            )
            .split(area);

        TitlePanelWidget.render(column[0], buffer);

        VersionPanelWidget.render(column[1], buffer);

        ClientInfoPanelWidget::new(self.state, self.menu).render(column[3], buffer, cursor);

        let row = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(2)
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Length(2), // Margin
                    Constraint::Length(WaitingRoomPanelWidget::WIDTH),
                ]
                .as_ref(),
            )
            .split(column[5]);

        ServerInfoPanelWidget::new(self.state).render(row[0], buffer);

        WaitingRoomPanelWidget::new(self.state, self.menu).render(row[2], buffer);

        NotificationLabelWidget::new(self.state, self.menu).render(column[7], buffer);
    }
}

struct TitlePanelWidget;

impl TitlePanelWidget {
    const MAIN_TITLE: &'static str = concat!(
        r"   _____                .__.__   _____                                ",
        "\n",
        r"  /  _  \   ______ ____ |__|__| /  _  \_______   ____   ____ _____    ",
        "\n",
        r" /  /_\  \ /  ___// ___\|  |  |/  /_\  \_  __ \_/ __ \ /    \\__  \   ",
        "\n",
        r"/    |    \\___ \\  \___|  |  /    |    \  | \/\  ___/|   |  \/ __ \_ ",
        "\n",
        r"\____|__  /______>\_____>__|__\____|__  /__|    \_____>___|__(______/ ",
        "\n",
        r"        \/                            \/",
        "\n",
    );

    fn dimension() -> (u16, u16) {
        (
            Self::MAIN_TITLE.find('\n').unwrap() as u16,
            Self::MAIN_TITLE.chars().filter(|&c| c == '\n').count() as u16,
        )
    }
}

impl Widget for TitlePanelWidget {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(Self::MAIN_TITLE)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left)
            .render(area, buffer);
    }
}

struct VersionPanelWidget;

impl VersionPanelWidget {
    const X_OFFSET: u16 = 54;
    const HEIGHT: u16 = 1;
}

impl Widget for VersionPanelWidget {
    fn render(self, mut area: Rect, buffer: &mut Buffer) {
        if area.width >= Self::X_OFFSET {
            area.x += Self::X_OFFSET;
            area.width -= Self::X_OFFSET;

            let message = format!("version: {}", version::current());
            let version = Span::styled(message, Style::default().fg(Color::Gray));

            Paragraph::new(version).alignment(Alignment::Left).render(area, buffer);
        }
    }
}

#[derive(derive_new::new)]
struct ClientInfoPanelWidget<'a> {
    state: &'a State,
    menu: &'a Menu,
}

impl ClientInfoPanelWidget<'_> {
    const INITIAL_CURSOR: u16 = 17;
    const HEIGHT: u16 = 2;
}

impl StatefulWidget for ClientInfoPanelWidget<'_> {
    type State = Cursor;
    fn render(self, area: Rect, buffer: &mut Buffer, cursor: &mut Cursor) {
        let column = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(4)
            .constraints((0..Self::HEIGHT).map(|_| Constraint::Length(1)).collect::<Vec<_>>())
            .split(area);

        ServerAddressLabelWidget::new(self.state, self.menu).render(column[0], buffer, cursor);

        CharacterLabelWidget::new(self.state, self.menu).render(column[1], buffer, cursor);
    }
}

#[derive(derive_new::new)]
struct ServerAddressLabelWidget<'a> {
    state: &'a State,
    menu: &'a Menu,
}

impl StatefulWidget for ServerAddressLabelWidget<'_> {
    type State = Cursor;
    fn render(self, area: Rect, buffer: &mut Buffer, cursor: &mut Cursor) {
        let server_addrees_msg = Spans::from(vec![
            Span::raw("Server address:  "),
            Span::styled(
                self.menu.server_addr_input.content(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]);

        Paragraph::new(server_addrees_msg).alignment(Alignment::Left).render(area, buffer);

        let (message, hint_color) = if self.menu.server_addr_input.content().is_empty() {
            ("Not connected", Color::DarkGray)
        }
        else {
            match self.menu.server_addr_input.content().parse::<SocketAddr>() {
                Err(_) => ("Use 'ip:port' syntax", Color::Yellow),
                Ok(_) => match self.state.server.connection_status {
                    ConnectionStatus::Connected => ("Connected", Color::LightGreen),
                    ConnectionStatus::NotConnected => ("Not connected", Color::DarkGray),
                    ConnectionStatus::NotFound => ("Server not found", Color::LightRed),
                    ConnectionStatus::Lost => {
                        if !self.state.server.has_compatible_version() {
                            ("Version error", Color::LightRed)
                        }
                        else {
                            ("Connection lost", Color::LightRed)
                        }
                    }
                },
            }
        };

        let hint = Span::styled(message, Style::default().fg(hint_color));
        Paragraph::new(hint).alignment(Alignment::Right).render(area, buffer);

        if let Some(ref pos) = self.menu.server_addr_input.cursor_position() {
            cursor.set(area.x + ClientInfoPanelWidget::INITIAL_CURSOR + *pos as u16, area.y);
        }
    }
}

#[derive(derive_new::new)]
struct CharacterLabelWidget<'a> {
    state: &'a State,
    menu: &'a Menu,
}

impl StatefulWidget for CharacterLabelWidget<'_> {
    type State = Cursor;
    fn render(self, area: Rect, buffer: &mut Buffer, cursor: &mut Cursor) {
        let character = self.menu.character_symbol_input.content().unwrap_or(' ');

        let character_msg = Spans::from(vec![
            Span::raw("Character name:  "),
            Span::styled(character.to_string(), Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(character_msg).alignment(Alignment::Left).render(area, buffer);

        let (status_message, status_color) = if self.state.user.is_logged() {
            ("Logged", Color::LightGreen)
        }
        else if self.state.server.logged_players.contains(&character) {
            ("Name already chosen", Color::LightRed)
        }
        else if self.state.server.is_full() {
            ("Player limit reached", Color::LightYellow)
        }
        else if let Some(LoginStatus::InvalidPlayerName) = self.state.user.login_status {
            ("Invalid player name", Color::LightRed)
        }
        else {
            ("Not logged", Color::DarkGray)
        };

        let hint = Span::styled(status_message, Style::default().fg(status_color));
        Paragraph::new(hint).alignment(Alignment::Right).render(area, buffer);

        if self.menu.character_symbol_input.has_focus() {
            cursor.set(area.x + ClientInfoPanelWidget::INITIAL_CURSOR, area.y);
        }
    }
}

#[derive(derive_new::new)]
struct ServerInfoPanelWidget<'a> {
    state: &'a State,
}

impl ServerInfoPanelWidget<'_> {
    const HEIGHT: u16 = 2 + ServerInfoWithContentPanelWidget::HEIGHT;
}

impl Widget for ServerInfoPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled("Server info", Style::default().add_modifier(Modifier::BOLD)))
            .render(area, buffer);

        let inner = area.inner(&Margin { vertical: 1, horizontal: 2 });

        if self.state.server.game_info.is_some() {
            ServerInfoWithContentPanelWidget::new(self.state).render(inner, buffer);
        }
        else {
            ServerInfoWithoutContentPanelWidget::new(self.state).render(inner, buffer);
        }
    }
}

#[derive(derive_new::new)]
struct ServerInfoWithoutContentPanelWidget<'a> {
    state: &'a State,
}

impl Widget for ServerInfoWithoutContentPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let message =
            if let Some(VersionInfo { version, compatibility }) = &self.state.server.version_info {
                if !compatibility.is_compatible() {
                    Spans::from(vec![
                        Span::styled(
                            "Incompatible versions. Server:",
                            Style::default().fg(Color::LightRed),
                        ),
                        Span::styled(
                            version,
                            Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
                        ),
                    ])
                }
                else {
                    Spans::from(Span::styled(
                        "Loading information...",
                        Style::default().fg(Color::Gray),
                    ))
                }
            }
            else {
                Spans::from(Span::styled("Without information", Style::default().fg(Color::Gray)))
            };

        Paragraph::new(message)
            .alignment(Alignment::Center)
            .render(util::vertically_centered(area, 1), buffer);
    }
}

#[derive(derive_new::new)]
struct ServerInfoWithContentPanelWidget<'a> {
    state: &'a State,
}

impl ServerInfoWithContentPanelWidget<'_> {
    const HEIGHT: u16 = 5;
}

impl Widget for ServerInfoWithContentPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let column = Layout::default()
            .direction(Direction::Vertical)
            .constraints((0..Self::HEIGHT).map(|_| Constraint::Length(1)).collect::<Vec<_>>())
            .split(area);

        ServerInfoVersionLabelWidget::new(self.state).render(column[0], buffer);

        ServerInfoUdpLabelWidget::new(self.state).render(column[1], buffer);

        ServerInfoMapSizeLabelWidget::new(self.state).render(column[2], buffer);

        ServerInfoPointsLabelWidget::new(self.state).render(column[3], buffer);

        ServerInfoPlayersLabelWidget::new(self.state).render(column[4], buffer);
    }
}

#[derive(derive_new::new)]
struct ServerInfoVersionLabelWidget<'a> {
    state: &'a State,
}

impl Widget for ServerInfoVersionLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let VersionInfo { version, compatibility } =
            self.state.server.version_info.as_ref().unwrap();

        let left = Spans::from(vec![
            Span::raw("Version:  "),
            Span::styled(version, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(left).alignment(Alignment::Left).render(area, buffer);

        let compatibility_color = match compatibility {
            Compatibility::Fully => Color::LightGreen,
            Compatibility::NotExact => Color::Yellow,
            Compatibility::None => unreachable!(),
        };

        let right = Span::styled("Compatible", Style::default().fg(compatibility_color));

        Paragraph::new(right).alignment(Alignment::Right).render(area, buffer);
    }
}

#[derive(derive_new::new)]
struct ServerInfoUdpLabelWidget<'a> {
    state: &'a State,
}

impl Widget for ServerInfoUdpLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let udp_port = self.state.server.udp_port.unwrap();
        let left = Spans::from(vec![
            Span::raw("UDP port: "),
            Span::styled(udp_port.to_string(), Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(left).alignment(Alignment::Left).render(area, buffer);

        if self.state.user.is_logged() {
            let (status_message, status_color) = match self.state.server.udp_confirmed {
                Some(value) => match value {
                    true => ("Checked", Color::LightGreen),
                    false => ("Not available", Color::Yellow),
                },
                None => ("Checking...", Color::LightYellow),
            };

            let right = Span::styled(status_message, Style::default().fg(status_color));

            Paragraph::new(right).alignment(Alignment::Right).render(area, buffer);
        }
    }
}

#[derive(derive_new::new)]
struct ServerInfoMapSizeLabelWidget<'a> {
    state: &'a State,
}

impl Widget for ServerInfoMapSizeLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let game_info = self.state.server.game_info.as_ref().unwrap();
        let map_size = game_info.map_size;
        let dimension = format!("{}x{}", map_size, map_size);
        let left = Spans::from(vec![
            Span::raw("Map size: "),
            Span::styled(dimension, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(left).alignment(Alignment::Left).render(area, buffer);
    }
}

#[derive(derive_new::new)]
struct ServerInfoPointsLabelWidget<'a> {
    state: &'a State,
}

impl Widget for ServerInfoPointsLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let game_info = self.state.server.game_info.as_ref().unwrap();
        let points = game_info.winner_points.to_string();
        let left = Spans::from(vec![
            Span::raw("Points:   "),
            Span::styled(points, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(left).alignment(Alignment::Left).render(area, buffer);
    }
}

#[derive(derive_new::new)]
struct ServerInfoPlayersLabelWidget<'a> {
    state: &'a State,
}

impl Widget for ServerInfoPlayersLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let game_info = self.state.server.game_info.as_ref().unwrap();
        let current_players_number = self.state.server.logged_players.len();

        let players_ratio = format!("{}/{}", current_players_number, game_info.players_number);
        let left = Spans::from(vec![
            Span::raw("Players:  "),
            Span::styled(players_ratio, Style::default().add_modifier(Modifier::BOLD)),
        ]);

        Paragraph::new(left).alignment(Alignment::Left).render(area, buffer);

        let (status_message, status_color) = if current_players_number == game_info.players_number {
            if self.state.user.is_logged() {
                ("Ready", Color::LightGreen)
            }
            else {
                ("Completed", Color::LightRed)
            }
        }
        else {
            ("Waiting other players...", Color::LightYellow)
        };

        let right = Span::styled(status_message, Style::default().fg(status_color));

        Paragraph::new(right).alignment(Alignment::Right).render(area, buffer);
    }
}

#[derive(derive_new::new)]
struct WaitingRoomPanelWidget<'a> {
    state: &'a State,
    menu: &'a Menu,
}

impl WaitingRoomPanelWidget<'_> {
    const WIDTH: u16 = 20;
}

impl Widget for WaitingRoomPanelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let message = if self.state.server.is_full() && !self.state.user.is_logged() {
            "Players at game"
        }
        else {
            "Waiting room"
        };

        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(message, Style::default().add_modifier(Modifier::BOLD)))
            .render(area, buffer);

        let inner = area.inner(&Margin { vertical: 1, horizontal: 1 });

        WaitingRoomWidget::new(&self.menu.waiting_room).render(inner, buffer);
    }
}

#[derive(derive_new::new)]
struct NotificationLabelWidget<'a> {
    state: &'a State,
    menu: &'a Menu,
}

impl NotificationLabelWidget<'_> {
    const HEIGHT: u16 = 2;
}

impl Widget for NotificationLabelWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let enter = Span::styled(
            " <Enter> ",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
        );

        let esc = Span::styled(
            " <Esc> ",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow),
        );

        let messages = if !self.state.server.is_connected()
            || !self.state.server.has_compatible_version()
        {
            vec![
                Spans::from(vec![Span::raw("Press"), enter, Span::raw("to connect to server")]),
                Spans::from(vec![Span::raw("Press"), esc, Span::raw("to exit from asciiarena")]),
            ]
        }
        else if !self.state.user.is_logged() {
            vec![
                if self.menu.character_symbol_input.content().is_none() {
                    Spans::from(vec![Span::raw("Choose a character (an ascii uppercase letter)")])
                }
                else {
                    Spans::from(vec![
                        Span::raw("Press"),
                        enter,
                        Span::raw("to login with the character"),
                    ])
                },
                Spans::from(vec![
                    Span::raw("Press"),
                    esc,
                    Span::raw("to disconnect from the server"),
                ]),
            ]
        }
        else if let GameStatus::Started = self.state.server.game.status {
            let waiting_secs = match self.state.server.game.next_arena_timestamp {
                Some(timestamp) => {
                    timestamp.saturating_duration_since(Instant::now()).as_secs() + 1
                }
                None => 0,
            };

            let style = Style::default().fg(Color::LightCyan);

            vec![Spans::from(vec![
                Span::styled("Starting game in ", style),
                Span::styled(waiting_secs.to_string(), style.add_modifier(Modifier::BOLD)),
                Span::styled("...", style),
            ])]
        }
        else {
            vec![Spans::from(vec![Span::raw("Press"), esc, Span::raw("to logout the character")])]
        };

        Paragraph::new(messages).alignment(Alignment::Center).render(area, buffer);
    }
}
