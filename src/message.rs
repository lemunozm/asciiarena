use crate::version::{Compatibility};
use crate::character::{CharacterId, Character};
use crate::vec2::{Vec2};
use crate::direction::{Direction};

use serde::{Serialize, Deserialize};

use std::time::{Duration};

// See the protocol diagram in docs/design/communication.md

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
    Login(char),
    Logout,

    // Udp handshake
    ConnectUdp(SessionToken),
    TrustUdp,

    // Arena real time messages
    MovePlayer(Direction), //direction
    CastSkill, //id
}

/// Messages that Server sends to Client
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    // Messages out of login
    Version(String, Compatibility),

    // Server info
    ServerInfo(ServerInfo),
    DynamicServerInfo(Vec<char>), //player list

    // Login messages
    LoginStatus(char, LoginStatus), //player, status

    // Udp handshake
    UdpConnected,

    // Game level messages
    StartGame(GameInfo),
    FinishGame, //points

    // Arena level messages
    WaitArena(Duration),
    StartArena(ArenaInfo),
    ArenaChange(ArenaChange),
    Step(Frame), //arena state
    FinishArena, // winners
}

// ===================================================
//     Composable message pieces
// ===================================================
pub type SessionToken = usize;
pub type EntityId = usize;

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
    pub logged_players: Vec<char>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntityData {
    pub id: EntityId,
    pub character_id: CharacterId,
    pub position: Vec2,
    pub live: usize,
    pub energy: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameInfo {
    pub characters: Vec<Character>,
    pub players: Vec<(CharacterId, usize)>, //id, total_pointsd
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArenaInfo {
    pub number: usize,
    pub players: Vec<(Option<EntityId>, usize)>, //id, partial_points
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ArenaChange {
    PlayerPartialPoints(Vec<usize>)
    // Other possible arena changes here
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Frame {
    pub entities: Vec<EntityData>,
}
