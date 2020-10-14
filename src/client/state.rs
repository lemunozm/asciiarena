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
                udp_port: None,
                udp_confirmed: false,
                version_info: None,
                game: Game {
                    static_info: None,
                    logged_players: Vec::new(),
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
    udp_port: Option<u16>,
    udp_confirmed: bool,
    version_info: Option<VersionInfo>,
    game: Game,
}

impl Server {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn set_connection_status(&mut self, status: ConnectionStatus) {
        self.connection_status = status;
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        self.connection_status
    }

    pub fn set_udp_port(&mut self, port: u16) {
        self.udp_port = Some(port)
    }

    pub fn udp_port(&mut self, port: u16) -> Option<u16> {
        self.udp_port
    }

    pub fn confirm_udp_connection(&mut self) {
        self.udp_confirmed = true;
    }

    pub fn is_udp_confirmed(&self) -> bool {
        self.udp_confirmed
    }

    pub fn set_version_info(&mut self, version: String, compatibility: Compatibility) {
        self.version_info = Some(VersionInfo { version, compatibility });
    }

    pub fn reset_version_info(&mut self) {
        self.version_info = None
    }

    pub fn version_info(&self) -> Option<&VersionInfo> {
        self.version_info.as_ref()
    }

    pub fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }

    pub fn game(&self) -> &Game {
        &self.game
    }
}

pub struct StaticGameInfo {
    pub players_number: usize,
    pub map_size: usize,
    pub winner_points: usize,
}

pub struct Game {
    static_info: Option<StaticGameInfo>,
    logged_players: Vec<String>,
}

impl Game {
    pub fn set_static_info(&mut self, static_info: StaticGameInfo) {
        self.static_info = Some(static_info);
    }

    pub fn static_info(&self) -> Option<&StaticGameInfo> {
        self.static_info.as_ref()
    }

    pub fn set_logged_players(&mut self, logged_players: Vec<String>) {
        self.logged_players = logged_players;
    }

    pub fn logged_players(&self) -> impl Iterator<Item = &String> {
        self.logged_players.iter()
    }
}
