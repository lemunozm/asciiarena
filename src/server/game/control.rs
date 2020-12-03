use crate::message::{EntityId};
use crate::direction::{Direction};

use std::collections::{VecDeque};

pub enum EntityAction {
    Walk(Direction),
    Cast(usize /*Skill*/),
}

#[derive(Default)]
pub struct EntityControl {
    entity_id: EntityId,
    pending_actions: VecDeque<EntityAction>,
}

impl EntityControl {
    pub fn push_action(&mut self, action: EntityAction) {
        self.pending_actions.push_back(action);
    }

    pub fn pop_action(&mut self) -> Option<EntityAction> {
        self.pending_actions.pop_front()
    }

    pub fn attach_entity(&mut self, id: EntityId) {
        self.entity_id = id;
    }

    pub fn detach_entity(&mut self) {
        self.entity_id = EntityId::NO_ENTITY;
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}
