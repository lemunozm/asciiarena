use crate::character::{Character};
use crate::vec2::{Vec2};
use crate::direction::{Direction};

use std::rc::{Rc};

pub type EntityId = usize;

pub enum Action {
    Walk(Direction),
   // Cast(Skill),
}

pub trait Control {
    fn next_action(&mut self) -> Option<Action>;
    fn notify_death(&mut self);
}

pub struct Entity {
    id: EntityId,
    character: Rc<Character>,
    direction: Direction,
    position: Vec2,
    live: usize,
    energy: usize,
}

impl Entity {
    pub fn new(id: EntityId, character: Rc<Character>, position: Vec2) -> Entity {
        Entity {
            id,
            position,
            direction: Direction::Down,
            live: character.max_live(),
            energy: character.max_energy(),
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

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn displace(&mut self, displacement: Vec2) {
        self.position += displacement;
    }

    pub fn walk(&mut self, direction: Direction) {
        //TODO: compute speed
        self.position += direction.to_vec2();
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
}
