use crate::version::{Compatibility};

use std::net::{SocketAddr};

pub struct State {
    player_name: Option<String>,
    server: Server,
}

impl State {
    pub fn new(addr: SocketAddr, player_name: Option<&str>) -> State {
        State {
            player_name: player_name.map(|name| name.into()),
            server: Server {
                addr,
                connection_status: ConnectionStatus::NotConnected,
                version_info: None,
                game: Game {
                    static_info: None,
                    dynamic_info: None,
                },
            },
        }
    }

    pub fn set_player_name(&mut self, player_name: Option<String>) {
        self.player_name = player_name;
    }

    pub fn player_name(&self) -> Option<&str> {
        self.player_name.as_ref().map(|name| name.as_ref())
    }

    pub fn server(&self) -> &Server {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Server {
        &mut self.server
    }
}

#[derive(Clone, Copy, PartialEq)]
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
    addr: SocketAddr,
    connection_status: ConnectionStatus,
    version_info: Option<VersionInfo>,
    game: Game,
}

impl Server {
    pub fn set_connection_status(&mut self, status: ConnectionStatus) {
        self.connection_status = status;
    }

    pub fn set_version_info(&mut self, version: String, compatibility: Compatibility) {
        self.version_info = Some(VersionInfo { version, compatibility });
    }

    pub fn reset_version_info(&mut self) {
        self.version_info = None
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        self.connection_status
    }

    pub fn version_info(&self) -> Option<&VersionInfo> {
        self.version_info.as_ref()
    }

    pub fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }
}

pub struct StaticGameInfo {
    players_number: usize,
    map_size: usize,
    winner_points: usize,
}

pub struct DynamicGameInfo {
    logged_players: Vec<String>,
}

pub struct Game {
    static_info: Option<StaticGameInfo>,
    dynamic_info: Option<DynamicGameInfo>,
}

impl Game {
    pub fn set_static_game_info(&mut self, players_number: usize, map_size: usize, winner_points: usize) {
        self.static_info = Some(StaticGameInfo { players_number, map_size, winner_points });
    }

    pub fn set_dynamic_game_info(&mut self, logged_players: Vec<String>) {
        self.dynamic_info = Some(DynamicGameInfo { logged_players });
    }
}
