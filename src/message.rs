use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientMessage {
    Version { tag: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMessage {
    Version { tag: String, compatible: bool },
}
