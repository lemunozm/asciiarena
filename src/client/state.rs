use crate::version::{Compatibility};
use crate::message::{LoginStatus};

use std::net::{SocketAddr};

pub struct Config {
    pub server_addr: Option<SocketAddr>,
    pub player_name: Option<String>,
}

#[derive(Clone, Copy)]
pub enum ConnectionStatus {
    Connected,
    NotConnected,
    NotFound,
    Lost,
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
    pub logged_players: Vec<String>,
    pub login_status: Option<LoginStatus>,
}

pub mod gui {
    use crate::client::input_widgets::{InputTextWidget, InputCapitalLetterWidget};

    pub struct Menu {
        pub server_addr_input: InputTextWidget,
        pub player_name_input: InputCapitalLetterWidget,
    }

    impl Menu {
        pub fn new(config: &super::Config) -> Menu {
            Menu {
                server_addr_input: InputTextWidget::new(
                    config.server_addr.map(|addr| addr.to_string())
                ),
                player_name_input: InputCapitalLetterWidget::new(
                    match &config.player_name {
                        Some(name) => name.chars().next(),
                        None => None
                    },
                )
            }
        }
    }

    pub struct Game { }
}

pub enum Gui {
    Menu(gui::Menu),
    Game(gui::Game),
}

impl Gui {
    pub fn menu(&self) -> &gui::Menu {
        match self {
            Gui::Menu(menu) => menu,
            _ => panic!("Must be a 'Menu'"),
        }
    }

    pub fn menu_mut(&mut self) -> &mut gui::Menu {
        match self {
            Gui::Menu(menu) => menu,
            _ => panic!("Must be a 'Menu'"),
        }
    }

    pub fn game(&self) -> &gui::Game {
        match self {
            Gui::Game(game) => game,
            _ => panic!("Must be a 'Game'"),
        }
    }

    pub fn game_mut(&mut self) -> &mut gui::Game {
        match self {
            Gui::Game(game) => game,
            _ => panic!("Must be a 'Game'"),
        }
    }
}

pub struct User {
    pub player_name: Option<String>,
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
                player_name: config.player_name.clone(),
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
                    login_status: None,
                },
            },
            gui: Gui::Menu(gui::Menu::new(&config)),
        }
    }
}
