use crate::version::{Compatibility};
use crate::util::{SessionToken};

use serde::{Serialize, Deserialize};

use std::time::{Duration};

// ===================================================
//     High level messages
// ===================================================

/// Messages that Client sends to Server
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Version(String),
    RequestServerInfo,

    Login(String),
    //UdpHello(SessionToken),

    Move, //direction
    Skill, //id
}

/// Messages that Server sends to Client
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Version(String, Compatibility),
    ServerInfo(ServerInfo),

    LoginStatus(LoginStatus),
    PlayerListUpdated(Vec<String>),
    //UdpHello(SessionToken),

    StartGame,
    EndGame, //points

    PrepareArena(Duration),
    StartArena,
    EndArena, //winners

    Step, //arena state
}

// ===================================================
//     Composable message pieces
// ===================================================
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum LoginStatus {
    Logged(SessionToken),
    Reconnected(SessionToken),
    InvalidPlayerName,
    AlreadyLogged,
    PlayerLimit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInfo {
    pub udp_port: u16,
    pub players_number: u8,
    pub map_size: u16,
    pub winner_points: u16,
    pub logged_players: Vec<String>,
}
