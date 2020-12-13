use crate::version::{Compatibility};
use crate::character::{CharacterId, Character};
use crate::vec2::{Vec2};
use crate::direction::{Direction};
use crate::ids::{SessionToken, EntityId, SpellId, SpellSpecId, SkillId};

use serde::{Serialize, Deserialize};

use std::time::{Duration};

// See the protocol diagram in docs/design/communication.md

// ===================================================
//     High level messages
// ===================================================

/// Messages that Client sends to Server
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    // Version
    Version(String),

    // Server info
    SubscribeServerInfo,

    // Login messages
    Login(char),
    Logout,

    // Udp handshake
    ConnectUdp(SessionToken),
    TrustUdp,

    // Arena real time messages
    MovePlayer(Direction),
    CastSkill(Direction, SkillId),
}

/// Messages that Server sends to Client
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    // Version
    Version(String, Compatibility),

    // Server info
    StaticServerInfo(ServerInfo),
    DynamicServerInfo(Vec<char>), //player list

    // Login messages
    LoginStatus(char, LoginStatus), //player, status

    // Udp handshake
    UdpConnected,

    // Game messages
    StartGame(GameInfo),
    FinishGame,
    GameEvent(GameEvent),
    GameStep(Frame),

    // Arena messages
    WaitArena(Duration),
    StartArena(ArenaInfo),
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
    pub logged_players: Vec<char>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameInfo {
    pub characters: Vec<Character>,
    pub players: Vec<(CharacterId, usize)>, //id, points
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Terrain {
    Floor,
    Wall,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArenaInfo {
    pub number: usize,
    pub players: Vec<EntityId>, //id
    pub ground: Vec<Terrain>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameEvent {
    PlayerPointsUpdated(Vec<usize>)
    // Other possible game event here
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntityData {
    pub id: EntityId,
    pub character_id: CharacterId,
    pub position: Vec2,
    pub health: usize,
    pub energy: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpellData {
    pub id: SpellId,
    pub spec_id: SpellSpecId,
    pub position: Vec2,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Frame {
    pub entities: Vec<EntityData>,
    pub spells: Vec<SpellData>,
}
