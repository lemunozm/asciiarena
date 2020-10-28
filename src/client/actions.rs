use super::util::store::{Actionable};
use super::state::{State, StaticGameInfo, VersionInfo, GameStatus, GuiSelector};
use super::server_proxy::{ServerApi, ApiCall, ConnectionStatus, ServerEvent};

use crate::client::terminal::input::{InputEvent};
use crate::message::{LoginStatus};
use crate::version::{self};

use crossterm::event::{KeyCode, KeyModifiers};

use std::net::{SocketAddr};

pub trait AppController {
    fn close(&mut self);
}

/// Action API
#[derive(Debug)]
pub enum Action {
    StartApp,
    Connect,
    Disconnect,
    Login,
    Logout,
    ServerEvent(ServerEvent),
    InputEvent(InputEvent),
    InputServerAddrFocus,
    InputPlayerNameFocus,
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

            Action::ServerEvent(server_event) => match server_event {
                ServerEvent::ConnectionResult(status)  => {
                    state.server.connection_status = status;
                    if let ConnectionStatus::Connected = status {
                        self.server.call(ApiCall::CheckVersion(version::current().into()));
                    }
                    else {
                        state.user.login_status = None;
                        self.dispatch(state, Action::InputServerAddrFocus);
                    }
                },

                ServerEvent::CheckedVersion(server_version, compatibility) => {
                    let version_info = VersionInfo { version: server_version, compatibility };
                    state.server.version_info = Some(version_info);

                    if compatibility.is_compatible() {
                        self.server.call(ApiCall::SubscribeInfo);
                    }
                    else {
                        self.dispatch(state, Action::InputServerAddrFocus);
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

                    self.dispatch(state, Action::Login);
                },

                ServerEvent::PlayerListUpdated(player_names) => {
                    state.server.logged_players = player_names;
                },

                ServerEvent::LoginStatus(status) => {
                    state.user.login_status = Some(status);
                    if !status.is_logged() {
                        self.dispatch(state, Action::InputPlayerNameFocus);
                    }
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
                    state.user.login_status = None;
                    self.dispatch(state, Action::InputPlayerNameFocus);
                },

                ServerEvent::PrepareArena(_duration) => {
                    //TODO
                },

                ServerEvent::StartArena => {
                    state.gui.selector = GuiSelector::Arena;
                },

                ServerEvent::FinishArena => {
                    //TODO
                },

                ServerEvent::ArenaStep => {
                    //TODO
                },
            },

            Action::InputEvent(input_event) => match input_event {

                InputEvent::KeyPressed(key_event) => {
                    match key_event.code {
                        KeyCode::Char(character) => {
                            if character == 'c' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                                return self.app.close()
                            }
                        },
                        _ => (),
                    }
                    match state.gui.selector {
                        GuiSelector::Menu => {
                            let menu = &mut state.gui.menu;
                            menu.server_addr_input.key_pressed(key_event);
                            menu.character_input.key_pressed(key_event);
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
                                    else if menu.character_input.has_focus() {
                                        match menu.character_input.content() {
                                            Some(character) => {
                                                state.user.character = Some(character);
                                                menu.character_input.focus(false);
                                                self.dispatch(state, Action::Login);
                                            }
                                            None => state.user.character = None,
                                        }
                                    }
                                }
                                KeyCode::Esc => {
                                    if let Some(LoginStatus::Logged(..)) = state.user.login_status {
                                        if !state.server.is_full() {
                                            self.dispatch(state, Action::Logout);
                                        }
                                    }
                                    else if let ConnectionStatus::Connected = state.server.connection_status {
                                        self.dispatch(state, Action::Disconnect);
                                    }
                                    else {
                                        self.app.close();
                                    }
                                },
                                _ => (),
                            }
                        },
                        GuiSelector::Arena => {
                            match key_event.code {
                                KeyCode::Enter => {
                                    if let GameStatus::Finished = state.server.game.status {
                                        state.server.game.status = GameStatus::NotStarted;
                                        state.gui.selector = GuiSelector::Menu;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                },

                InputEvent::ResizeDisplay(_, _) => {},
            }

            Action::InputServerAddrFocus => {
                state.gui.menu.server_addr_input.focus(true);
                state.gui.menu.character_input.focus(false);
            },

            Action::InputPlayerNameFocus => {
                state.gui.menu.server_addr_input.focus(false);
                state.gui.menu.character_input.focus(true);
            },
        }
    }
}
