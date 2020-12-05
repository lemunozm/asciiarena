use super::control::{Control};

use crate::character::{Character};
use crate::vec2::{Vec2};
use crate::direction::{Direction};
use crate::ids::{EntityId};

use std::time::{Instant, Duration};
use std::rc::{Rc};
use std::cell::{RefCell};

pub enum EntityAction {
    Walk(Direction),
    Cast(usize /*Skill*/),
}

pub type EntityControl = Control<EntityId, EntityAction>;

pub struct Entity {
    id: EntityId,
    character: Rc<Character>,
    control: Rc<RefCell<EntityControl>>,
    direction: Direction,
    position: Vec2,
    live: usize,
    energy: usize,
    speed: f32,
    next_walk_time: Instant,
}

impl Entity {
    pub fn new(id: EntityId, character: Rc<Character>, position: Vec2) -> Entity {
        Entity {
            id,
            position,
            control: Rc::new(RefCell::new(EntityControl::new(id))),
            direction: Direction::Down,
            live: character.max_live(),
            energy: character.max_energy(),
            speed: character.speed_base(),
            next_walk_time: Instant::now(),
            character,
        }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn character(&self) -> &Character {
        &*self.character
    }

    pub fn control(&self) -> &Rc<RefCell<EntityControl>> {
        &self.control
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

    pub fn walk(&mut self, direction: Direction, current: Instant) -> bool {
        self.direction = direction;
        if current > self.next_walk_time {
            self.position += direction.to_vec2();
            self.next_walk_time = current + Duration::from_secs_f32(1.0 / self.speed);
            return true
        }
        false
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
}
