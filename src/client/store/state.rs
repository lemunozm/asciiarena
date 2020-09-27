use crate::version::{Compatibility};

use std::net::{SocketAddr};


pub struct State {
    addr: SocketAddr,
    player_name: Option<String>,
    server: Server,
}

impl State {
    pub fn new(addr: SocketAddr) -> State {
        State {
            addr,
            player_name: None,
            server: Server {
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

    pub fn server_mut(&mut self) -> &mut Server {
        &mut self.server
    }
}

pub struct VersionInfo {
    server_version: String,
    compatibility: Compatibility,
}

pub struct Server {
    version_info: Option<VersionInfo>,
    game: Game,
}

impl Server {
    pub fn set_version_info(&mut self, server_version: String, compatibility: Compatibility) {
        self.version_info = Some(VersionInfo { server_version, compatibility });
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
