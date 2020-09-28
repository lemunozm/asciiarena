use super::util::store::{Actionable, StateManager};
use super::state::{State};

use crate::message::{ServerInfo, LoginStatus};
use crate::version::{self, Compatibility};

use message_io::events::{Senderable};

use std::time::{Duration};
use std::net::{SocketAddr};

/// Event API to control the connection
#[derive(Debug)]
pub enum ApiCall {
    CheckVersion(String),
    SubscribeInfo,
    Login(String),
    Logout,
    MovePlayer,
    CastSkill,
}

/// Event API to close the application
#[derive(Debug)]
pub enum ClosingReason {
    ServerNotFound(SocketAddr),
    Forced, //Ctrl-c
    ConnectionLost,
    IncompatibleVersions,
}

pub enum ActionableEvent {
    Api(ApiCall),
    Close(ClosingReason),
}

pub struct ActionManager {
    event_sender: Box<dyn Senderable<ActionableEvent>>,
}

impl ActionManager {
    pub fn new<S>(event_sender: S) -> ActionManager
    where S: Senderable<ActionableEvent> + Send + 'static + Clone {
        ActionManager {
            event_sender: Box::new(event_sender),
        }
    }

    fn server_call(&mut self, api_call: ApiCall) {
        self.event_sender.send_with_priority(ActionableEvent::Api(api_call));
    }

    fn close_app(&mut self, reason: ClosingReason) {
        self.event_sender.send_with_priority(ActionableEvent::Close(reason))
    }
}

/// Action API
#[derive(Debug)]
pub enum Action {
    Connected,
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

impl Actionable for ActionManager {
    type State = State;
    type Action = Action;

    fn dispatch(&mut self, state: &mut StateManager<State>, action: Action) {
        log::trace!("Dispatch: {:?}", action);
        match action {

            Action::Connected => {
                state.mutate(|state| state.server_mut().set_connected(true));
                self.server_call(ApiCall::CheckVersion(version::current().into()));
            },

            Action::Disconnected => {
                state.mutate(|state| state.server_mut().set_connected(false));
                self.close_app(ClosingReason::ConnectionLost);
            },

            Action::CheckedVersion(server_version, compatibility) => {
                state.mutate(|state| {
                    state.server_mut().set_version_info(server_version, compatibility);
                });

                if compatibility.is_compatible() {
                    self.server_call(ApiCall::SubscribeInfo);
                }
                else {
                    self.close_app(ClosingReason::IncompatibleVersions);
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

                self.server_call(ApiCall::Login(player_name));
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
                self.close_app(ClosingReason::Forced)
            },
        }
    }
}
