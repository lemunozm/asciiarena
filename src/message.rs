use crate::util::Direction;

pub enum UplinkMessage {
    Version { version: String },
    Login { user_name: String, skill_ids: Vec<usize> },
    Reconnect { user_name: String },
    PlayerMove { direction: Direction },
    PlayerCast { skill_id: usize },
}

pub enum DownlinkMessage {
    Version { version: String, compatible: bool },
    ServerStatus { },
    LoginError { },
    FrameInfo { },
    StartArena { },
    EndArena { },
    StartGame { },
    EndGame { },
}
