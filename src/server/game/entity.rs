use crate::character::{Character};
use crate::vec2::{Vec2};
use crate::direction::{Direction};

use std::time::{Instant, Duration};
use std::rc::{Rc};

pub type EntityId = usize;

pub struct Entity {
    id: EntityId,
    character: Rc<Character>,
    direction: Direction,
    position: Vec2,
    live: usize,
    energy: usize,
    speed: f32,
    last_walk_moving: Instant,
}

impl Entity {
    pub fn new(id: EntityId, character: Rc<Character>, position: Vec2) -> Entity {
        let now = Instant::now();
        Entity {
            id,
            position,
            direction: Direction::Down,
            live: character.max_live(),
            energy: character.max_energy(),
            speed: character.speed_base(),
            last_walk_moving: now - Duration::from_secs_f32(1.0 / character.speed_base()),
            character,
        }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn character(&self) -> &Character {
        &*self.character
    }

    pub fn live(&self) -> usize {
        self.live
    }

    pub fn energy(&self) -> usize {
        self.energy
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn is_alive(&self) -> bool {
        self.live > 0
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn displace(&mut self, displacement: Vec2) {
        self.position += displacement;
    }

    pub fn walk(&mut self, direction: Direction, current: Instant) {
        let next_move_time = self.last_walk_moving + Duration::from_secs_f32(1.0 / self.speed);
        if current > next_move_time {
            self.position += direction.to_vec2();
        }
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
}
