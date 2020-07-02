use crate::util::Direction;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Message {
    //Uplink
    Version { value: String },
    Login { user_name: String, skill_ids: Vec<usize> },
    Reconnect { user_name: String },
    Disconnect,
    PlayerMove { direction: Direction },
    PlayerCast { skill_id: usize },

    //Downlink
    VersionInfo { value: String, compatible: bool },
    ServerStatus { },
    LoginError { },
    FrameInfo { },
    StartArena { },
    EndArena { },
    StartGame { },
    EndGame { },
}

