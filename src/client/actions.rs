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
    Disconnect,
    CheckVersion(String),
    SubscribeInfo,
    Login(char),
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
    Connect,
    Disconnect,
    ConnectionResult(ConnectionStatus),
    CheckedVersion(String, Compatibility),
    ServerInfo(ServerInfo),
    PlayerListUpdated(Vec<char>),
    Login,
    Logout,
    LoginStatus(LoginStatus),
    UdpReachable(bool),
    StartGame,
    FinishGame,
    PrepareArena(Duration),
    StartArena,
    FinishArena,
    ArenaStep,
    ResizeWindow(usize, usize),
    KeyPressed(KeyEvent),
    InputServerAddrFocus,
    InputPlayerNameFocus,
    Close,
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
                self.dispatch(state, Action::Connect);
            },

            Action::Connect => {
                match state.server.addr {
                    Some(addr) => self.server.call(ApiCall::Connect(addr)),
                    None => self.dispatch(state, Action::InputServerAddrFocus),
                }
            },

            Action::Disconnect => {
                self.server.call(ApiCall::Disconnect);
            }

            Action::ConnectionResult(status)  => {
                state.server.connection_status = status;
                if let ConnectionStatus::Connected = status {
                    self.server.call(ApiCall::CheckVersion(version::current().into()));
                }
                else {
                    state.user.login_status = None;
                    self.dispatch(state, Action::InputServerAddrFocus);
                }
            },

            Action::CheckedVersion(server_version, compatibility) => {
                let version_info = VersionInfo { version: server_version, compatibility };
                state.server.version_info = Some(version_info);

                if compatibility.is_compatible() {
                    self.server.call(ApiCall::SubscribeInfo);
                }
                else {
                    self.dispatch(state, Action::InputServerAddrFocus);
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

                self.dispatch(state, Action::Login);
            },

            Action::PlayerListUpdated(player_names) => {
                state.server.game.logged_players = player_names;
            },

            Action::Login => {
                match state.user.character {
                    Some(character) => self.server.call(ApiCall::Login(character)),
                    None => self.dispatch(state, Action::InputPlayerNameFocus),
                }
            },

            Action::Logout => {
                self.server.call(ApiCall::Logout);
                state.user.login_status = None;
                self.dispatch(state, Action::InputPlayerNameFocus);
            }

            Action::LoginStatus(status) => {
                state.user.login_status = Some(status);
                if !status.is_logged() {
                    self.dispatch(state, Action::InputPlayerNameFocus);
                }
            },

            Action::UdpReachable(value) => {
                state.server.udp_confirmed = Some(value);
            },

            Action::StartGame => {
                //TODO
            },

            Action::FinishGame => {
                state.server.game.logged_players = Vec::new();
                state.user.login_status = None;
                state.server.udp_confirmed = None;
                self.dispatch(state, Action::InputPlayerNameFocus);
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
                                    let content = menu.server_addr_input.content();
                                    match content.parse::<SocketAddr>() {
                                        Ok(addr) => {
                                            state.server.addr = Some(addr);
                                            menu.server_addr_input.focus(false);
                                            self.dispatch(state, Action::Connect);
                                        },
                                        Err(_) => state.server.addr = None,
                                    }
                                }
                                else if menu.player_name_input.has_focus() {
                                    match menu.player_name_input.content() {
                                        Some(character) => {
                                            state.user.character = Some(character);
                                            menu.player_name_input.focus(false);
                                            self.dispatch(state, Action::Login);
                                        }
                                        None => state.user.character = None,
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                if let Some(LoginStatus::Logged(..)) = state.user.login_status {
                                    if !state.server.game.is_full() {
                                        self.dispatch(state, Action::Logout);
                                    }
                                }
                                else if let ConnectionStatus::Connected = state.server.connection_status {
                                    self.dispatch(state, Action::Disconnect);
                                }
                            },
                            _ => (),
                        }
                    },
                    Gui::Arena(ref mut game) => {
                        //TODO
                    }
                }
            },
            Action::InputServerAddrFocus => {
                if let Gui::Menu(ref mut menu) = state.gui {
                    menu.server_addr_input.focus(true);
                    menu.player_name_input.focus(false);
                }
            },
            Action::InputPlayerNameFocus => {
                if let Gui::Menu(ref mut menu) = state.gui {
                    menu.server_addr_input.focus(false);
                    menu.player_name_input.focus(true);
                }
            },
            Action::Close => {
                self.app.close();
            },
        }
    }
}
