use crate::version::{Compatibility};
use crate::message::{LoginStatus};

use std::net::{SocketAddr};

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
    pub addr: SocketAddr,
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
    use crossterm::event::KeyCode;

    pub enum SelectedPanel {
       ServerAddress(usize), //cursor
       PlayerName,
       None,
    }

    pub struct Menu {
        pub selected_panel: SelectedPanel,
    }

    pub struct Game {

    }

    pub enum Gui {
        Menu(Menu),
        Game(Game),
    }
}

pub struct State {
    pub player_name: Option<String>,
    pub server: Server,
    pub gui: gui::Gui,
}

impl State {
    pub fn new(addr: SocketAddr, player_name: Option<&str>) -> State {
        State {
            player_name: player_name.map(|name| name.into()),
            server: Server {
                addr,
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
            gui: gui::Gui::Menu(gui::Menu {
                selected_panel: gui::SelectedPanel::None,
            }),
        }
    }
}
