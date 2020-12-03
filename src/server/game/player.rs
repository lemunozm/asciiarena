use super::entity::{EntityControl, EntityAction};

use crate::character::{Character};
use crate::direction::{Direction};

use std::rc::{Rc};
use std::cell::{RefCell, Ref};

pub struct Player {
    character: Rc<Character>,
    control: Option<Rc<RefCell<EntityControl>>>,
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
            control: None,
            total_points: 0,
            partial_points: 0,
        }
    }

    pub fn character(&self) -> &Rc<Character> {
        &self.character
    }

    pub fn control(&self) -> Option<Ref<'_, EntityControl>> {
        self.control.as_ref().map(|control| control.borrow())
    }

    pub fn total_points(&self) -> usize {
        self.total_points
    }

    pub fn partial_points(&self) -> usize {
        self.partial_points
    }

    pub fn is_alive(&self) -> bool {
        self.control.is_some()
    }

    pub fn set_control(&mut self, control: Rc<RefCell<EntityControl>>) {
        self.control = Some(control);
    }

    pub fn remove_control(&mut self) {
        self.control = None;
    }

    pub fn walk(&mut self, direction: Direction) {
        match &self.control {
            Some(control) => control.borrow_mut().push_action(EntityAction::Walk(direction)),
            None => panic!("The player must have an entity to move it"),
        }
    }

    pub fn update_points(&mut self, points: usize) {
        self.partial_points = points;
        self.total_points += points;
    }

    pub fn reset_partial_points(&mut self) {
        self.partial_points = 0;
    }

}
