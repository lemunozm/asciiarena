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

pub struct StaticGameInfo {
    pub players_number: usize,
    pub map_size: usize,
    pub winner_points: usize,
}

pub enum GameStatus {
    NotStarted,
    Started,
    Finished,
}

pub struct Game {
    pub status: GameStatus,
}

pub struct Server {
    pub addr: Option<SocketAddr>,
    pub connection_status: ConnectionStatus,
    pub udp_port: Option<u16>,
    pub udp_confirmed: Option<bool>,
    pub version_info: Option<VersionInfo>,
    pub game_info: Option<StaticGameInfo>,
    pub logged_players: Vec<char>,
    pub game: Game,
}

impl Server {
    pub fn is_full(&self) -> bool {
        if let Some(StaticGameInfo {players_number, .. }) = self.game_info {
            if players_number == self.logged_players.len() {
                return true
            }
        }
        false
    }
}

pub struct MenuGuiState {
    pub server_addr_input: InputTextWidget,
    pub character_input: InputCapitalLetterWidget,
}

impl MenuGuiState {
    pub fn new(config: &super::Config) -> MenuGuiState {
        MenuGuiState {
            server_addr_input: InputTextWidget::new(
                config.server_addr.map(|addr| addr.to_string())
            ),
            character_input: InputCapitalLetterWidget::new(config.character),
        }
    }
}

pub struct ArenaGuiState { }

impl ArenaGuiState {
    pub fn new(config: &super::Config) -> ArenaGuiState {
        ArenaGuiState { }
    }
}

pub enum GuiSelector {
    Menu,
    Arena,
}

pub struct Gui {
    pub menu: MenuGuiState,
    pub arena: ArenaGuiState,
    pub selector: GuiSelector,
}

pub struct State {
    pub user: User,
    pub server: Server,
    pub gui: Gui,
    config: Config,
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
                game_info: None,
                logged_players: Vec::new(),
                game: Game {
                    status: GameStatus::NotStarted,
                },
            },
            gui: Gui {
                menu: MenuGuiState::new(&config),
                arena: ArenaGuiState::new(&config),
                selector: GuiSelector::Menu,
            },
            config,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
