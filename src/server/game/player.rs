use super::entity::{EntityId, Control as EntityControl, Action as EntityAction};

use crate::character::{Character};
use crate::direction::{Direction};

use std::collections::{VecDeque};

use std::rc::{Rc};
use std::cell::{RefCell};

pub struct Player {
    character: Rc<Character>,
    control: Rc<RefCell<PlayerControl>>,
    total_points: usize,
    partial_points: usize,
}

impl Player {
    pub const MAX_LIFE: usize = 100;
    pub const MAX_ENERGY: usize = 100;
    pub const SPEED_BASE: usize = 8;

    pub fn new(character: Rc<Character>) -> Player {
        Player {
            character,
            control: Rc::new(RefCell::new(PlayerControl::default())),
            total_points: 0,
            partial_points: 0,
        }
    }

    pub fn character(&self) -> &Rc<Character> {
        &self.character
    }

    pub fn control(&self) -> &Rc<RefCell<PlayerControl>> {
        &self.control
    }

    pub fn total_points(&self) -> usize {
        self.total_points
    }

    pub fn partial_points(&self) -> usize {
        self.partial_points
    }

    pub fn is_dead(&self) -> bool {
        self.control.borrow().entity_id.is_none()
    }

    pub fn walk(&mut self, direction: Direction) {
        self.control.borrow_mut().append_action(EntityAction::Walk(direction));
    }

    pub fn update_points(&mut self, points: usize) {
        self.partial_points = points;
        self.total_points += points;
    }

    pub fn reset_partial_points(&mut self) {
        self.partial_points = 0;
    }

}

#[derive(Default)]
pub struct PlayerControl {
    entity_id: Option<EntityId>,
    pending_actions: VecDeque<EntityAction>,
}

impl PlayerControl {
    fn append_action(&mut self, action: EntityAction) {
        self.pending_actions.push_back(action);
    }
}

impl EntityControl for PlayerControl {
    fn attach_entity(&mut self, id: EntityId) {
        self.entity_id = Some(id);
    }

    fn detach_entity(&mut self) {
        self.entity_id = None;
        self.pending_actions.clear();
    }

    fn next_action(&mut self) -> Option<EntityAction> {
        self.pending_actions.pop_front()
    }
}
