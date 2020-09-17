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
    // Messages out of login
    Version(String),
    SubscribeServerInfo,

    // Login messages
    Login(String),
    Logout,

    // Udp handshake
    ConnectUdp(SessionToken),
    TrustUdp,

    // Arena real time messages
    Move, //direction
    Skill, //id
}

/// Messages that Server sends to Client
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    // Messages out of login
    Version(String, Compatibility),

    // Server info
    StaticServerInfo(ServerInfo),
    DynamicServerInfo(Vec<String>), //player list

    // Login messages
    LoginStatus(LoginStatus),

    // Udp handshake
    UdpConnected,

    // Game level messages
    StartGame,
    FinishGame, //points

    // Arena prelude level messages
    PrepareArena(Duration),
    StartArena,
    FinishArena, //winners

    // Arena real time messages
    Step, //arena state
}

// ===================================================
//     Composable message pieces
// ===================================================
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum LoggedKind {
    FirstTime,
    Reconnection,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum LoginStatus {
    Logged(SessionToken, LoggedKind),
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
