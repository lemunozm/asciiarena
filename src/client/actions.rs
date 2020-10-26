use super::util::store::{Actionable};
use super::state::{State, ConnectionStatus, StaticGameInfo, VersionInfo, Gui};

use crate::message::{ServerInfo, LoginStatus};
use crate::version::{self, Compatibility};

use crossterm::event::{KeyEvent, KeyCode};

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
    UdpReachable(bool),
    StartGame,
    FinishGame,
    PrepareArena(Duration),
    StartArena,
    FinishArena,
    ArenaStep,
    ResizeWindow(usize, usize),
    KeyPressed(KeyEvent),
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

    fn dispatch(&mut self, state: &mut State, action: Action) {
        log::trace!("Dispatch: {:?}", action);
        match action {

            Action::StartApp => {
                match state.server.addr {
                    Some(addr) => self.server.call(ApiCall::Connect(addr)),
                    None => {
                        if let Gui::Menu(menu) = &mut state.gui {
                            menu.server_addr_input.focus(true);
                        }
                    }
                }
            },

            Action::ConnectionResult(result)  => {
                match result {
                    ConnectionResult::Connected => {
                        state.server.connection_status = ConnectionStatus::Connected;
                        self.server.call(ApiCall::CheckVersion(version::current().into()));
                    },
                    ConnectionResult::NotFound => {
                        state.server.connection_status = ConnectionStatus::NotFound;
                    },
                }
            },

            Action::Disconnected => {
                state.server.connection_status = ConnectionStatus::Lost;
            },

            Action::CheckedVersion(server_version, compatibility) => {
                let version_info = VersionInfo { version: server_version, compatibility };
                state.server.version_info = Some(version_info);

                if compatibility.is_compatible() {
                    self.server.call(ApiCall::SubscribeInfo);
                }
            },

            Action::ServerInfo(info) => {
                let static_info = StaticGameInfo {
                    players_number: info.players_number as usize,
                    map_size: info.map_size as usize,
                    winner_points: info.winner_points as usize,
                };
                state.server.udp_port = Some(info.udp_port);
                state.server.game.static_info = Some(static_info);
                state.server.game.logged_players = info.logged_players;

                match &state.user.player_name {
                    Some(_) => self.dispatch(state, Action::Login),
                    None => {
                        if let Gui::Menu(menu) = &mut state.gui {
                            menu.player_name_input.focus(true);
                        }
                    }
                }
            },

            Action::PlayerListUpdated(player_names) => {
                state.server.game.logged_players = player_names;
            },

            Action::Login => {
                let player_name = state.user.player_name.as_ref()
                    .expect("The player name must be already defined")
                    .into();

                self.server.call(ApiCall::Login(player_name));
            },

            Action::UpdatePlayerName(player_name) => {
                state.user.player_name = player_name;
            },

            Action::LoginStatus(_player_name, status) => {
                state.server.game.login_status = Some(status);
            },

            Action::UdpReachable(value) => {
                state.server.udp_confirmed = Some(value);
            },

            Action::StartGame => {
                //TODO
            },

            Action::FinishGame => {
                state.server.game.logged_players = Vec::new();
                state.server.game.login_status = None;
                state.server.udp_confirmed = None;
            },

            Action::PrepareArena(_duration) => {
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
            Action::KeyPressed(key_event) => {
                match state.gui {
                    Gui::Menu(ref mut menu) => {
                        menu.server_addr_input.key_pressed(key_event);
                        menu.player_name_input.key_pressed(key_event);
                        match key_event.code {
                            KeyCode::Enter => {
                                if menu.server_addr_input.has_focus() {
                                    //TODO: dispatch new action? (connect)
                                    menu.server_addr_input.focus(false)
                                }
                                if menu.player_name_input.has_focus() {
                                    //TODO: dispatch new action? (login)
                                    menu.server_addr_input.focus(false)
                                }
                            }
                            _ => (),
                        }
                    },
                    Gui::Game(ref _game) => {
                        //TODO
                    }
                }
            },
            Action::Close => {
                self.app.close();
            },
        }
    }
}
