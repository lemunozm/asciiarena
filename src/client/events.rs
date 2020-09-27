pub use super::connection::{ServerEvent, ServerInfo, LoginStatus};

use std::net::{SocketAddr};

#[derive(Debug)]
pub enum ClosingReason {
    ServerNotFound(SocketAddr),
    //Forced, //Ctrl-c
    ConnectionLost,
    IncompatibleVersions,
}

#[derive(Debug)]
pub enum AppEvent {
    Server(ServerEvent),
    //FrontEnd(F::FrontendEvent),
    Close(ClosingReason),
}

