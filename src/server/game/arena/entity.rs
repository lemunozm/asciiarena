use super::control::{Control};

use crate::character::{Character};
use crate::vec2::{Vec2};
use crate::direction::{Direction};
use crate::ids::{EntityId, SkillId};

use std::time::{Instant, Duration};
use std::rc::{Rc};
use std::cell::{RefCell};

pub enum EntityAction {
    Walk(Direction),
    Cast(SkillId),
}

pub type EntityControl = Control<EntityId, EntityAction>;

pub struct Entity {
    id: EntityId,
    character: Rc<Character>,
    control: Rc<RefCell<EntityControl>>,
    direction: Direction,
    position: Vec2,
    health: usize,
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
            health: character.max_health(),
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

    pub fn health(&self) -> usize {
        self.health
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
        self.health > 0
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn displace(&mut self, displacement: Vec2) {
        self.position += displacement;
    }

    pub fn add_health(&mut self, health: i32) {
        let new_health = self.health as i32 + health;
        if new_health > 0 {
            self.health = 0;
        }
        else if new_health as usize > self.character().max_health() {
            self.health = self.character().max_health();
        }
        else {
            self.health = new_health as usize;
        }
    }

    pub fn add_energy(&mut self, energy: i32) {
        let new_energy = self.energy as i32 + energy;
        if new_energy > 0 {
            self.energy = 0;
        }
        else if new_energy as usize > self.character().max_energy() {
            self.energy = self.character().max_energy();
        }
        else {
            self.energy = new_energy as usize;
        }
    }

    pub fn walk(&mut self, current: Instant) -> bool {
        if current > self.next_walk_time {
            self.position += self.direction.to_vec2();
            self.next_walk_time = current + Duration::from_secs_f32(1.0 / self.speed);
            return true
        }
        false
    }
}
