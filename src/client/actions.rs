use super::util::store::{Actionable, StateManager};
use super::state::{State, ConnectionStatus, VersionInfo, StaticGameInfo};

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

pub trait AppController: Send {
    fn close(&mut self);
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
    ResizeWindow(usize, usize),
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
    app: Box<dyn AppController>,
    server: Box<dyn ServerApi>,
}

impl ActionManager {
    pub fn new(app: impl AppController + 'static, server: impl ServerApi + 'static) -> ActionManager {
        ActionManager {
            app: Box::new(app),
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
                        state.mutate(|state| {
                            state.server_mut().set_connection_status(ConnectionStatus::Connected)
                        });
                        self.server.call(ApiCall::CheckVersion(version::current().into()));
                    },
                    ConnectionResult::NotFound => {
                        state.mutate(|state| {
                            state.server_mut().set_connection_status(ConnectionStatus::NotFound)
                        });
                    },
                }
            },

            Action::Disconnected => {
                state.mutate(|state| {
                    state.server_mut().set_connection_status(ConnectionStatus::Lost);
                });
            },

            Action::CheckedVersion(server_version, compatibility) => {
                state.mutate(|state| {
                    state.server_mut().set_version_info(server_version, compatibility);
                });

                if compatibility.is_compatible() {
                    self.server.call(ApiCall::SubscribeInfo);
                }
            },

            Action::ServerInfo(info) => {
                state.mutate(|state| {
                    let static_info = StaticGameInfo {
                        players_number: info.players_number as usize,
                        map_size: info.map_size as usize,
                        winner_points: info.winner_points as usize,
                    };
                    state.server_mut().set_udp_port(info.udp_port);
                    state.server_mut().game_mut().set_static_info(static_info);
                    state.server_mut().game_mut().set_dynamic_info(info.logged_players);
                });
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
                };
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
                //TODO
            },
            Action::ResizeWindow(_, _) => {},
            Action::Close => {
                self.app.close();
            },
        }
    }
}
