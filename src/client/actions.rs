use crate::version::{self, Compatibility};

use super::events::{AppEvent, ClosingReason, ServerEvent, ServerInfo, LoginStatus};
use super::util::store::{Actionable, Mutator};
use super::state::{State};

use message_io::events::{EventSender, Senderable};

use std::time::{Duration};

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

pub struct ActionManager {
    event_sender: EventSender<AppEvent>,
}

impl ActionManager {
    pub fn new(event_sender: EventSender<AppEvent>) -> ActionManager {
        ActionManager {
            event_sender: event_sender,
        }
    }

    fn server_call(&mut self, api_call: ApiCall) {
        self.event_sender.send_with_priority(AppEvent::Server(ServerEvent::Api(api_call)));
    }

    fn close(&mut self, reason: ClosingReason) {
        self.event_sender.send_with_priority(AppEvent::Close(reason))
    }
}

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
}

impl Actionable for ActionManager {
    type State = State;
    type Action = Action;

    fn dispatch(&mut self, state: &State, mutator: &mut Mutator<State>, action: Action) {
        log::trace!("Dispatch: {:?}", action);
        match action {

            Action::Connected => {
                mutator.mutate(|state| state.server_mut().set_connected(true));
                self.server_call(ApiCall::CheckVersion(version::current().into()));
            },

            Action::Disconnected => {
                mutator.mutate(|state| state.server_mut().set_connected(false));
                self.close(ClosingReason::ConnectionLost);
            },

            Action::CheckedVersion(server_version, compatibility) => {
                mutator.mutate(|state| {
                    state.server_mut().set_version_info(server_version, compatibility);
                });

                if compatibility.is_compatible() {
                    self.server_call(ApiCall::SubscribeInfo);
                }
                else {
                    self.close(ClosingReason::IncompatibleVersions);
                }
            },

            Action::ServerInfo(info) => {
                self.dispatch(state, mutator, Action::Login);
            },

            Action::PlayerListUpdated(player_names) => {
                //TODO
            },

            Action::Login => {
                let player_name = state
                    .player_name()
                    .expect("The player name must be already defined")
                    .into();

                self.server_call(ApiCall::Login(player_name));
            },

            Action::UpdatePlayerName(player_name) => {
                mutator.mutate(|state| state.set_player_name(player_name));
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
        }
    }
}
