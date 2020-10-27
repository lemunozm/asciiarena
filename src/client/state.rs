use crate::version::{Compatibility};
use crate::message::{LoginStatus};

use super::input_widgets::{InputTextWidget, InputCapitalLetterWidget};
use super::server_proxy::{ConnectionStatus};

use std::net::{SocketAddr};

pub struct Config {
    pub server_addr: Option<SocketAddr>,
    pub character: Option<char>,
}

pub struct User {
    pub character: Option<char>,
    pub login_status: Option<LoginStatus>,
}

pub struct VersionInfo {
    pub version: String,
    pub compatibility: Compatibility,
}

pub struct Server {
    pub addr: Option<SocketAddr>,
    pub connection_status: ConnectionStatus,
    pub udp_port: Option<u16>,
    pub udp_confirmed: Option<bool>,
    pub version_info: Option<VersionInfo>,
    pub game: Game,
}

pub struct StaticGameInfo {
    pub players_number: usize,
    pub map_size: usize,
    pub winner_points: usize,
}

pub struct Game {
    pub static_info: Option<StaticGameInfo>,
    pub logged_players: Vec<char>,
}

impl Game {
    pub fn is_full(&self) -> bool {
        if let Some(StaticGameInfo {players_number, .. }) = self.static_info {
            if players_number == self.logged_players.len() {
                return true
            }
        }
        false
    }
}

pub struct MenuState {
    pub server_addr_input: InputTextWidget,
    pub character_input: InputCapitalLetterWidget,
}

impl MenuState {
    pub fn new(config: &super::Config) -> MenuState {
        MenuState {
            server_addr_input: InputTextWidget::new(
                config.server_addr.map(|addr| addr.to_string())
            ),
            character_input: InputCapitalLetterWidget::new(config.character),
        }
    }
}

pub struct ArenaState { }

pub enum Gui {
    Menu(MenuState),
    Arena(ArenaState),
}

impl Gui {
    pub fn menu(&self) -> &MenuState {
        match self {
            Gui::Menu(menu) => menu,
            _ => panic!("Must be a 'Menu'"),
        }
    }

    pub fn arena(&self) -> &ArenaState {
        match self {
            Gui::Arena(arena) => arena,
            _ => panic!("Must be an 'Arena'"),
        }
    }
}

pub struct State {
    pub user: User,
    pub server: Server,
    pub gui: Gui,
}

impl State {
    pub fn new(config: Config) -> State {
        State {
            user: User {
                character: config.character,
                login_status: None,
            },
            server: Server {
                addr: config.server_addr,
                connection_status: ConnectionStatus::NotConnected,
                udp_port: None,
                udp_confirmed: None,
                version_info: None,
                game: Game {
                    static_info: None,
                    logged_players: Vec::new(),
                },
            },
            gui: Gui::Menu(MenuState::new(&config)),
        }
    }
}
