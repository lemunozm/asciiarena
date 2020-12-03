use crate::message::{EntityId};
use crate::direction::{Direction};

use std::collections::{VecDeque};

pub enum EntityAction {
    Walk(Direction),
    Cast(usize /*Skill*/),
}

pub struct EntityControl {
    entity_id: EntityId,
    pending_actions: VecDeque<EntityAction>,
}

impl EntityControl {
    pub fn new(entity_id: EntityId) -> EntityControl {
        EntityControl {
            entity_id,
            pending_actions: VecDeque::new(),
        }
    }

    pub fn push_action(&mut self, action: EntityAction) {
        self.pending_actions.push_back(action);
    }

    pub fn pop_action(&mut self) -> Option<EntityAction> {
        self.pending_actions.pop_front()
    }

    pub fn detach_entity(&mut self) {
        self.entity_id = EntityId::NO_ENTITY;
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}
