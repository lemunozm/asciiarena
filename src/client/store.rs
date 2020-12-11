use super::state::{State, StaticGameInfo, VersionInfo, GameStatus, Arena,
    Player, UserPlayer};
use super::server_proxy::{ServerApi, ApiCall, ConnectionStatus, ServerEvent};

use crate::message::{GameEvent};
use crate::character::{CharacterId};
use crate::direction::{Direction};
use crate::ids::{EntityId, SkillId};
use crate::version::{self};

use std::net::{SocketAddr};
use std::time::{Instant};
use std::collections::{HashMap};

/// Action API
#[derive(Debug)]
pub enum Action {
    StartApp,
    Connect(SocketAddr),
    Disconnect,
    Login(char),
    Logout,
    CloseGame,
    CloseApp,
    MovePlayer(Direction),
    CastSkill(SkillId),
    ServerEvent(ServerEvent),
}

pub struct Store {
    state: State,
    server: ServerApi,
    close : bool,
}

impl Store {
    pub fn new(state: State, server: ServerApi) -> Store {
        Store {
            state,
            server,
            close: false,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn should_close(&self) -> bool {
        self.close
    }

    pub fn dispatch(&mut self, action: Action) {
        log::trace!("Dispatch: {:?}", action);
        match action {
            Action::StartApp => {
                if let Some(addr) = self.state.server.addr {
                    self.server.call(ApiCall::Connect(addr));
                }
            },

            Action::Connect(addr) => {
                self.state.server.addr = Some(addr);
                self.server.call(ApiCall::Connect(addr));
            },

            Action::Disconnect => {
                self.state.server.addr = None;
                self.server.call(ApiCall::Disconnect);
            }

            Action::Login(character) => {
                self.state.user.character_symbol = Some(character);
                self.server.call(ApiCall::Login(character));
            },

            Action::Logout => {
                self.state.user.character_symbol = None;
                self.state.user.login_status = None;
                self.server.call(ApiCall::Logout);
            }

            Action::CloseGame => {
                self.state.server.game.status = GameStatus::NotStarted;
                self.state.server.game.arena = None;
            }

            Action::CloseApp => {
                self.close = true;
            }

            Action::MovePlayer(direction) => {
                self.state.server.game.arena_mut().user_player.direction = direction;
                self.server.call(ApiCall::MovePlayer(direction));
            }

            Action::CastSkill(id) => {
                let direction = self.state.server.game.arena_mut().user_player.direction;
                self.server.call(ApiCall::CastSkill(direction, id));
            }

            Action::ServerEvent(server_event) => match server_event {
                ServerEvent::ConnectionResult(status)  => {
                    self.state.server.connection_status = status;
                    if let ConnectionStatus::Connected = status {
                        self.server.call(ApiCall::CheckVersion(version::current().into()));
                    }
                    else { //No connected (no matter the reason)
                        self.state.server.game.status = GameStatus::Finished;
                        self.state.server.udp_confirmed = None;
                        self.state.user.character_symbol = None;
                        self.state.user.login_status = None;
                        self.state.server.logged_players = Vec::new();
                        self.state.server.game.arena = None;
                        self.state.server.game_info = None;
                    }
                },

                ServerEvent::CheckedVersion(server_version, compatibility) => {
                    let version_info = VersionInfo { version: server_version, compatibility };
                    self.state.server.version_info = Some(version_info);

                    if compatibility.is_compatible() {
                        self.server.call(ApiCall::SubscribeInfo);
                    }
                    else {
                        // Protect the client against an unknown or not compatible server version
                        self.server.call(ApiCall::Disconnect);
                    }
                },

                ServerEvent::StaticServerInfo(info) => {
                    let game_info = StaticGameInfo {
                        players_number: info.players_number as usize,
                        map_size: info.map_size as usize,
                        winner_points: info.winner_points as usize,
                    };
                    self.state.server.udp_port = Some(info.udp_port);
                    self.state.server.game_info = Some(game_info);
                    self.state.server.logged_players = info.logged_players;

                    if let Some(character) = self.state.user.character_symbol {
                        self.server.call(ApiCall::Login(character));
                    }
                },

                ServerEvent::DynamicServerInfo(logged_players) => {
                    self.state.server.logged_players = logged_players;
                },

                ServerEvent::LoginStatus(status) => {
                    self.state.user.login_status = Some(status);
                },

                ServerEvent::UdpReachable(value) => {
                    self.state.server.udp_confirmed = Some(value);
                },

                ServerEvent::StartGame(game_info) => {
                    self.state.server.game.status = GameStatus::Started;
                    self.state.server.game.characters = game_info.characters
                        .into_iter()
                        .map(|character| (character.id(), character))
                        .collect();

                    self.state.server.game.players = game_info.players
                        .into_iter()
                        .enumerate()
                        .map(|(index, (character_id, points))| Player {
                            id: index,
                            character_id,
                            entity_id: EntityId::NONE,
                            points,
                        })
                        .collect();
                },

                ServerEvent::FinishGame => {
                    self.state.server.game.status = GameStatus::Finished;
                    self.state.server.udp_confirmed = None;
                    self.state.user.character_symbol = None;
                    self.state.user.login_status = None;
                    self.state.server.game.arena_mut().entities = HashMap::new();
                    self.state.server.game.arena_mut().spells = HashMap::new();
                },

                ServerEvent::WaitArena(duration) => {
                    self.state.server.game.next_arena_timestamp = Some(
                        Instant::now() + duration
                    );
                },

                ServerEvent::StartArena(arena_info) => {
                    self.state.server.game.next_arena_timestamp = None;
                    self.state.server.game.arena_number = arena_info.number;

                    for (i, player) in arena_info.players.into_iter().enumerate() {
                        self.state.server.game.players[i].entity_id = player;
                    }

                    self.state.server.game.arena = Some(Arena {
                        entities: HashMap::new(),
                        spells: HashMap::new(),
                        user_player: UserPlayer {
                            player_id: self.state.server.game.players
                                .iter()
                                .enumerate()
                                .find(|(_, player)| match player.character_id {
                                    CharacterId::Player(symbol) =>
                                        symbol == self.state.user.character_symbol.unwrap(),
                                    _ => false
                                })
                                .map(|(index, _)| index)
                                .unwrap(),
                            direction: Direction::Down,
                        }
                    });
                },

                ServerEvent::GameEvent(game_event) => {
                    let GameEvent::PlayerPointsUpdated(player_points) = game_event;
                    for (i, points) in player_points.into_iter().enumerate() {
                        self.state.server.game.players[i].points = points;
                    }
                }

                ServerEvent::GameStep(frame) => {
                    self.state.server.game.arena_mut().entities = frame.entities
                        .into_iter()
                        .map(|entity| (entity.id, entity))
                        .collect::<HashMap<_, _>>();

                    self.state.server.game.arena_mut().spells = frame.spells
                        .into_iter()
                        .map(|spell| (spell.id, spell))
                        .collect::<HashMap<_, _>>();
                },
            },
        }
    }
}
