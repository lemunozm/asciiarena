use crate::client::state::{State};

use crate::direction::{Direction};

use tui::widgets::{Widget};
use tui::buffer::{Buffer};
use tui::layout::{Rect};
use tui::style::{Style, Modifier};

use rand::{distributions::{Distribution, Uniform}, Rng};

use std::collections::{HashMap, HashSet};
use std::time::{Instant, Duration};

struct PlayerState {
    position: (u16, u16),
    direction: Direction,
    last_move: Instant,
}

pub struct WaitingRoom {
    dimension: (u16, u16),
    players: HashMap<char, PlayerState>,
}

impl WaitingRoom {
    pub fn new(x: u16, y: u16) -> WaitingRoom {
        WaitingRoom {
            dimension: (x / 2, y),
            players: HashMap::new(),
        }
    }

    pub fn update(&mut self, state: &State) {
        const MINIMAL_MOVE_TIME: Duration = Duration::from_millis(500);
        let mut rng = rand::thread_rng();

        self.players.retain(|player, _| state.server.logged_players.contains(player));
        for player in &state.server.logged_players {
            if !self.players.contains_key(player) {
                let x_range = Uniform::from(0..self.dimension.0);
                let y_range = Uniform::from(0..self.dimension.1);
                let position = (x_range.sample(&mut rng), y_range.sample(&mut rng));

                self.players.insert(*player, PlayerState{
                    position,
                    direction: rng.gen(),
                    last_move: Instant::now() - MINIMAL_MOVE_TIME,
                });
            }
        }

        let mut player_positions = self.players
            .values()
            .map(|state| state.position)
            .collect::<HashSet<_>>();

        let now = Instant::now();
        for (_, state) in &mut self.players {
            if now > state.last_move + MINIMAL_MOVE_TIME {
                state.last_move = now;
                let should_move = rng.gen::<f32>() < 0.85;
                if should_move {
                    let should_turn = rng.gen::<f32>() < 0.33;
                    if should_turn {
                        match rng.gen::<bool>() {
                            true => state.direction.turn_right(),
                            false => state.direction.turn_left(),
                        }
                    }

                    let movement = match state.direction {
                        Direction::Up if state.position.1 > 0 => {
                            (0, -1)
                        },
                        Direction::Left if state.position.0 > 0 => {
                            (-1, 0)
                        },
                        Direction::Down if state.position.1 < self.dimension.1 - 1 => {
                            (0, 1)
                        },
                        Direction::Right if state.position.0 < self.dimension.0 - 1 => {
                            (1, 0)
                        },
                        _ => (0, 0),
                    };

                    let new_position = (
                        (state.position.0 as i16 + movement.0) as u16,
                        (state.position.1 as i16 + movement.1) as u16,
                    );

                    if !player_positions.contains(&new_position) { // To avoid player collisions
                        state.position = new_position;
                        player_positions.insert(state.position);
                    }

                    if movement == (0, 0) { // An easy way to avoid to stop in the walls
                        state.direction.turn_right()
                    }
                }
            }
        }
    }
}

pub struct WaitingRoomWidget<'a>(pub &'a WaitingRoom);

impl Widget for WaitingRoomWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        for (player, state) in &self.0.players {
            let draw_at = (1 + state.position.0 * 2, 1 + state.position.1);
            if draw_at.0 < area.width || draw_at.1 < area.height {
                buffer
                    .get_mut(area.x + draw_at.0, area.y + draw_at.1)
                    .set_char(*player)
                    .set_style(Style::default().add_modifier(Modifier::BOLD));
            }
        }
    }
}

