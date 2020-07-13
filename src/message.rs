use crate::version::{Compatibility};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Version(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Version(String, Compatibility),
}
