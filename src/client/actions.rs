use super::util::store::{Actionable, StateManager};
use super::state::{State};

use crate::message::{ServerInfo, LoginStatus};
use crate::version::{self, Compatibility};

use std::time::{Duration};
use std::net::{SocketAddr};

/// Event API to control the connection
#[derive(Debug)]
pub enum ApiCall {
    Connect(SocketAddr),
    CheckVersion(String),
    SubscribeInfo,
    Login(String),
    Logout,
    MovePlayer,
    CastSkill,
}

pub trait ServerApi {
    fn call(&mut self, api_call: ApiCall);
}


/// Event API to close the application
#[derive(Debug)]
pub enum ClosingReason {
    ServerNotFound(SocketAddr),
    Forced, //Ctrl-c
    ConnectionLost,
    IncompatibleVersions,
}

pub trait Closer: Send {
    fn close(&mut self, reason: ClosingReason);
}

/// Action API
#[derive(Debug)]
pub enum Action {
    StartApp,
    ConnectionResult(ConnectionResult),
    Disconnected,
    CheckedVersion(String, Compatibility),
    ServerInfo(ServerInfo),
    PlayerListUpdated(Vec<String>),
    Login,
    UpdatePlayerName(Option<String>),
    LoginStatus(String, LoginStatus),
    UdpReachable,
    StartGame,
    FinishGame,
    PrepareArena(Duration),
    StartArena,
    FinishArena,
    ArenaStep,
    Close,
}

/// Action API utils
#[derive(Debug)]
pub enum ConnectionResult {
    Connected,
    NotFound,
}

pub trait Dispatcher: Send + Sync {
    fn dispatch(&mut self, action: Action);
}


pub struct ActionManager {
    closer: Box<dyn Closer>,
    server: Box<dyn ServerApi>,
}

impl ActionManager {
    pub fn new(closer: impl Closer + 'static, server: impl ServerApi + 'static) -> ActionManager {
        ActionManager {
            closer: Box::new(closer),
            server: Box::new(server),
        }
    }
}

impl Actionable for ActionManager {
    type State = State;
    type Action = Action;

    fn dispatch(&mut self, state: &mut StateManager<State>, action: Action) {
        log::trace!("Dispatch: {:?}", action);
        match action {

            Action::StartApp => {
                self.server.call(ApiCall::Connect(state.get().server().addr()));
            },

            Action::ConnectionResult(result)  => {
                match result {
                    ConnectionResult::Connected => {
                        state.mutate(|state| state.server_mut().set_connected(true));
                        self.server.call(ApiCall::CheckVersion(version::current().into()));
                    },
                    ConnectionResult::NotFound => (),
                }
            },

            Action::Disconnected => {
                state.mutate(|state| state.server_mut().set_connected(false));
                self.closer.close(ClosingReason::ConnectionLost);
            },

            Action::CheckedVersion(server_version, compatibility) => {
                state.mutate(|state| {
                    state.server_mut().set_version_info(server_version, compatibility);
                });

                if compatibility.is_compatible() {
                    self.server.call(ApiCall::SubscribeInfo);
                }
                else {
                    self.closer.close(ClosingReason::IncompatibleVersions);
                }
            },

            Action::ServerInfo(info) => {
                self.dispatch(state, Action::Login);
            },

            Action::PlayerListUpdated(player_names) => {
                //TODO
            },

            Action::Login => {
                let player_name = state.get()
                    .player_name()
                    .expect("The player name must be already defined")
                    .into();

                self.server.call(ApiCall::Login(player_name));
            },

            Action::UpdatePlayerName(player_name) => {
                state.mutate(|state| state.set_player_name(player_name));
            },

            Action::LoginStatus(player_name, status) => {
                match status {
                    LoginStatus::Logged(_token, _kind) => {
                    },
                    LoginStatus::InvalidPlayerName => {
                    },
                    LoginStatus::AlreadyLogged => {
                    },
                    LoginStatus::PlayerLimit => {
                    },
                }
            },

            Action::UdpReachable => {
                //TODO
            },

            Action::StartGame => {
                //TODO
            },

            Action::FinishGame => {
                //TODO
            },

            Action::PrepareArena(duration) => {
                //TODO
            },

            Action::StartArena => {
                //TODO
            },

            Action::FinishArena => {
                //TODO
            },

            Action::ArenaStep => {
                println!("step");
                //TODO
            },
            Action::Close => {
                self.closer.close(ClosingReason::Forced)
            },
        }
    }
}
