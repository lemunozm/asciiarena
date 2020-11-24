use super::control::{EntityControl, EntityAction};

use crate::character::{Character};
use crate::direction::{Direction};

use std::rc::{Rc};
use std::cell::{RefCell};

pub struct Player {
    character: Rc<Character>,
    control: Rc<RefCell<EntityControl>>,
    total_points: usize,
    partial_points: usize,
}

impl Player {
    pub const MAX_LIFE: usize = 100;
    pub const MAX_ENERGY: usize = 100;
    pub const SPEED_BASE: f32 = 8.0;

    pub fn new(character: Rc<Character>) -> Player {
        Player {
            character,
            control: Rc::new(RefCell::new(EntityControl::default())),
            total_points: 0,
            partial_points: 0,
        }
    }

    pub fn character(&self) -> &Rc<Character> {
        &self.character
    }

    pub fn control(&self) -> &Rc<RefCell<EntityControl>> {
        &self.control
    }

    pub fn total_points(&self) -> usize {
        self.total_points
    }

    pub fn partial_points(&self) -> usize {
        self.partial_points
    }

    pub fn is_alive(&self) -> bool {
        self.control.borrow().entity_id().is_some()
    }

    pub fn walk(&mut self, direction: Direction) {
        self.control.borrow_mut().push_action(EntityAction::Walk(direction));
    }

    pub fn update_points(&mut self, points: usize) {
        self.partial_points = points;
        self.total_points += points;
    }

    pub fn reset_partial_points(&mut self) {
        self.partial_points = 0;
    }

}
