use crate::version::{Compatibility};
use crate::util::{SessionToken};

use serde::{Serialize, Deserialize};

// ===================================================
//     High level messages
// ===================================================

/// Messages that Client sends to Server
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Version(String),
    Login(String),
    RequestServerInfo,
    //UdpHello(SessionToken),
}

/// Messages that Server sends to Client
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Version(String, Compatibility),
    ServerInfo(ServerInfo),
    LoginStatus(LoginStatus),
    NotifyNewPlayer(String),
    //UdpHello(SessionToken),
}

// ===================================================
//     Composable message pieces
// ===================================================
#[derive(Serialize, Deserialize, Debug)]
pub enum LoginStatus {
    Logged(SessionToken),
    Relogged(SessionToken),
    InvalidPlayerName,
    AlreadyLogged,
    PlayerLimit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInfo {
    pub udp_port: u16,
    pub players: u8,
    pub map_size: u16,
    pub winner_points: u16,
    pub logged_players: Vec<String>,
}
