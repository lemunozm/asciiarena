use super::util::store::{Actionable};
use super::state::{State, StaticGameInfo, VersionInfo, GameStatus};
use super::server_proxy::{ServerApi, ApiCall, ConnectionStatus, ServerEvent};

use crate::version::{self};

use std::net::{SocketAddr};

pub trait AppController {
    fn close(&mut self);
}

/// Action API
#[derive(Debug)]
pub enum Action {
    StartApp,
    Connect(SocketAddr),
    Disconnect,
    Login(char),
    Logout,
    CloseGame,
    Close,
    ServerEvent(ServerEvent),
}

pub struct ActionManager {
    app: Box<dyn AppController>,
    server: ServerApi,
}

impl ActionManager {
    pub fn new(app: impl AppController + 'static, server: ServerApi) -> ActionManager {
        ActionManager {
            app: Box::new(app),
            server
        }
    }
}

impl Actionable for ActionManager {
    type State = State;
    type Action = Action;

    fn dispatch(&mut self, state: &mut State, action: Action) {
        log::trace!("Dispatch: {:?}", action);
        match action {
            Action::StartApp => {
                if let Some(addr) = state.server.addr {
                    self.server.call(ApiCall::Connect(addr));
                }
            },

            Action::Connect(addr) => {
                state.server.addr = Some(addr);
                self.server.call(ApiCall::Connect(addr));
            },

            Action::Disconnect => {
                state.server.addr = None;
                self.server.call(ApiCall::Disconnect);
            }

            Action::Login(character) => {
                state.user.character = Some(character);
                self.server.call(ApiCall::Login(character));
            },

            Action::Logout => {
                state.user.character = None;
                state.user.login_status = None;
                self.server.call(ApiCall::Logout);
            }

            Action::CloseGame => {
                state.server.game.status = GameStatus::NotStarted;
            }

            Action::Close => {
                self.app.close();
            }

            Action::ServerEvent(server_event) => match server_event {
                ServerEvent::ConnectionResult(status)  => {
                    state.server.connection_status = status;
                    if let ConnectionStatus::Connected = status {
                        self.server.call(ApiCall::CheckVersion(version::current().into()));
                    }
                    else {
                        state.user.login_status = None;
                    }
                },

                ServerEvent::CheckedVersion(server_version, compatibility) => {
                    let version_info = VersionInfo { version: server_version, compatibility };
                    state.server.version_info = Some(version_info);

                    if compatibility.is_compatible() {
                        self.server.call(ApiCall::SubscribeInfo);
                    }
                },

                ServerEvent::ServerInfo(info) => {
                    let game_info = StaticGameInfo {
                        players_number: info.players_number as usize,
                        map_size: info.map_size as usize,
                        winner_points: info.winner_points as usize,
                    };
                    state.server.udp_port = Some(info.udp_port);
                    state.server.game_info = Some(game_info);
                    state.server.logged_players = info.logged_players;

                    if let Some(character) = state.user.character {
                        self.server.call(ApiCall::Login(character));
                    }
                },

                ServerEvent::PlayerListUpdated(player_names) => {
                    state.server.logged_players = player_names;
                },

                ServerEvent::LoginStatus(status) => {
                    state.user.login_status = Some(status);
                },

                ServerEvent::UdpReachable(value) => {
                    state.server.udp_confirmed = Some(value);
                },

                ServerEvent::StartGame => {
                    state.server.game.status = GameStatus::Started;
                },

                ServerEvent::FinishGame => {
                    state.server.game.status = GameStatus::Finished;
                    state.server.logged_players = Vec::new();
                    state.server.udp_confirmed = None;
                    state.user.character = None;
                    state.user.login_status = None;
                },

                ServerEvent::PrepareArena(_duration) => {
                    //TODO
                },

                ServerEvent::StartArena => {
                },

                ServerEvent::FinishArena => {
                    //TODO
                },

                ServerEvent::ArenaStep => {
                    //TODO
                },
            },
        }
    }
}
