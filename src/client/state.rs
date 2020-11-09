use crate::version::{Compatibility};
use crate::message::{LoginStatus};

use super::server_proxy::{ConnectionStatus};
use super::configuration::{Config};

use std::net::{SocketAddr};
use std::time::{Duration};

pub struct User {
    pub character: Option<char>,
    pub login_status: Option<LoginStatus>,
}

impl User {
    pub fn is_logged(&self) -> bool {
        if let Some(LoginStatus::Logged(..)) = self.login_status {
            return true
        }
        false
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArenaStatus {
    Playing,
    Finished,
}

pub struct Arena {
    pub number: usize,
    pub status: ArenaStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameStatus {
    NotStarted,
    Started,
    Finished,
}

pub struct Game {
    pub status: GameStatus,
    pub waiting_arena: Option<Duration>,
    pub arena: Option<Arena>,
}

impl Game {
    pub fn arena(&self) -> &Arena {
        self.arena.as_ref().unwrap()
    }
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

    pub fn is_connected(&self) -> bool {
        match self.connection_status {
            ConnectionStatus::Connected => true,
            _ => false,
        }
    }

    pub fn has_compatible_version(&self) -> bool {
        if let Some(version_info) = &self.version_info {
            return version_info.compatibility.is_compatible()
        }
        false
    }
}

pub struct State {
    pub user: User,
    pub server: Server,
}

impl State {
    pub fn new(config: &Config) -> State {
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
                    waiting_arena: None,
                    arena: None,
                },
            },
        }
    }
}
