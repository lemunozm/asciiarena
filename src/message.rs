use crate::version::{Compatibility};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Version(String),
    RequestServerInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Version(String, Compatibility),
    ServerInfo(ServerInfo),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInfo {
    pub udp_port: u16,
    pub players: u8,
    pub map_dimension: (u16, u16),
    pub winner_points: u16,
    pub logged_players: Vec<String>,
}
