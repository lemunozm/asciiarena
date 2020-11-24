use super::entity::{EntityId, Control as EntityControl, Action as EntityAction};

use crate::character::{Character};
use crate::direction::{Direction};

use std::collections::{VecDeque};

use std::rc::{Rc};

pub struct Player {
    character: Rc<Character>,
    entity_id: Option<EntityId>,
    total_points: usize,
    partial_points: usize,
    pending_actions: VecDeque<EntityAction>,
}

impl Player {
    pub const MAX_LIFE: usize = 100;
    pub const MAX_ENERGY: usize = 100;
    pub const SPEED_BASE: usize = 8;

    pub fn new(character: Rc<Character>) -> Player {
        Player {
            character,
            entity_id: None,
            total_points: 0,
            partial_points: 0,
            pending_actions: VecDeque::new(),
        }
    }

    pub fn character(&self) -> &Rc<Character> {
        &self.character
    }

    pub fn entity_id(&self) -> Option<EntityId> {
        self.entity_id
    }

    pub fn total_points(&self) -> usize {
        self.total_points
    }

    pub fn partial_points(&self) -> usize {
        self.partial_points
    }

    pub fn is_dead(&self) -> bool {
        self.entity_id.is_none()
    }

    pub fn walk(&mut self, direction: Direction) {
        self.pending_actions.push_back(EntityAction::Walk(direction));
    }

    pub fn update_points(&mut self, points: usize) {
        self.partial_points = points;
        self.total_points += points;
    }

    pub fn reset_partial_points(&mut self) {
        self.partial_points = 0;
    }

    pub fn attach_entity(&mut self, id: EntityId) {
        self.entity_id = Some(id);
    }
}

impl EntityControl for Player {
    fn next_action(&mut self) -> Option<EntityAction> {
        self.pending_actions.pop_front()
    }

    fn notify_death(&mut self) {
        self.entity_id = None;
    }
}
